use serde::{Deserialize, Serialize};

pub const SWITCH_PROVIDER_ID: &str = "codex-switch";
pub const DEFAULT_MODEL: &str = "gpt-5.5";
pub const DEFAULT_PROXY_PORT: u16 = 8787;
pub const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub api_key_ref: Option<String>,
    pub model: String,
    pub homepage: Option<String>,
    pub notes: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub disable_image_generation: bool,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub key_status: KeyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    Present,
    Missing,
}

impl Default for KeyStatus {
    fn default() -> Self {
        Self::Missing
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub openai_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredState {
    pub version: u32,
    pub enabled: bool,
    pub active_provider_id: Option<String>,
    pub proxy_port: u16,
    pub codex_dir_override: Option<String>,
    pub launch_at_login: bool,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    pub last_backup_path: Option<String>,
    pub last_written_model: Option<String>,
    pub restore_point: Option<RestorePoint>,
    pub providers: Vec<Provider>,
}

impl Default for StoredState {
    fn default() -> Self {
        Self {
            version: STATE_VERSION,
            enabled: false,
            active_provider_id: None,
            proxy_port: DEFAULT_PROXY_PORT,
            codex_dir_override: None,
            launch_at_login: false,
            theme_mode: ThemeMode::Light,
            last_backup_path: None,
            last_written_model: None,
            restore_point: None,
            providers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEvent {
    pub timestamp: i64,
    pub provider_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u128,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexDiagnostic {
    pub codex_dir: String,
    pub config_path: String,
    pub auth_path: String,
    pub config_exists: bool,
    pub auth_exists: bool,
    pub login_mode: LoginMode,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoginMode {
    ApiKey,
    ChatGpt,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePaths {
    pub app_home: String,
    pub state_file: String,
    pub backup_dir: String,
    pub log_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: StoredState,
    pub active_provider: Option<Provider>,
    pub proxy_running: bool,
    pub codex: CodexDiagnostic,
    pub paths: RuntimePaths,
    pub recent_logs: Vec<RequestEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInput {
    pub id: Option<String>,
    pub name: String,
    pub endpoint: String,
    pub api_key: Option<String>,
    pub model: String,
    pub homepage: Option<String>,
    pub notes: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub disable_image_generation: bool,
    pub switch_now: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub proxy_port: u16,
    pub codex_dir_override: Option<String>,
    pub launch_at_login: bool,
    pub theme_mode: ThemeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub scheme: String,
    pub source: String,
    pub name: String,
    pub endpoint: String,
    pub api_key_masked: Option<String>,
    pub has_api_key: bool,
    pub model: String,
    pub homepage: Option<String>,
    pub notes: Option<String>,
    pub icon: Option<String>,
    pub enabled_hint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelList {
    pub provider_id: String,
    pub models: Vec<String>,
}
