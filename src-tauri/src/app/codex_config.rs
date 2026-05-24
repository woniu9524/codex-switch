use super::codex_auth;
use super::models::{Provider, RestorePoint, SWITCH_PROVIDER_ID};
use super::store::{display_path, write_bytes_atomic, AppPaths};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut, Item, Table};

#[derive(Debug, Clone)]
pub struct ConfigWriteOutcome {
    pub backup_path: String,
    pub restore_point: RestorePoint,
    pub written_model: String,
}

pub fn config_path(codex_dir_override: Option<&str>) -> Result<PathBuf> {
    Ok(codex_auth::codex_dir(codex_dir_override)?.join("config.toml"))
}

pub fn write_proxy_config(
    paths: &AppPaths,
    codex_dir_override: Option<&str>,
    proxy_port: u16,
    provider: &Provider,
) -> Result<ConfigWriteOutcome> {
    let path = config_path(codex_dir_override)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 Codex 配置目录失败: {}", parent.display()))?;
    }

    let raw = read_config_text(&path)?;
    let mut doc = parse_doc(&raw)?;
    let restore_point = RestorePoint {
        model_provider: doc
            .get("model_provider")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
        model: doc
            .get("model")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
        openai_base_url: doc
            .get("openai_base_url")
            .and_then(|item| item.as_str())
            .map(ToString::to_string),
    };
    let backup_path = backup_config(paths, &path, raw.as_bytes())?;

    write_switch_provider(&mut doc, proxy_port, provider);

    write_text_atomic(&path, &doc.to_string())?;
    Ok(ConfigWriteOutcome {
        backup_path: display_path(&backup_path),
        restore_point,
        written_model: provider.model.clone(),
    })
}

pub fn update_codex_model(codex_dir_override: Option<&str>, model: &str) -> Result<()> {
    let path = config_path(codex_dir_override)?;
    let raw = read_config_text(&path)?;
    let mut doc = parse_doc(&raw)?;
    doc["model"] = value(model);
    write_text_atomic(&path, &doc.to_string())
}

pub fn remove_proxy_config(
    codex_dir_override: Option<&str>,
    restore_point: Option<RestorePoint>,
    last_written_model: Option<&str>,
) -> Result<()> {
    let path = config_path(codex_dir_override)?;
    let Some(raw) = read_existing_config_text(&path)? else {
        return Ok(());
    };
    let mut doc = parse_doc(&raw)?;

    restore_model_provider(&mut doc, restore_point.as_ref());
    restore_model(&mut doc, restore_point.as_ref(), last_written_model);
    restore_openai_base_url(&mut doc, restore_point.as_ref());
    remove_switch_provider(&mut doc);

    write_text_atomic(&path, &doc.to_string())
}

pub fn restore_backup(codex_dir_override: Option<&str>, backup_path: &Path) -> Result<()> {
    let config_path = config_path(codex_dir_override)?;
    let bytes = fs::read(backup_path)
        .with_context(|| format!("读取备份失败: {}", backup_path.display()))?;
    write_bytes_atomic(&config_path, &bytes)
}

fn restore_model_provider(doc: &mut DocumentMut, restore_point: Option<&RestorePoint>) {
    let is_switch = doc
        .get("model_provider")
        .and_then(|item| item.as_str())
        .is_some_and(|value| value == SWITCH_PROVIDER_ID);
    if !is_switch {
        return;
    }

    match restore_point.and_then(|restore| restore.model_provider.as_ref()) {
        Some(previous) => doc["model_provider"] = value(previous.as_str()),
        None => {
            doc.as_table_mut().remove("model_provider");
        }
    }
}

fn restore_model(
    doc: &mut DocumentMut,
    restore_point: Option<&RestorePoint>,
    last_written_model: Option<&str>,
) {
    let current_model = doc
        .get("model")
        .and_then(|item| item.as_str())
        .map(ToString::to_string);
    if current_model.as_deref() != last_written_model {
        return;
    }

    match restore_point.and_then(|restore| restore.model.as_ref()) {
        Some(previous) => doc["model"] = value(previous.as_str()),
        None => {
            doc.as_table_mut().remove("model");
        }
    }
}

fn restore_openai_base_url(doc: &mut DocumentMut, restore_point: Option<&RestorePoint>) {
    let current = doc
        .get("openai_base_url")
        .and_then(|item| item.as_str())
        .map(ToString::to_string);
    let should_restore = current.as_deref().is_some_and(is_local_proxy_url)
        || current.is_none()
            && restore_point
                .and_then(|restore| restore.openai_base_url.as_ref())
                .is_some();
    if !should_restore {
        return;
    }

    match restore_point.and_then(|restore| restore.openai_base_url.as_ref()) {
        Some(previous) => doc["openai_base_url"] = value(previous.as_str()),
        None => {
            doc.as_table_mut().remove("openai_base_url");
        }
    }
}

