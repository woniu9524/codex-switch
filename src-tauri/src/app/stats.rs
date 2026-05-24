use super::models::{DailyTokenUsage, ModelTokenUsage, StatsSummary, TokenSpeed, TokenUsage};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, TimeZone};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const DAILY_HISTORY_DAYS: i64 = 30;
const SPEED_WINDOW_DAYS: i64 = 7;

#[derive(Debug, Default)]
struct StatsAccumulator {
    all_time_usage: TokenUsage,
    today_usage: TokenUsage,
    last_7_days_usage: TokenUsage,
    daily_usage: BTreeMap<String, TokenUsage>,
    model_usage: HashMap<ModelKey, TokenUsage>,
    speed_output_tokens: u64,
    speed_duration_ms: u64,
    usage_event_count: usize,
    scanned_session_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ModelKey {
    provider_id: String,
    model: String,
}

#[derive(Debug)]
struct TimeBounds {
    today_start: DateTime<Local>,
    last_7_days_start: DateTime<Local>,
    daily_start: DateTime<Local>,
    now: DateTime<Local>,
}

#[derive(Debug, Default)]
struct SessionState {
    provider_id: String,
    current_model: String,
    previous_total: Option<TokenUsage>,
    pending_output_tokens: u64,
}

#[derive(Debug, Deserialize)]
struct RolloutEvent {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    event_type: String,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SessionMetaPayload {
    model_provider: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TurnContextPayload {
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventMessagePayload {
    #[serde(rename = "type")]
    message_type: Option<String>,
    duration_ms: Option<u64>,
    info: Option<TokenInfo>,
}

#[derive(Debug, Deserialize)]
struct TokenInfo {
    last_token_usage: Option<TokenUsage>,
    total_token_usage: Option<TokenUsage>,
}

pub fn stats_summary(codex_dir: &Path) -> Result<StatsSummary> {
    let now = Local::now();
    stats_summary_at(codex_dir, now)
}

fn stats_summary_at(codex_dir: &Path, now: DateTime<Local>) -> Result<StatsSummary> {
    let bounds = TimeBounds::new(now);
    let mut accumulator = StatsAccumulator::new(&bounds);
    let sessions_dir = codex_dir.join("sessions");

    for path in session_files(&sessions_dir)? {
        accumulator.scanned_session_count += 1;
        scan_session_file(&path, &bounds, &mut accumulator)
            .with_context(|| format!("scan Codex session {}", path.display()))?;
    }

    Ok(accumulator.finish(&bounds))
}

impl TimeBounds {
    fn new(now: DateTime<Local>) -> Self {
        let today_start = local_day_start(now);
        Self {
            today_start,
            last_7_days_start: now - Duration::days(SPEED_WINDOW_DAYS),
            daily_start: today_start - Duration::days(DAILY_HISTORY_DAYS - 1),
            now,
        }
    }
}

impl StatsAccumulator {
    fn new(bounds: &TimeBounds) -> Self {
        let mut daily_usage = BTreeMap::new();
        for offset in 0..DAILY_HISTORY_DAYS {
            let day = bounds.daily_start + Duration::days(offset);
            daily_usage.insert(day_key(day), TokenUsage::default());
        }
        Self {
            daily_usage,
            ..Self::default()
        }
    }

    fn record_usage(
        &mut self,
        timestamp: DateTime<Local>,
        provider_id: &str,
        model: &str,
        usage: &TokenUsage,
        bounds: &TimeBounds,
    ) {
        if usage.is_zero() {
            return;
        }

        self.all_time_usage.add_assign(usage);

        if timestamp >= bounds.today_start {
            self.today_usage.add_assign(usage);
        }
        if timestamp >= bounds.last_7_days_start {
            self.last_7_days_usage.add_assign(usage);
        }

        let date = day_key(timestamp);
        if let Some(daily) = self.daily_usage.get_mut(&date) {
            daily.add_assign(usage);
        }

        let key = ModelKey {
            provider_id: normalize_label(provider_id, "unknown"),
            model: normalize_label(model, "unknown"),
        };
        self.model_usage.entry(key).or_default().add_assign(usage);
        self.usage_event_count += 1;
    }

    fn record_speed_sample(
        &mut self,
        timestamp: DateTime<Local>,
        duration_ms: u64,
        output: u64,
        bounds: &TimeBounds,
    ) {
        if timestamp < bounds.last_7_days_start || duration_ms == 0 || output == 0 {
            return;
        }
        self.speed_output_tokens = self.speed_output_tokens.saturating_add(output);
        self.speed_duration_ms = self.speed_duration_ms.saturating_add(duration_ms);
    }

    fn finish(self, bounds: &TimeBounds) -> StatsSummary {
        let elapsed_hours =
            ((bounds.now - bounds.last_7_days_start).num_seconds().max(1) as f64) / 3600.0;
        let tokens_per_hour = self.last_7_days_usage.total_tokens as f64 / elapsed_hours;
        let output_tokens_per_second = if self.speed_duration_ms == 0 {
            0.0
        } else {
            self.speed_output_tokens as f64 / (self.speed_duration_ms as f64 / 1000.0)
        };

        let mut model_usage = self
            .model_usage
            .into_iter()
            .map(|(key, usage)| ModelTokenUsage {
                provider_id: key.provider_id,
                model: key.model,
                usage,
            })
            .collect::<Vec<_>>();
        model_usage.sort_by(|left, right| {
            right
                .usage
                .total_tokens
                .cmp(&left.usage.total_tokens)
                .then_with(|| left.provider_id.cmp(&right.provider_id))
                .then_with(|| left.model.cmp(&right.model))
        });

        StatsSummary {
            all_time_usage: self.all_time_usage,
            today_usage: self.today_usage,
            last_7_days_usage: self.last_7_days_usage,
            daily_usage: self
                .daily_usage
                .into_iter()
                .map(|(date, usage)| DailyTokenUsage { date, usage })
                .collect(),
            model_usage,
            speed: TokenSpeed {
                tokens_per_hour,
                output_tokens_per_second,
            },
            scanned_session_count: self.scanned_session_count,
            usage_event_count: self.usage_event_count,
        }
    }
}

impl TokenUsage {
    fn add_assign(&mut self, other: &Self) {
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.cached_input_tokens = self
            .cached_input_tokens
            .saturating_add(other.cached_input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.reasoning_output_tokens = self
            .reasoning_output_tokens
            .saturating_add(other.reasoning_output_tokens);
        self.total_tokens = self.total_tokens.saturating_add(other.total_tokens);
    }

    fn saturating_delta(&self, previous: &Self) -> Self {
        Self {
            input_tokens: self.input_tokens.saturating_sub(previous.input_tokens),
            cached_input_tokens: self
                .cached_input_tokens
                .saturating_sub(previous.cached_input_tokens),
            output_tokens: self.output_tokens.saturating_sub(previous.output_tokens),
            reasoning_output_tokens: self
                .reasoning_output_tokens
                .saturating_sub(previous.reasoning_output_tokens),
            total_tokens: self.total_tokens.saturating_sub(previous.total_tokens),
        }
    }

    fn is_zero(&self) -> bool {
        self.input_tokens == 0
            && self.cached_input_tokens == 0
            && self.output_tokens == 0
            && self.reasoning_output_tokens == 0
            && self.total_tokens == 0
    }
}

fn scan_session_file(
    path: &Path,
    bounds: &TimeBounds,
    accumulator: &mut StatsAccumulator,
) -> Result<()> {
    let raw = fs::read_to_string(path)?;
    let mut session = SessionState::default();

    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(event) = serde_json::from_str::<RolloutEvent>(line) else {
            continue;
        };
        let Some(timestamp) = parse_event_time(event.timestamp.as_deref()) else {
            continue;
        };

        match event.event_type.as_str() {
            "session_meta" => {
                if let Ok(payload) = serde_json::from_value::<SessionMetaPayload>(event.payload) {
                    if let Some(provider) = payload.model_provider {
                        session.provider_id = provider;
                    }
                }
            }
            "turn_context" => {
                if let Ok(payload) = serde_json::from_value::<TurnContextPayload>(event.payload) {
                    if let Some(model) = payload.model {
                        session.current_model = model;
                    }
                }
            }
            "event_msg" => {
                if let Ok(payload) = serde_json::from_value::<EventMessagePayload>(event.payload) {
                    handle_event_msg(timestamp, payload, &mut session, bounds, accumulator);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn handle_event_msg(
    timestamp: DateTime<Local>,
    payload: EventMessagePayload,
    session: &mut SessionState,
    bounds: &TimeBounds,
    accumulator: &mut StatsAccumulator,
) {
    match payload.message_type.as_deref() {
        Some("token_count") => {
            let Some(info) = payload.info else {
                return;
            };
            let Some(delta) = token_delta(&mut session.previous_total, &info) else {
                return;
            };
            session.pending_output_tokens = session
                .pending_output_tokens
                .saturating_add(delta.output_tokens);
            accumulator.record_usage(
                timestamp,
                &session.provider_id,
                &session.current_model,
                &delta,
                bounds,
            );
        }
        Some("task_complete") => {
            let duration_ms = payload.duration_ms.unwrap_or_default();
            let output = session.pending_output_tokens;
            session.pending_output_tokens = 0;
            accumulator.record_speed_sample(timestamp, duration_ms, output, bounds);
        }
        Some("turn_aborted") => {
            session.pending_output_tokens = 0;
        }
        _ => {}
    }
}

fn token_delta(previous_total: &mut Option<TokenUsage>, info: &TokenInfo) -> Option<TokenUsage> {
    if let Some(total) = info.total_token_usage.as_ref() {
        let delta = match previous_total.as_ref() {
            Some(previous) if total.total_tokens >= previous.total_tokens => {
                total.saturating_delta(previous)
            }
            _ => total.clone(),
        };
        *previous_total = Some(total.clone());
        return Some(delta);
    }

    info.last_token_usage.clone()
}

fn session_files(sessions_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if !sessions_dir.exists() {
        return Ok(files);
    }
    collect_session_files(sessions_dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_session_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_session_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn parse_event_time(value: Option<&str>) -> Option<DateTime<Local>> {
    let value = value?;
    DateTime::parse_from_rfc3339(value)
        .map(|datetime| datetime.with_timezone(&Local))
        .or_else(|_| {
            DateTime::parse_from_rfc3339(&format!("{value}Z"))
                .map(|datetime| datetime.with_timezone(&Local))
        })
        .ok()
}

fn local_day_start(value: DateTime<Local>) -> DateTime<Local> {
    Local
        .with_ymd_and_hms(value.year(), value.month(), value.day(), 0, 0, 0)
        .single()
        .unwrap_or(value)
}

fn day_key(value: DateTime<Local>) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn normalize_label(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accumulates_split_usage_without_duplicate_total_events() {
        let temp = tempfile::tempdir().unwrap();
        write_session(
            temp.path(),
            "2026-05-24T02:00:00Z",
            vec![
                session_meta("provider-a"),
                turn_context("model-a"),
                token_count(
                    "2026-05-24T02:01:00Z",
                    usage(100, 40, 20, 5, 120),
                    usage(100, 40, 20, 5, 120),
                ),
                token_count(
                    "2026-05-24T02:01:30Z",
                    usage(100, 40, 20, 5, 120),
                    usage(100, 40, 20, 5, 120),
                ),
                token_count(
                    "2026-05-24T02:03:00Z",
                    usage(30, 10, 7, 2, 37),
                    usage(130, 50, 27, 7, 157),
                ),
            ],
        );

        let summary = summary_for(temp.path());

        assert_eq!(summary.all_time_usage.total_tokens, 157);
        assert_eq!(summary.all_time_usage.input_tokens, 130);
        assert_eq!(summary.all_time_usage.cached_input_tokens, 50);
        assert_eq!(summary.all_time_usage.output_tokens, 27);
        assert_eq!(summary.all_time_usage.reasoning_output_tokens, 7);
        assert_eq!(summary.usage_event_count, 2);
    }

    #[test]
    fn groups_usage_by_model_after_model_switch() {
        let temp = tempfile::tempdir().unwrap();
        write_session(
            temp.path(),
            "2026-05-24T02:00:00Z",
            vec![
                session_meta("provider-a"),
                turn_context("model-a"),
                token_count(
                    "2026-05-24T02:01:00Z",
                    usage(10, 0, 5, 0, 15),
                    usage(10, 0, 5, 0, 15),
                ),
                turn_context("model-b"),
                token_count(
                    "2026-05-24T02:02:00Z",
                    usage(20, 0, 10, 0, 30),
                    usage(30, 0, 15, 0, 45),
                ),
            ],
        );

        let summary = summary_for(temp.path());
        let model_a = summary
            .model_usage
            .iter()
            .find(|item| item.model == "model-a")
            .unwrap();
        let model_b = summary
            .model_usage
            .iter()
            .find(|item| item.model == "model-b")
            .unwrap();

        assert_eq!(model_a.usage.total_tokens, 15);
        assert_eq!(model_b.usage.total_tokens, 30);
    }

    #[test]
    fn creates_daily_history_and_today_usage() {
        let temp = tempfile::tempdir().unwrap();
        write_session(
            temp.path(),
            "2026-05-24T02:00:00Z",
            vec![
                session_meta("provider-a"),
                turn_context("model-a"),
                token_count(
                    "2026-05-23T02:01:00Z",
                    usage(10, 0, 5, 0, 15),
                    usage(10, 0, 5, 0, 15),
                ),
                token_count(
                    "2026-05-24T02:02:00Z",
                    usage(20, 0, 10, 0, 30),
                    usage(30, 0, 15, 0, 45),
                ),
            ],
        );

        let summary = summary_for(temp.path());

        assert_eq!(summary.today_usage.total_tokens, 30);
        assert_eq!(summary.daily_usage.len(), 30);
        assert_eq!(
            summary
                .daily_usage
                .iter()
                .find(|item| item.date == "2026-05-23")
                .unwrap()
                .usage
                .total_tokens,
            15
        );
        assert_eq!(
            summary
                .daily_usage
                .iter()
                .find(|item| item.date == "2026-05-24")
                .unwrap()
                .usage
                .total_tokens,
            30
        );
    }

    #[test]
    fn estimates_output_speed_from_pending_output_and_task_duration() {
        let temp = tempfile::tempdir().unwrap();
        write_session(
            temp.path(),
            "2026-05-24T02:00:00Z",
            vec![
                session_meta("provider-a"),
                turn_context("model-a"),
                token_count(
                    "2026-05-24T02:01:00Z",
                    usage(10, 0, 100, 0, 110),
                    usage(10, 0, 100, 0, 110),
                ),
                task_complete("2026-05-24T02:02:00Z", 20_000),
            ],
        );

        let summary = summary_for(temp.path());

        assert!((summary.speed.output_tokens_per_second - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ignores_missing_or_broken_session_data() {
        let temp = tempfile::tempdir().unwrap();
        let sessions = temp.path().join("sessions").join("2026");
        fs::create_dir_all(&sessions).unwrap();
        fs::write(sessions.join("broken.jsonl"), "{not-json}\n").unwrap();

        let summary = summary_for(temp.path());

        assert_eq!(summary.scanned_session_count, 1);
        assert_eq!(summary.all_time_usage.total_tokens, 0);

        let empty = tempfile::tempdir().unwrap();
        let empty_summary = summary_for(empty.path());
        assert_eq!(empty_summary.scanned_session_count, 0);
    }

    fn summary_for(codex_dir: &Path) -> StatsSummary {
        let now = Local
            .with_ymd_and_hms(2026, 5, 24, 12, 0, 0)
            .single()
            .unwrap();
        stats_summary_at(codex_dir, now).unwrap()
    }

    fn write_session(codex_dir: &Path, start: &str, events: Vec<serde_json::Value>) {
        let sessions = codex_dir
            .join("sessions")
            .join("2026")
            .join("05")
            .join("24");
        fs::create_dir_all(&sessions).unwrap();
        let mut lines = vec![json!({
            "timestamp": start,
            "type": "session_meta",
            "payload": {"model_provider": "provider-a"}
        })];
        lines.extend(events);
        let raw = lines
            .into_iter()
            .map(|event| event.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(sessions.join("rollout-test.jsonl"), raw).unwrap();
    }

    fn session_meta(provider: &str) -> serde_json::Value {
        json!({
            "timestamp": "2026-05-24T02:00:00Z",
            "type": "session_meta",
            "payload": {"model_provider": provider}
        })
    }

    fn turn_context(model: &str) -> serde_json::Value {
        json!({
            "timestamp": "2026-05-24T02:00:10Z",
            "type": "turn_context",
            "payload": {"model": model}
        })
    }

    fn token_count(timestamp: &str, last: TokenUsage, total: TokenUsage) -> serde_json::Value {
        json!({
            "timestamp": timestamp,
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": last,
                    "total_token_usage": total
                }
            }
        })
    }

    fn task_complete(timestamp: &str, duration_ms: u64) -> serde_json::Value {
        json!({
            "timestamp": timestamp,
            "type": "event_msg",
            "payload": {
                "type": "task_complete",
                "duration_ms": duration_ms
            }
        })
    }

    fn usage(input: u64, cached: u64, output: u64, reasoning: u64, total: u64) -> TokenUsage {
        TokenUsage {
            input_tokens: input,
            cached_input_tokens: cached,
            output_tokens: output,
            reasoning_output_tokens: reasoning,
            total_tokens: total,
        }
    }
}
