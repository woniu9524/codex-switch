pub mod codex_auth;
pub mod codex_config;
pub mod deeplink;
pub mod models;
pub mod provider_icons;
pub mod secrets;
pub mod store;

use crate::proxy::{self, ProxyHandle};
use anyhow::{anyhow, Result};
use models::{
    ImportPreview, KeyStatus, ModelList, Provider, ProviderInput, SettingsInput, Snapshot,
    UpdateInfo,
};
use reqwest::header::{ACCEPT, USER_AGENT};
use semver::Version;
use serde::Deserialize;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use store::AppStore;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_dialog::DialogExt;
use url::Url;
use uuid::Uuid;

pub struct RuntimeState {
    store: Arc<AppStore>,
    proxy: Mutex<Option<ProxyHandle>>,
    config_guard: Mutex<()>,
    exiting: AtomicBool,
}

impl RuntimeState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: Arc::new(AppStore::load()?),
            proxy: Mutex::new(None),
            config_guard: Mutex::new(()),
            exiting: AtomicBool::new(false),
        })
    }

    pub async fn start_proxy_if_enabled(self: &Arc<Self>) {
        if let Err(error) = self.recover_config_on_startup() {
            log::error!("failed to recover Codex config on startup: {error}");
        }

        let enabled = self.store.with_data(|state| state.enabled);
        if enabled {
            if let Err(error) = self.enable_current_provider(false).await {
                log::error!("failed to restore enabled state: {error}");
                self.stop_proxy();
            }
        }
    }

    pub fn cleanup_before_exit(&self) -> Result<()> {
        if self.exiting.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        if let Err(error) = self.release_config_lease() {
            self.exiting.store(false, Ordering::SeqCst);
            return Err(error);
        }
        self.stop_proxy();
        Ok(())
    }

    pub fn is_exiting(&self) -> bool {
        self.exiting.load(Ordering::SeqCst)
    }

    pub fn launch_at_login_enabled(&self) -> bool {
        self.store.with_data(|state| state.launch_at_login)
    }

    fn update_codex_model(&self, model: &str) -> Result<()> {
        let _guard = self.config_guard.lock().expect("config guard poisoned");
        let codex_dir_override = self
            .store
            .with_data(|state| state.codex_dir_override.clone());
        codex_config::update_codex_model(codex_dir_override.as_deref(), model)?;
        self.store.update(|state| {
            state.last_written_model = Some(model.to_string());
            if let Some(lease) = state.config_lease.as_mut() {
                lease.last_written_model = model.to_string();
            }
            Ok(())
        })?;
        Ok(())
    }

    async fn enable_current_provider(&self, mark_enabled: bool) -> Result<()> {
        if self.exiting.load(Ordering::SeqCst) {
            return Err(anyhow!("应用正在退出，无法启用代理"));
        }

        let active = self
            .store
            .active_provider()
            .ok_or_else(|| anyhow!("请先添加并选择一个供应商"))?;
        ensure_provider_can_route(&active).map_err(|error| anyhow!(error))?;

        self.release_config_lease()?;
        let port = self.start_proxy().await?;
        if self.exiting.load(Ordering::SeqCst) {
            self.stop_proxy();
            return Ok(());
        }

        if let Err(error) = self.apply_proxy_config_for_provider(port, &active, mark_enabled) {
            self.stop_proxy();
            return Err(error);
        }
        Ok(())
    }

    async fn start_proxy(&self) -> Result<u16> {
        if let Some(handle) = self.proxy.lock().expect("proxy poisoned").as_ref() {
            return Ok(handle.port());
        }

        let requested_port = self.store.with_data(|state| state.proxy_port);
        let port = find_available_port(requested_port)?;
        let handle = proxy::start(self.store.clone(), port).await?;
        let actual_port = handle.port();
        {
            let mut guard = self.proxy.lock().expect("proxy poisoned");
            *guard = Some(handle);
        }
        self.store.update(|state| {
            state.proxy_port = actual_port;
            Ok(())
        })?;
        Ok(actual_port)
    }

    fn stop_proxy(&self) {
        let handle = {
            let mut guard = self.proxy.lock().expect("proxy poisoned");
            guard.take()
        };
        if let Some(handle) = handle {
            handle.stop();
        }
    }

    fn proxy_running(&self) -> bool {
        self.proxy.lock().expect("proxy poisoned").is_some()
    }

    fn recover_config_on_startup(&self) -> Result<()> {
        self.release_config_lease()
    }

    fn release_config_lease(&self) -> Result<()> {
        let _guard = self.config_guard.lock().expect("config guard poisoned");
        let (lease, codex_dir_override) = self
            .store
            .with_data(|state| (state.config_lease.clone(), state.codex_dir_override.clone()));

        if let Some(lease) = lease {
            codex_config::release_config_lease(&lease)?;
            self.store.update(|state| {
                state.config_lease = None;
                state.last_written_model = None;
                Ok(())
            })?;
        } else {
            codex_config::cleanup_stale_switch_config(codex_dir_override.as_deref())?;
        }
        Ok(())
    }

    fn cleanup_stale_switch_config_for(&self, codex_dir_override: Option<String>) -> Result<()> {
        let _guard = self.config_guard.lock().expect("config guard poisoned");
        codex_config::cleanup_stale_switch_config(codex_dir_override.as_deref())
    }

    fn apply_proxy_config_for_provider(
        &self,
        port: u16,
        active: &Provider,
        mark_enabled: bool,
    ) -> Result<()> {
        let _guard = self.config_guard.lock().expect("config guard poisoned");
        if self.exiting.load(Ordering::SeqCst) {
            return Ok(());
        }

        let codex_dir_override = self
            .store
            .with_data(|state| state.codex_dir_override.clone());
        codex_config::cleanup_stale_switch_config(codex_dir_override.as_deref())?;
        let lease = codex_config::prepare_config_lease(
            self.store.paths(),
            codex_dir_override,
            port,
            active,
        )?;
        self.store.update(|state| {
            state.config_lease = Some(lease.clone());
            state.last_backup_path = Some(lease.backup_path.clone());
            state.last_written_model = Some(lease.last_written_model.clone());
            Ok(())
        })?;
        codex_config::apply_config_lease(&lease, active)?;
        let lease = codex_config::mark_lease_applied(&lease);
        self.store.update(|state| {
            if mark_enabled {
                state.enabled = true;
            }
            state.active_provider_id = Some(active.id.clone());
            state.proxy_port = port;
            state.last_backup_path = Some(lease.backup_path.clone());
            state.last_written_model = Some(lease.last_written_model.clone());
            state.config_lease = Some(lease);
            Ok(())
        })?;
        Ok(())
    }
}

