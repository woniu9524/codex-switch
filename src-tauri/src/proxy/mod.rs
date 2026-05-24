use crate::app::models::RequestEvent;
use crate::app::secrets;
use crate::app::store::AppStore;
use anyhow::{anyhow, Result};
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::header::{CONTENT_ENCODING, CONTENT_LENGTH, HOST};
use axum::http::{HeaderMap, Method, Response, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::TryStreamExt;
use serde_json::json;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Clone)]
struct ProxyState {
    store: Arc<AppStore>,
    client: reqwest::Client,
}

pub struct ProxyHandle {
    port: u16,
    shutdown: Option<oneshot::Sender<()>>,
}

impl ProxyHandle {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn stop(mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
    }
}

pub async fn start(store: Arc<AppStore>, port: u16) -> Result<ProxyHandle> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| anyhow!("本地端口 {port} 无法监听: {error}"))?;
    let actual_port = listener.local_addr()?.port();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let state = ProxyState { store, client };
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(forward))
        .route("/v1/responses", post(forward))
        .route("/v1/responses/compact", post(forward))
        .fallback(reject)
        .with_state(state);

    let (sender, receiver) = oneshot::channel::<()>();
    tauri::async_runtime::spawn(async move {
        let shutdown = async {
            let _ = receiver.await;
        };
        if let Err(error) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
        {
            log::error!("proxy server stopped with error: {error}");
        }
    });

    Ok(ProxyHandle {
        port: actual_port,
        shutdown: Some(sender),
    })
}

async fn health(State(state): State<ProxyState>) -> impl IntoResponse {
    let snapshot = state.store.snapshot_data();
    Json(json!({
        "ok": true,
        "activeProviderId": snapshot.active_provider_id,
        "port": snapshot.proxy_port,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn reject() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "message": "codex-switch only proxies GET /health, GET /v1/models, POST /v1/responses and POST /v1/responses/compact",
                "type": "codex_switch_path_not_allowed",
                "code": "path_not_allowed"
            }
        })),
    )
}

async fn forward(
    State(state): State<ProxyState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    let started_at = Instant::now();
    let path = uri
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| uri.path().to_string());
    let Some(provider) = state.store.active_provider() else {
        return json_error(
            StatusCode::PRECONDITION_FAILED,
            "provider_not_configured",
            "请先在 codex-switch 中选择一个供应商",
        );
    };
    let provider_id = provider.id.clone();
    let upstream_path = upstream_request_path(&path);
    let upstream_url = join_upstream_url(&provider.endpoint, &upstream_path);
    let request = match build_request(
        &state,
        &provider,
        &method,
        &path,
        &upstream_url,
        &headers,
        body,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(error) => {
            let status = StatusCode::UNAUTHORIZED;
            record_proxy_event(
                &state,
                &provider_id,
                &method,
                &path,
                status.as_u16(),
                started_at.elapsed().as_millis(),
                Some(error.to_string()),
            );
            return json_error(status, "provider_auth_missing", &error.to_string());
        }
    };
    let response = state.client.execute(request).await;

    match response {
        Ok(upstream) => {
            handle_upstream_response(&state, &provider_id, &method, &path, started_at, upstream)
                .await
        }
        Err(error) => {
            let status = StatusCode::BAD_GATEWAY;
            record_proxy_event(
                &state,
                &provider_id,
                &method,
                &path,
                status.as_u16(),
                started_at.elapsed().as_millis(),
                Some(error.to_string()),
            );
            json_error(
                status,
                "upstream_unreachable",
                "codex-switch failed to reach upstream",
            )
        }
    }
}

async fn build_request(
    state: &ProxyState,
    provider: &crate::app::models::Provider,
    method: &Method,
    request_path: &str,
    upstream_url: &str,
    headers: &HeaderMap,
    mut body: Bytes,
) -> Result<reqwest::Request> {
    let mut builder = state.client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes())?,
        upstream_url,
    );
    let api_key = secrets::get_provider_key(&provider.id)?;
    body = normalize_request_body(headers, body)?;
    if is_responses_compact_path(request_path) {
        body = rewrite_compact_model(body, &provider.model)?;
    }
    let body = if provider.disable_image_generation {
        strip_image_generation_tool(body.clone())?.unwrap_or(body)
    } else {
        body
    };

    for (name, value) in headers {
        if request_header_allowed(name.as_str()) {
            builder = builder.header(name, value);
        }
    }
    builder = builder.bearer_auth(api_key);
    Ok(builder.body(body).build()?)
}

