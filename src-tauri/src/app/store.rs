use super::models::{KeyStatus, Provider, RequestEvent, RuntimePaths, StoredState, STATE_VERSION};
use super::secrets;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

pub struct AppStore {
    paths: AppPaths,
    data: RwLock<StoredState>,
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub app_home: PathBuf,
    pub state_file: PathBuf,
    pub backup_dir: PathBuf,
    pub log_dir: PathBuf,
    pub provider_icon_dir: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("无法定位用户主目录"))?;
        let app_home = home.join(".codex-switch");
        Ok(Self {
            state_file: app_home.join("state").join("state.json"),
            backup_dir: app_home.join("backups"),
            log_dir: app_home.join("logs"),
            provider_icon_dir: app_home.join("icons").join("providers"),
            app_home,
        })
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(self.state_file.parent().expect("state file has parent"))
            .with_context(|| format!("创建状态目录失败: {}", self.state_file.display()))?;
        fs::create_dir_all(&self.backup_dir)
            .with_context(|| format!("创建备份目录失败: {}", self.backup_dir.display()))?;
        fs::create_dir_all(&self.log_dir)
            .with_context(|| format!("创建日志目录失败: {}", self.log_dir.display()))?;
        fs::create_dir_all(&self.provider_icon_dir).with_context(|| {
            format!(
                "创建供应商图标目录失败: {}",
                self.provider_icon_dir.display()
            )
        })?;
        Ok(())
    }

    pub fn dto(&self) -> RuntimePaths {
        RuntimePaths {
            app_home: display_path(&self.app_home),
            state_file: display_path(&self.state_file),
            backup_dir: display_path(&self.backup_dir),
            log_dir: display_path(&self.log_dir),
        }
    }
}

impl AppStore {
    pub fn load() -> Result<Self> {
        let paths = AppPaths::discover()?;
        paths.ensure()?;

        let mut data = read_state_file(&paths.state_file)?;
        normalize_state(&mut data);

        let store = Self {
            paths,
            data: RwLock::new(data),
        };
        store.save()?;
        Ok(store)
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn snapshot_data(&self) -> StoredState {
        let data = self.data.read().expect("state poisoned");
        let mut state = data.clone();
        refresh_key_statuses(&mut state.providers);
        state
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&StoredState) -> T) -> T {
        let data = self.data.read().expect("state poisoned");
        f(&data)
    }

    pub fn update<T>(&self, f: impl FnOnce(&mut StoredState) -> Result<T>) -> Result<T> {
        let result = {
            let mut data = self.data.write().expect("state poisoned");
            let result = f(&mut data)?;
            normalize_state(&mut data);
            result
        };
        self.save()?;
        Ok(result)
    }

    pub fn save(&self) -> Result<()> {
        let data = self.data.read().expect("state poisoned").clone();
        write_json_atomic(&self.paths.state_file, &data)
    }

    pub fn active_provider(&self) -> Option<Provider> {
        let state = self.data.read().expect("state poisoned");
        let active_id = state.active_provider_id.as_deref()?;
        state
            .providers
            .iter()
            .find(|provider| provider.id == active_id)
            .cloned()
    }

    pub fn provider_by_id(&self, id: &str) -> Option<Provider> {
        self.snapshot_data()
            .providers
            .into_iter()
            .find(|provider| provider.id == id)
    }

    pub fn record_request(&self, event: RequestEvent) {
        let _ = append_proxy_log(&self.paths.log_dir, &event);
    }

    pub fn recent_logs(&self, limit: usize) -> Vec<RequestEvent> {
        let path = self.paths.log_dir.join("proxy.jsonl");
        let Ok(raw) = fs::read_to_string(path) else {
            return Vec::new();
        };
        let mut events = raw
            .lines()
            .filter_map(|line| serde_json::from_str::<RequestEvent>(line).ok())
            .collect::<Vec<_>>();
        let keep_from = events.len().saturating_sub(limit);
        events.drain(0..keep_from);
        events
    }
}

fn read_state_file(path: &Path) -> Result<StoredState> {
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<StoredState>(&raw)
            .with_context(|| format!("状态文件不是有效 JSON: {}", path.display())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(StoredState::default()),
        Err(error) => Err(error).with_context(|| format!("读取状态文件失败: {}", path.display())),
    }
}

pub fn normalize_state(state: &mut StoredState) {
    state.version = STATE_VERSION;

    state
        .providers
        .sort_by_key(|provider| provider.name.to_ascii_lowercase());

    let active_provider_exists = state
        .providers
        .iter()
        .any(|provider| Some(provider.id.as_str()) == state.active_provider_id.as_deref());
    if !active_provider_exists {
        state.active_provider_id = None;
    }
}

fn refresh_key_statuses(providers: &mut [Provider]) {
    for provider in providers {
        provider.key_status = if secrets::provider_key_available(&provider.id) {
            KeyStatus::Present
        } else {
            KeyStatus::Missing
        };
    }
}

fn append_proxy_log(log_dir: &Path, event: &RequestEvent) -> Result<()> {
    fs::create_dir_all(log_dir)?;
    let path = log_dir.join("proxy.jsonl");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    write_bytes_atomic(path, &bytes)
}

pub fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("tmp");
    {
        let mut tmp = fs::File::create(&tmp_path)?;
        tmp.write_all(bytes)?;
        tmp.sync_all()?;
    }
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_state;
    use crate::app::models::{KeyStatus, Provider, StoredState};

    #[test]
    fn normalize_keeps_user_providers_only() {
        let mut state = StoredState::default();
        state.providers.push(test_provider("zeta"));

        normalize_state(&mut state);

        assert_eq!(state.providers.len(), 1);
        assert_eq!(state.providers[0].id, "zeta");
        assert_eq!(state.active_provider_id, None);
    }

    #[test]
    fn normalize_clears_missing_active_provider() {
        let mut state = StoredState {
            active_provider_id: Some("missing".to_string()),
            providers: vec![test_provider("alpha")],
            ..StoredState::default()
        };

        normalize_state(&mut state);

        assert_eq!(state.active_provider_id, None);
    }

    #[test]
    fn normalize_preserves_existing_active_provider() {
        let mut state = StoredState {
            active_provider_id: Some("alpha".to_string()),
            providers: vec![test_provider("zeta"), test_provider("alpha")],
            ..StoredState::default()
        };

        normalize_state(&mut state);

        assert_eq!(state.active_provider_id, Some("alpha".to_string()));
        assert_eq!(state.providers[0].id, "alpha");
        assert_eq!(state.providers[1].id, "zeta");
    }

    fn test_provider(name: &str) -> Provider {
        Provider {
            id: name.to_string(),
            name: name.to_string(),
            endpoint: "https://api.example.com/v1".to_string(),
            api_key_ref: None,
            model: "demo".to_string(),
            homepage: None,
            notes: None,
            icon: None,
            disable_image_generation: false,
            created_at: 0,
            updated_at: 0,
            key_status: KeyStatus::Missing,
        }
    }
}