#[tauri::command]
pub async fn snapshot(runtime: State<'_, Arc<RuntimeState>>) -> Result<Snapshot, String> {
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn update_info(app: AppHandle) -> Result<UpdateInfo, String> {
    const LATEST_RELEASE_API: &str =
        "https://api.github.com/repos/woniu9524/codex-switch/releases/latest";

    let current_version = app.package_info().version.to_string();
    let response = reqwest::Client::new()
        .get(LATEST_RELEASE_API)
        .header(ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, "codex-switch-update-check")
        .send()
        .await
        .map_err(|error| format!("检查更新失败：无法连接 GitHub Release API: {error}"))?;

    let status = response.status();
    let body = response
        .bytes()
        .await
        .map_err(|error| format!("检查更新失败：读取 GitHub 返回内容失败: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "检查更新失败：GitHub Release API 返回 {}: {}",
            status.as_u16(),
            summarize_response_body(&body)
        ));
    }

    let release: GithubLatestRelease = serde_json::from_slice(&body)
        .map_err(|error| format!("检查更新失败：GitHub Release 数据格式异常: {error}"))?;
    let latest_version = normalize_release_version(&release.tag_name);
    let has_update = compare_versions(&current_version, latest_version.as_deref());
    let notes = if let Some(latest) = latest_version.as_deref() {
        if has_update {
            format!("发现新版本 {latest}，你当前是 {current_version}")
        } else {
            format!("当前已是最新版本 {current_version}")
        }
    } else {
        format!(
            "已读取到最新 Release 标签 {}，但未能解析出标准版本号",
            release.tag_name
        )
    };

    Ok(UpdateInfo {
        current_version,
        latest_version,
        has_update,
        release_url: release.html_url,
        checked_at: chrono::Utc::now().timestamp_millis(),
        notes,
    })
}