async fn handle_upstream_response(
    state: &ProxyState,
    provider_id: &str,
    method: &Method,
    path: &str,
    started_at: Instant,
    upstream: reqwest::Response,
) -> Response<Body> {
    let status = upstream.status();
    let mut builder = Response::builder().status(status);
    for (name, value) in upstream.headers() {
        if response_header_allowed(name.as_str()) {
            builder = builder.header(name, value);
        }
    }
    if status.is_success() {
        let stream = upstream.bytes_stream().map_err(std::io::Error::other);
        record_proxy_event(
            state,
            provider_id,
            method,
            path,
            status.as_u16(),
            started_at.elapsed().as_millis(),
            None,
        );
        return builder
            .body(Body::from_stream(stream))
            .unwrap_or_else(|error| {
                json_error(
                    StatusCode::BAD_GATEWAY,
                    "proxy_response_build_failed",
                    &error.to_string(),
                )
            });
    }

    match upstream.bytes().await {
        Ok(bytes) => {
            let error = summarize_upstream_error(&bytes);
            record_proxy_event(
                state,
                provider_id,
                method,
                path,
                status.as_u16(),
                started_at.elapsed().as_millis(),
                error,
            );
            builder.body(Body::from(bytes)).unwrap_or_else(|error| {
                json_error(
                    StatusCode::BAD_GATEWAY,
                    "proxy_response_build_failed",
                    &error.to_string(),
                )
            })
        }
        Err(error) => json_error(
            StatusCode::BAD_GATEWAY,
            "proxy_response_read_failed",
            &error.to_string(),
        ),
    }
}

fn strip_image_generation_tool(body: Bytes) -> Result<Option<Bytes>> {
    let Ok(mut payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return Ok(None);
    };
    let Some(object) = payload.as_object_mut() else {
        return Ok(None);
    };
    let Some(tools) = object
        .get_mut("tools")
        .and_then(|value| value.as_array_mut())
    else {
        return Ok(None);
    };
    let before = tools.len();
    tools.retain(|tool| !is_image_generation_tool(tool));
    if tools.len() == before {
        return Ok(None);
    }
    if tools.is_empty() {
        object.remove("tools");
    }
    Ok(Some(Bytes::from(serde_json::to_vec(&payload)?)))
}

fn is_image_generation_tool(tool: &serde_json::Value) -> bool {
    tool.get("type")
        .and_then(|value| value.as_str())
        .is_some_and(|tool_type| tool_type == "image_generation")
}

fn request_header_allowed(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if hop_headers().contains(lower.as_str())
        || lower == HOST.as_str()
        || lower == CONTENT_LENGTH.as_str()
        || lower == CONTENT_ENCODING.as_str()
    {
        return false;
    }
    if matches!(lower.as_str(), "authorization" | "cookie") {
        return false;
    }
    true
}

fn normalize_request_body(headers: &HeaderMap, body: Bytes) -> Result<Bytes> {
    let Some(encoding) = headers
        .get(CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_ascii_lowercase())
    else {
        return Ok(body);
    };

    match encoding.as_str() {
        "identity" => Ok(body),
        "zstd" => Ok(Bytes::from(zstd::decode_all(body.as_ref())?)),
        other => Err(anyhow!("不支持的请求压缩格式 Content-Encoding: {other}")),
    }
}

fn is_responses_compact_path(path: &str) -> bool {
    let path = path
        .split_once('?')
        .map_or(path, |(path, _query)| path)
        .trim_end_matches('/');
    matches!(path, "/responses/compact" | "/v1/responses/compact")
}

fn upstream_request_path(path: &str) -> String {
    if !is_responses_compact_path(path) {
        return path.to_string();
    }

    match path.split_once('?') {
        Some((_path, query)) if !query.is_empty() => format!("/v1/responses?{query}"),
        _ => "/v1/responses".to_string(),
    }
}

fn rewrite_compact_model(body: Bytes, provider_model: &str) -> Result<Bytes> {
    let Ok(mut payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return Ok(body);
    };
    let Some(model) = payload.get_mut("model") else {
        return Ok(body);
    };
    let Some(model_name) = model.as_str() else {
        return Ok(body);
    };

    if is_codex_compact_model(model_name) {
        *model = serde_json::Value::String(provider_model.to_string());
        return Ok(Bytes::from(serde_json::to_vec(&payload)?));
    }

    Ok(body)
}

fn is_codex_compact_model(model: &str) -> bool {
    model
        .trim()
        .to_ascii_lowercase()
        .ends_with("-openai-compact")
}

fn response_header_allowed(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !hop_headers().contains(lower.as_str()) && lower != CONTENT_LENGTH.as_str()
}

