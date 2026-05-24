use super::models::{CodexDiagnostic, LoginMode};
use super::store::display_path;
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn codex_dir(override_dir: Option<&str>) -> Result<PathBuf> {
    if let Some(value) = override_dir.filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(value));
    }
    dirs::home_dir()
        .map(|home| home.join(".codex"))
        .ok_or_else(|| anyhow!("无法定位 Codex 配置目录"))
}

pub fn diagnose(override_dir: Option<&str>) -> CodexDiagnostic {
    let codex_dir = match codex_dir(override_dir) {
        Ok(path) => path,
        Err(error) => return unavailable_diagnostic(error.to_string()),
    };
    let config_path = codex_dir.join("config.toml");
    let auth_path = codex_dir.join("auth.json");
    let auth_exists = auth_path.exists();
    let (login_mode, detail) = match read_auth_json(&auth_path) {
        Ok(Some(auth)) => detect_login_mode(&auth),
        Ok(None) => (LoginMode::Unknown, "auth_json_missing".to_string()),
        Err(detail) => (LoginMode::Unknown, detail),
    };

    CodexDiagnostic {
        codex_dir: display_path(&codex_dir),
        config_path: display_path(&config_path),
        auth_path: display_path(&auth_path),
        config_exists: config_path.exists(),
        auth_exists,
        login_mode,
        detail,
    }
}

fn unavailable_diagnostic(detail: String) -> CodexDiagnostic {
    CodexDiagnostic {
        codex_dir: String::new(),
        config_path: String::new(),
        auth_path: String::new(),
        config_exists: false,
        auth_exists: false,
        login_mode: LoginMode::Unknown,
        detail,
    }
}

fn read_auth_json(auth_path: &Path) -> Result<Option<Value>, String> {
    let raw = match fs::read_to_string(auth_path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("auth_json_unreadable: {error}")),
    };
    serde_json::from_str::<Value>(&raw)
        .map(Some)
        .map_err(|error| format!("auth_json_invalid_json: {error}"))
}

fn detect_login_mode(auth: &Value) -> (LoginMode, String) {
    let Some(object) = auth.as_object() else {
        return (LoginMode::Unknown, "auth_json_not_object".to_string());
    };

    let api_key_auth = object.iter().any(|(key, value)| {
        key.ends_with("_API_KEY") && value.as_str().is_some_and(|s| !s.is_empty())
    });
    let chatgpt_auth = [
        "account",
        "accounts",
        "access_token",
        "chatgpt",
        "id_token",
        "oauth",
        "refresh_token",
        "tokens",
    ]
    .iter()
    .any(|key| object.get(*key).is_some_and(non_empty_auth_value));

    match (api_key_auth, chatgpt_auth) {
        (true, true) => (
            LoginMode::Mixed,
            "api_key_and_chatgpt_auth_detected".to_string(),
        ),
        (false, true) => (LoginMode::ChatGpt, "chatgpt_auth_detected".to_string()),
        (true, false) => (LoginMode::ApiKey, "api_key_auth_detected".to_string()),
        (false, false) => (LoginMode::Unknown, "auth_json_unclassified".to_string()),
    }
}

fn non_empty_auth_value(value: &Value) -> bool {
    match value {
        Value::String(value) => !value.trim().is_empty(),
        Value::Object(object) => object.values().any(non_empty_auth_value),
        Value::Array(values) => values.iter().any(non_empty_auth_value),
        Value::Null => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_chatgpt_login_mode() {
        let auth = json!({
            "tokens": {
                "access_token": "x.y.z"
            },
            "refresh_token": "rt-123"
        });

        let (mode, detail) = detect_login_mode(&auth);

        assert_eq!(mode, LoginMode::ChatGpt);
        assert_eq!(detail, "chatgpt_auth_detected");
    }

    #[test]
    fn reports_invalid_auth_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        std::fs::write(&path, "{").unwrap();

        let error = read_auth_json(&path).unwrap_err();

        assert!(error.starts_with("auth_json_invalid_json"));
    }
}