fn write_switch_provider(doc: &mut DocumentMut, proxy_port: u16, provider: &Provider) {
    doc["model_provider"] = value(SWITCH_PROVIDER_ID);
    doc["model"] = value(provider.model.as_str());
    doc.as_table_mut().remove("openai_base_url");

    if !doc.as_table().contains_key("model_providers") {
        doc["model_providers"] = Item::Table(Table::new());
    }
    let model_providers = doc["model_providers"]
        .as_table_mut()
        .expect("model_providers is table");

    model_providers.insert(SWITCH_PROVIDER_ID, Item::Table(Table::new()));
    let provider_table = model_providers[SWITCH_PROVIDER_ID]
        .as_table_mut()
        .expect("provider table");
    provider_table["name"] = value(SWITCH_PROVIDER_ID);
    provider_table["base_url"] = value(format!("http://127.0.0.1:{proxy_port}/v1"));
    provider_table["wire_api"] = value("responses");
    provider_table["requires_openai_auth"] = value(true);
    provider_table["supports_websockets"] = value(false);
}

fn remove_switch_provider(doc: &mut DocumentMut) {
    if let Some(model_providers) = doc
        .get_mut("model_providers")
        .and_then(|item| item.as_table_mut())
    {
        model_providers.remove(SWITCH_PROVIDER_ID);
    }
    remove_empty_model_providers(doc);
}

fn remove_empty_model_providers(doc: &mut DocumentMut) {
    let empty = doc
        .get("model_providers")
        .and_then(|item| item.as_table())
        .is_some_and(|table| table.is_empty());
    if empty {
        doc.as_table_mut().remove("model_providers");
    }
}

fn is_local_proxy_url(value: &str) -> bool {
    value.starts_with("http://127.0.0.1:")
}

fn parse_doc(raw: &str) -> Result<DocumentMut> {
    if raw.trim().is_empty() {
        Ok(DocumentMut::new())
    } else {
        raw.parse::<DocumentMut>()
            .map_err(|error| anyhow!("Codex config.toml 不是有效 TOML: {error}"))
    }
}

fn read_config_text(path: &Path) -> Result<String> {
    Ok(read_existing_config_text(path)?.unwrap_or_default())
}

fn read_existing_config_text(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("读取 Codex config.toml 失败: {}", path.display()))
        }
    }
}

fn backup_config(paths: &AppPaths, config_path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    fs::create_dir_all(&paths.backup_dir)?;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f");
    let backup_path = paths.backup_dir.join(format!("config-{stamp}.toml"));
    let backup_bytes = if config_path.exists() { bytes } else { b"" };
    write_bytes_atomic(&backup_path, backup_bytes)?;
    Ok(backup_path)
}

fn write_text_atomic(path: &Path, text: &str) -> Result<()> {
    write_bytes_atomic(path, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::models::{KeyStatus, DEFAULT_MODEL};

    #[test]
    fn switch_provider_preserves_other_toml_sections() {
        let mut doc = parse_doc(
            r#"model_provider = "openai"

[mcp_servers.demo]
command = "node"

[projects.'d:\codes\demo']
trust_level = "trusted"
"#,
        )
        .unwrap();

        write_switch_provider(&mut doc, 8787, &test_provider());
        let text = doc.to_string();

        assert!(text.contains("[mcp_servers.demo]"));
        assert!(text.contains("[projects.'d:\\codes\\demo']"));
        assert!(text.contains(r#"model_provider = "codex-switch""#));
        assert!(text.contains(r#"model = "gpt-5.5""#));
        assert!(text.contains("[model_providers.codex-switch]"));
        assert!(text.contains(r#"base_url = "http://127.0.0.1:8787/v1""#));
        assert!(!text.contains("openai_base_url"));
    }

    #[test]
    fn switch_provider_writes_http_only_provider_section() {
        let mut doc = parse_doc(
            r#"[mcp_servers.demo]
command = "node"
"#,
        )
        .unwrap();

        write_switch_provider(&mut doc, 8787, &test_provider());
        let text = doc.to_string();

        assert!(text.contains("[mcp_servers.demo]"));
        assert!(text.contains(r#"model_provider = "codex-switch""#));
        assert!(text.contains("[model_providers.codex-switch]"));
        assert!(text.contains("requires_openai_auth = true"));
        assert!(text.contains("supports_websockets = false"));
    }

    #[test]
    fn remove_proxy_config_cleans_old_switch_provider_table() {
        let mut doc = parse_doc(
            r#"model_provider = "codex-switch"
model = "gpt-5.5"
openai_base_url = "http://127.0.0.1:8787/v1"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:8787/v1"

[projects.'d:\codes\demo']
trust_level = "trusted"
"#,
        )
        .unwrap();
        let restore = RestorePoint {
            model_provider: None,
            model: Some("gpt-5-codex".to_string()),
            openai_base_url: None,
        };

        restore_model_provider(&mut doc, Some(&restore));
        restore_model(&mut doc, Some(&restore), Some("gpt-5.5"));
        restore_openai_base_url(&mut doc, Some(&restore));
        remove_switch_provider(&mut doc);
        let text = doc.to_string();

        assert!(!text.contains("codex-switch"));
        assert!(text.contains(r#"model = "gpt-5-codex""#));
        assert!(!text.contains("openai_base_url"));
        assert!(text.contains("[projects.'d:\\codes\\demo']"));
    }

    fn test_provider() -> Provider {
        Provider {
            id: "test-provider".to_string(),
            name: "Test Provider".to_string(),
            endpoint: "https://api.example.com/v1".to_string(),
            api_key_ref: Some("keyring://codex-switch/test-provider".to_string()),
            model: DEFAULT_MODEL.to_string(),
            homepage: None,
            notes: None,
            icon: None,
            disable_image_generation: false,
            created_at: 0,
            updated_at: 0,
            key_status: KeyStatus::Present,
        }
    }
}