#[tauri::command]
pub async fn enable(runtime: State<'_, Arc<RuntimeState>>) -> Result<Snapshot, String> {
    runtime
        .enable_current_provider(true)
        .await
        .map_err(to_user_error)?;
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn disable(runtime: State<'_, Arc<RuntimeState>>) -> Result<Snapshot, String> {
    runtime.release_config_lease().map_err(to_user_error)?;
    runtime.stop_proxy();
    runtime
        .store
        .update(|state| {
            state.enabled = false;
            state.config_lease = None;
            state.last_written_model = None;
            Ok(())
        })
        .map_err(to_user_error)?;
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn save_provider(
    input: ProviderInput,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<Snapshot, String> {
    let provider = save_provider_inner(&runtime.store, input).map_err(to_user_error)?;
    ensure_provider_icon(&runtime, &provider)
        .await
        .map_err(to_user_error)?;
    refresh_active_config_if_needed(&runtime, &provider)
        .await
        .map_err(to_user_error)?;
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn delete_provider(
    provider_id: String,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<Snapshot, String> {
    runtime
        .store
        .update(|state| {
            state
                .providers
                .iter()
                .find(|provider| provider.id == provider_id)
                .ok_or_else(|| anyhow!("供应商不存在"))?;
            if state.active_provider_id.as_deref() == Some(provider_id.as_str()) {
                return Err(anyhow!("当前使用中的供应商不能删除"));
            }
            state
                .providers
                .retain(|provider| provider.id != provider_id);
            Ok(())
        })
        .map_err(to_user_error)?;
    secrets::delete_provider_key(&provider_id);
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn switch_provider(
    provider_id: String,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<Snapshot, String> {
    let provider = runtime
        .store
        .provider_by_id(&provider_id)
        .ok_or_else(|| "供应商不存在".to_string())?;
    ensure_provider_can_route(&provider)?;

    let enabled = runtime.store.with_data(|state| state.enabled);
    if enabled {
        runtime
            .update_codex_model(&provider.model)
            .map_err(to_user_error)?;
    }

    runtime
        .store
        .update(|state| {
            state.active_provider_id = Some(provider.id.clone());
            state.last_written_model = Some(provider.model.clone());
            Ok(())
        })
        .map_err(to_user_error)?;
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn parse_import_url(url: String) -> Result<ImportPreview, String> {
    deeplink::parse_import_url(&url).map_err(to_user_error)
}

#[tauri::command]
pub async fn import_provider(
    url: String,
    switch_now: bool,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<Snapshot, String> {
    let provider_import = deeplink::parse_import(&url).map_err(to_user_error)?;
    let preview = provider_import.preview;
    let input = ProviderInput {
        id: None,
        name: preview.name,
        endpoint: preview.endpoint,
        api_key: provider_import.api_key,
        model: preview.model,
        homepage: preview.homepage,
        notes: preview.notes,
        icon: preview.icon,
        disable_image_generation: false,
        switch_now,
    };
    let provider = save_provider_inner(&runtime.store, input).map_err(to_user_error)?;
    ensure_provider_icon(&runtime, &provider)
        .await
        .map_err(to_user_error)?;
    if switch_now {
        ensure_provider_can_route(&provider)?;
        runtime
            .store
            .update(|state| {
                state.active_provider_id = Some(provider.id.clone());
                state.last_written_model = Some(provider.model.clone());
                Ok(())
            })
            .map_err(to_user_error)?;
        refresh_active_config_if_needed(&runtime, &provider)
            .await
            .map_err(to_user_error)?;
    }
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn provider_api_key(
    provider_id: String,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<String, String> {
    let provider = runtime
        .store
        .provider_by_id(&provider_id)
        .ok_or_else(|| "供应商不存在".to_string())?;
    secrets::get_provider_key(&provider.id).map_err(to_user_error)
}

#[tauri::command]
pub async fn fetch_provider_models(
    provider_id: String,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<ModelList, String> {
    let provider = runtime
        .store
        .provider_by_id(&provider_id)
        .ok_or_else(|| "供应商不存在".to_string())?;
    ensure_provider_can_route(&provider)?;

    let api_key = secrets::get_provider_key(&provider.id).map_err(to_user_error)?;
    let url = join_provider_url(&provider.endpoint, "/v1/models");
    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|error| format!("模型列表请求失败: {error}"))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("读取模型列表响应失败: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "模型列表请求返回 {}: {}",
            status.as_u16(),
            summarize_response_body(&bytes)
        ));
    }

    Ok(ModelList {
        provider_id,
        models: parse_model_ids(&bytes)?,
    })
}

#[tauri::command]
pub async fn update_settings(
    input: SettingsInput,
    app: AppHandle,
    runtime: State<'_, Arc<RuntimeState>>,
) -> Result<Snapshot, String> {
    if input.proxy_port == 0 {
        return Err("代理端口必须大于 0".to_string());
    }
    let next_codex_dir_override = normalize_optional(input.codex_dir_override.clone());
    let (was_enabled, previous_port, previous_codex_dir_override, previous_launch_at_login) =
        runtime.store.with_data(|state| {
            (
                state.enabled,
                state.proxy_port,
                state.codex_dir_override.clone(),
                state.launch_at_login,
            )
        });
    runtime
        .store
        .update(|state| {
            state.proxy_port = input.proxy_port;
            state.codex_dir_override = next_codex_dir_override.clone();
            state.launch_at_login = input.launch_at_login;
            state.theme_mode = input.theme_mode;
            state.language_mode = input.language_mode;
            Ok(())
        })
        .map_err(to_user_error)?;

    if previous_launch_at_login != input.launch_at_login {
        if let Err(error) = apply_launch_at_login(&app, input.launch_at_login) {
            let rollback = runtime.store.update(|state| {
                state.launch_at_login = previous_launch_at_login;
                Ok(())
            });
            if let Err(rollback_error) = rollback {
                log::error!(
                    "failed to roll back launch-at-login after autostart sync error: {rollback_error}"
                );
            }
            return Err(format!("鏇存柊寮€鏈哄惎鍔ㄥけ璐? {error}"));
        }
    }

    let needs_proxy_config_refresh = was_enabled
        && (previous_port != input.proxy_port
            || previous_codex_dir_override != next_codex_dir_override);
    if needs_proxy_config_refresh {
        let changed_codex_dir = previous_codex_dir_override != next_codex_dir_override;
        runtime.release_config_lease().map_err(to_user_error)?;
        if changed_codex_dir {
            runtime
                .cleanup_stale_switch_config_for(previous_codex_dir_override)
                .map_err(to_user_error)?;
        }

        runtime.stop_proxy();
        let port = runtime.start_proxy().await.map_err(to_user_error)?;
        let active = runtime
            .store
            .active_provider()
            .ok_or_else(|| "当前没有可用供应商，无法刷新代理配置".to_string())?;
        runtime
            .apply_proxy_config_for_provider(port, &active, false)
            .map_err(to_user_error)?;
    }

    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn restore_backup(runtime: State<'_, Arc<RuntimeState>>) -> Result<Snapshot, String> {
    let (codex_dir_override, backup_path) = runtime.store.with_data(|state| {
        (
            state.codex_dir_override.clone(),
            state.last_backup_path.clone(),
        )
    });
    let backup_path = backup_path.ok_or_else(|| "暂无可恢复备份".to_string())?;
    codex_config::restore_backup(codex_dir_override.as_deref(), &PathBuf::from(backup_path))
        .map_err(to_user_error)?;
    runtime
        .store
        .update(|state| {
            state.enabled = false;
            Ok(())
        })
        .map_err(to_user_error)?;
    runtime.stop_proxy();
    snapshot_inner(&runtime).map_err(to_user_error)
}

#[tauri::command]
pub async fn select_codex_directory(app: AppHandle) -> Result<Option<String>, String> {
    let folder = tokio::task::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("选择 Codex 目录")
            .blocking_pick_folder()
    })
    .await
    .map_err(|error| format!("选择目录失败: {error}"))?;

    folder
        .map(|path| {
            path.into_path()
                .map(|path| path.to_string_lossy().to_string())
                .map_err(|error| format!("读取目录路径失败: {error}"))
        })
        .transpose()
}

fn snapshot_inner(runtime: &RuntimeState) -> Result<Snapshot> {
    let state = runtime.store.snapshot_data();
    let active_provider = state
        .providers
        .iter()
        .find(|provider| Some(provider.id.as_str()) == state.active_provider_id.as_deref())
        .cloned();
    let codex = codex_auth::diagnose(state.codex_dir_override.as_deref());
    Ok(Snapshot {
        state,
        active_provider,
        proxy_running: runtime.proxy_running(),
        codex,
        paths: runtime.store.paths().dto(),
        recent_logs: runtime.store.recent_logs(8),
    })
}

async fn refresh_active_config_if_needed(
    runtime: &RuntimeState,
    provider: &Provider,
) -> Result<()> {
    let is_active = runtime
        .store
        .with_data(|state| state.active_provider_id.as_deref() == Some(provider.id.as_str()));
    let enabled = runtime.store.with_data(|state| state.enabled);
    if is_active && enabled {
        runtime.update_codex_model(&provider.model)?;
    }
    Ok(())
}

async fn ensure_provider_icon(runtime: &RuntimeState, provider: &Provider) -> Result<()> {
    if provider
        .icon
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return Ok(());
    }
    let Some(icon) = provider_icons::resolve_provider_icon(
        runtime.store.paths(),
        &provider.id,
        provider.homepage.as_deref(),
        &provider.endpoint,
    )
    .await
    else {
        return Ok(());
    };

    let provider_id = provider.id.clone();
    runtime.store.update(|state| {
        if let Some(provider) = state
            .providers
            .iter_mut()
            .find(|item| item.id == provider_id)
        {
            if provider
                .icon
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                provider.icon = Some(icon);
            }
        }
        Ok(())
    })?;
    Ok(())
}

fn save_provider_inner(store: &AppStore, input: ProviderInput) -> Result<Provider> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(anyhow!("供应商名称不能为空"));
    }
    let endpoint = normalize_endpoint(&input.endpoint)?;
    let model = input.model.trim();
    if model.is_empty() {
        return Err(anyhow!("默认模型不能为空"));
    }
    if input.switch_now && !provider_has_key_after_save(input.id.as_deref(), &input.api_key) {
        return Err(anyhow!("缺少 API Key，不能设为当前供应商"));
    }

    let now = chrono::Utc::now().timestamp_millis();
    store.update(|state| {
        if let Some(id) = input.id.as_ref() {
            let provider = state
                .providers
                .iter_mut()
                .find(|provider| provider.id == *id)
                .ok_or_else(|| anyhow!("供应商不存在"))?;
            provider.name = name.to_string();
            provider.endpoint = endpoint.clone();
            provider.model = model.to_string();
            provider.homepage = normalize_optional(input.homepage.clone());
            provider.notes = normalize_optional(input.notes.clone());
            provider.icon = normalize_optional(input.icon.clone());
            provider.disable_image_generation = input.disable_image_generation;
            provider.updated_at = now;
            if let Some(api_key) = normalize_api_key(input.api_key.as_deref()) {
                provider.api_key_ref = Some(secrets::set_provider_key(id, api_key)?);
                provider.key_status = KeyStatus::Present;
            }
            if input.switch_now {
                state.active_provider_id = Some(id.clone());
                state.last_written_model = Some(model.to_string());
            }
            return Ok(provider.clone());
        }

        let id = unique_provider_id(name);
        let api_key = normalize_api_key(input.api_key.as_deref());
        let api_key_ref = api_key
            .map(|api_key| secrets::set_provider_key(&id, api_key))
            .transpose()?;
        let has_api_key = api_key_ref.is_some();
        let provider = Provider {
            id: id.clone(),
            name: name.to_string(),
            endpoint,
            api_key_ref,
            model: model.to_string(),
            homepage: normalize_optional(input.homepage.clone()),
            notes: normalize_optional(input.notes.clone()),
            icon: normalize_optional(input.icon.clone()),
            disable_image_generation: input.disable_image_generation,
            created_at: now,
            updated_at: now,
            key_status: if has_api_key {
                KeyStatus::Present
            } else {
                KeyStatus::Missing
            },
        };
        if input.switch_now {
            state.active_provider_id = Some(id);
            state.last_written_model = Some(provider.model.clone());
        }
        state.providers.push(provider.clone());
        Ok(provider)
    })
}

fn provider_has_key_after_save(provider_id: Option<&str>, api_key: &Option<String>) -> bool {
    normalize_api_key(api_key.as_deref()).is_some()
        || provider_id.is_some_and(secrets::provider_key_available)
}

fn ensure_provider_can_route(provider: &Provider) -> Result<(), String> {
    if !secrets::provider_key_available(&provider.id) {
        return Err("缺少 API Key，不能切换到第三方供应商".to_string());
    }
    if provider.model.trim().is_empty() {
        return Err("缺少默认模型，不能启用供应商".to_string());
    }
    Ok(())
}

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Deserialize)]
struct GithubLatestRelease {
    tag_name: String,
    html_url: String,
}

#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}

fn normalize_release_version(tag: &str) -> Option<String> {
    let normalized = tag.trim().trim_start_matches(['v', 'V']).trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn compare_versions(current: &str, latest: Option<&str>) -> bool {
    let Some(latest) = latest else {
        return false;
    };

    let Ok(current) = Version::parse(current.trim()) else {
        return false;
    };
    let Ok(latest) = Version::parse(latest.trim()) else {
        return false;
    };

    latest > current
}

fn join_provider_url(base_url: &str, request_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let path = request_path.trim_start_matches('/');
    if base.ends_with("/v1") {
        let suffix = path.strip_prefix("v1/").unwrap_or(path);
        format!("{base}/{suffix}")
    } else {
        format!("{base}/{path}")
    }
}

fn summarize_response_body(bytes: &[u8]) -> String {
    if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if let Some(message) = payload
            .get("error")
            .and_then(|error| error.get("message").and_then(|value| value.as_str()))
            .or_else(|| payload.get("message").and_then(|value| value.as_str()))
        {
            let message = message.trim();
            if !message.is_empty() {
                return message.to_string();
            }
        }
    }

    let summary = String::from_utf8_lossy(bytes).trim().to_string();
    if summary.is_empty() {
        "空响应".to_string()
    } else {
        summary.chars().take(240).collect()
    }
}

fn parse_model_ids(bytes: &[u8]) -> Result<Vec<String>, String> {
    let payload: OpenAiModelsResponse = serde_json::from_slice(bytes)
        .map_err(|error| format!("模型列表不是 OpenAI-compatible JSON: {error}"))?;
    let mut models = payload
        .data
        .into_iter()
        .map(|model| model.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    models.sort();
    models.dedup();
    if models.is_empty() {
        return Err("模型列表为空".to_string());
    }
    Ok(models)
}

fn normalize_endpoint(value: &str) -> Result<String> {
    let endpoint = value.trim().trim_end_matches('/').to_string();
    let url = Url::parse(&endpoint).map_err(|error| anyhow!("Endpoint 不是有效 URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(anyhow!("Endpoint 必须是 http(s) URL"));
    }
    Ok(endpoint)
}

fn normalize_api_key(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn unique_provider_id(name: &str) -> String {
    let slug = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else if ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let slug = if slug.is_empty() {
        "provider".to_string()
    } else {
        slug
    };
    let suffix = Uuid::new_v4().simple().to_string();
    format!("{slug}-{}", &suffix[..8])
}

fn find_available_port(start: u16) -> Result<u16> {
    for port in start..start.saturating_add(80) {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
    Err(anyhow!("从 {start} 开始没有找到可用端口"))
}

fn apply_launch_at_login(app: &AppHandle, enabled: bool) -> Result<()> {
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable()?;
    } else {
        autostart.disable()?;
    }
    Ok(())
}

fn to_user_error(error: anyhow::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_api_key, normalize_endpoint, parse_model_ids, summarize_response_body,
        unique_provider_id,
    };

    #[test]
    fn normalizes_endpoint() {
        assert_eq!(
            normalize_endpoint("https://api.example.com/v1/").unwrap(),
            "https://api.example.com/v1"
        );
    }

    #[test]
    fn rejects_non_http_endpoint() {
        assert!(normalize_endpoint("file:///tmp/demo").is_err());
    }

    #[test]
    fn normalizes_blank_api_keys() {
        assert_eq!(normalize_api_key(Some("  sk-test  ")), Some("sk-test"));
        assert_eq!(normalize_api_key(Some("   ")), None);
        assert_eq!(normalize_api_key(None), None);
    }

    #[test]
    fn provider_id_has_slug_and_suffix() {
        let id = unique_provider_id("Deep Seek");
        assert!(id.starts_with("deep-seek-"));
    }

    #[test]
    fn parses_openai_compatible_models() {
        let body = br#"{"data":[{"id":"gpt-4.1"},{"id":"gpt-4.1"},{"id":" deepseek-chat "}]}"#;
        let models = parse_model_ids(body).unwrap();

        assert_eq!(models, vec!["deepseek-chat", "gpt-4.1"]);
    }

    #[test]
    fn rejects_non_standard_model_payload() {
        let error = parse_model_ids(br#"{"models":["gpt-4.1"]}"#).unwrap_err();

        assert!(error.contains("OpenAI-compatible"));
    }

    #[test]
    fn summarizes_model_fetch_error_body() {
        let body = br#"{"error":{"message":"invalid api key"}}"#;

        assert_eq!(summarize_response_body(body), "invalid api key");
    }
}