fn hop_headers() -> HashSet<&'static str> {
    [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ]
    .into_iter()
    .collect()
}

fn join_upstream_url(base_url: &str, request_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = request_path.trim_start_matches('/');
    if base.ends_with("/v1") {
        let suffix = path.strip_prefix("v1/").unwrap_or(path);
        format!("{base}/{suffix}")
    } else {
        format!("{base}/{path}")
    }
}

fn record_proxy_event(
    state: &ProxyState,
    provider_id: &str,
    method: &Method,
    path: &str,
    status: u16,
    duration_ms: u128,
    error: Option<String>,
) {
    state.store.record_request(RequestEvent {
        timestamp: chrono::Utc::now().timestamp_millis(),
        provider_id: provider_id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        duration_ms,
        error,
    });
}

fn json_error(status: StatusCode, code: &str, message: &str) -> Response<Body> {
    let payload = json!({
        "error": {
            "message": message,
            "type": "codex_switch_error",
            "code": code
        }
    });
    Response::builder()
        .status(status)
        .header("content-type", "application/json; charset=utf-8")
        .body(Body::from(payload.to_string()))
        .expect("json error response")
}

fn summarize_upstream_error(bytes: &[u8]) -> Option<String> {
    if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(message) = payload
            .get("error")
            .and_then(|error| error.get("message").and_then(|value| value.as_str()))
            .or_else(|| payload.get("message").and_then(|value| value.as_str()))
        {
            let message = message.trim();
            if !message.is_empty() {
                return Some(message.to_string());
            }
        }
    }

    let text = String::from_utf8_lossy(bytes);
    let summary = text.trim();
    if summary.is_empty() {
        None
    } else {
        Some(summary.chars().take(240).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_responses_compact_path, join_upstream_url, normalize_request_body,
        request_header_allowed, rewrite_compact_model, strip_image_generation_tool,
        upstream_request_path,
    };
    use axum::http::header::CONTENT_ENCODING;
    use axum::http::HeaderMap;
    use bytes::Bytes;

    #[test]
    fn joins_without_double_v1() {
        assert_eq!(
            join_upstream_url("https://api.openai.com/v1", "/v1/responses"),
            "https://api.openai.com/v1/responses"
        );
    }

    #[test]
    fn decodes_zstd_request_body() {
        let body = br#"{"model":"gpt-5.5","stream":true}"#;
        let compressed = zstd::encode_all(body.as_slice(), 0).unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_ENCODING, "zstd".parse().unwrap());

        assert_eq!(
            normalize_request_body(&headers, Bytes::from(compressed)).unwrap(),
            Bytes::from_static(body)
        );
    }

    #[test]
    fn detects_compact_responses_path() {
        assert!(is_responses_compact_path("/v1/responses/compact"));
        assert!(is_responses_compact_path("/v1/responses/compact?foo=bar"));
        assert!(is_responses_compact_path("/responses/compact"));
        assert!(!is_responses_compact_path("/v1/responses"));
    }

    #[test]
    fn rewrites_compact_upstream_path_to_responses() {
        assert_eq!(
            upstream_request_path("/v1/responses/compact"),
            "/v1/responses"
        );
    }

    #[test]
    fn rewrites_codex_compact_model_to_provider_model() {
        let body = Bytes::from_static(br#"{"model":"gpt-5.5-openai-compact","stream":true}"#);
        let rewritten = rewrite_compact_model(body, "deepseek-chat").unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&rewritten).unwrap();

        assert_eq!(payload["model"], "deepseek-chat");
    }

    #[test]
    fn third_party_requests_strip_codex_credentials() {
        assert!(!request_header_allowed("Authorization"));
        assert!(!request_header_allowed("Cookie"));
        assert!(request_header_allowed("X-Request-Id"));
    }

    #[test]
    fn detects_and_strips_image_generation_tool() {
        let body = Bytes::from_static(
            br#"{"model":"gpt-5.5","tools":[{"type":"function","name":"shell_command"},{"type":"image_generation"}]}"#,
        );

        let stripped = strip_image_generation_tool(body).unwrap().unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&stripped).unwrap();
        let tools = payload["tools"].as_array().unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
    }

    #[test]
    fn removes_empty_tools_after_stripping_image_generation() {
        let body =
            Bytes::from_static(br#"{"model":"gpt-5.5","tools":[{"type":"image_generation"}]}"#);

        let stripped = strip_image_generation_tool(body).unwrap().unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&stripped).unwrap();

        assert!(payload.get("tools").is_none());
    }
}
