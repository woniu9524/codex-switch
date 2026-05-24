use super::codex_auth;
use super::models::{
    ConfigLease, ConfigLeaseStatus, OriginalCodexConfig, Provider, SWITCH_PROVIDER_ID,
};
use super::store::{display_path, write_bytes_atomic, AppPaths};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut, Item, Table};

pub fn config_path(codex_dir_override: Option<&str>) -> Result<PathBuf> {
    Ok(codex_auth::codex_dir(codex_dir_override)?.join("config.toml"))
}

pub fn prepare_config_lease(
    paths: &AppPaths,
    codex_dir_override: Option<String>,
    proxy_port: u16,
    provider: &Provider,
) -> Result<ConfigLease> {
    let path = config_path(codex_dir_override.as_deref())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("创建 Codex 配置目录失败: {}", parent.display()))?;
    }

    let raw = read_config_text(&path)?;
    let doc = parse_doc(&raw)?;
    let original_config = OriginalCodexConfig {
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
        switch_provider_table: doc
            .get("model_providers")
            .and_then(|item| item.as_table())
            .and_then(|table| table.get(SWITCH_PROVIDER_ID))
            .and_then(|item| item.as_table())
            .map(|table| table.to_string()),
    };
    let backup_path = backup_config(paths, &path, raw.as_bytes())?;

    Ok(ConfigLease {
        codex_dir_override,
        backup_path: display_path(&backup_path),
        original_config,
        last_written_model: provider.model.clone(),
        proxy_port,
        status: ConfigLeaseStatus::Pending,
    })
}

pub fn apply_config_lease(lease: &ConfigLease, provider: &Provider) -> Result<()> {
    let path = config_path(lease.codex_dir_override.as_deref())?;
    let raw = read_config_text(&path)?;
    let mut doc = parse_doc(&raw)?;
    write_switch_provider(&mut doc, lease.proxy_port, provider);
    write_text_atomic(&path, &doc.to_string())
}

pub fn mark_lease_applied(lease: &ConfigLease) -> ConfigLease {
    let mut lease = lease.clone();
    lease.status = ConfigLeaseStatus::Applied;
    lease
}

pub fn update_codex_model(codex_dir_override: Option<&str>, model: &str) -> Result<()> {
    let path = config_path(codex_dir_override)?;
    let raw = read_config_text(&path)?;
    let mut doc = parse_doc(&raw)?;
    doc["model"] = value(model);
    write_text_atomic(&path, &doc.to_string())
}

pub fn release_config_lease(lease: &ConfigLease) -> Result<()> {
    let path = config_path(lease.codex_dir_override.as_deref())?;
    let Some(raw) = read_existing_config_text(&path)? else {
        return Ok(());
    };
    let mut doc = parse_doc(&raw)?;

    restore_model_provider(&mut doc, &lease.original_config);
    restore_model(&mut doc, &lease.original_config, &lease.last_written_model);
    restore_openai_base_url(&mut doc, &lease.original_config);
    restore_switch_provider_table(&mut doc, &lease.original_config)?;

    write_text_atomic(&path, &doc.to_string())
}

pub fn cleanup_stale_switch_config(codex_dir_override: Option<&str>) -> Result<()> {
    let path = config_path(codex_dir_override)?;
    let Some(raw) = read_existing_config_text(&path)? else {
        return Ok(());
    };
    let mut doc = parse_doc(&raw)?;
    if remove_stale_switch_provider(&mut doc) {
        write_text_atomic(&path, &doc.to_string())?;
    }
    Ok(())
}

pub fn restore_backup(codex_dir_override: Option<&str>, backup_path: &Path) -> Result<()> {
    let config_path = config_path(codex_dir_override)?;
    let bytes = fs::read(backup_path)
        .with_context(|| format!("读取备份失败: {}", backup_path.display()))?;
    write_bytes_atomic(&config_path, &bytes)
}

fn restore_model_provider(doc: &mut DocumentMut, original: &OriginalCodexConfig) {
    let is_switch = doc
        .get("model_provider")
        .and_then(|item| item.as_str())
        .is_some_and(|value| value == SWITCH_PROVIDER_ID);
    if !is_switch {
        return;
    }

    match original.model_provider.as_ref() {
        Some(previous) => doc["model_provider"] = value(previous.as_str()),
        None => {
            doc.as_table_mut().remove("model_provider");
        }
    }
}

fn restore_model(doc: &mut DocumentMut, original: &OriginalCodexConfig, last_written_model: &str) {
    let current_model = doc
        .get("model")
        .and_then(|item| item.as_str())
        .map(ToString::to_string);
    if current_model.as_deref() != Some(last_written_model) {
        return;
    }

    match original.model.as_ref() {
        Some(previous) => doc["model"] = value(previous.as_str()),
        None => {
            doc.as_table_mut().remove("model");
        }
    }
}

fn restore_openai_base_url(doc: &mut DocumentMut, original: &OriginalCodexConfig) {
    let current = doc
        .get("openai_base_url")
        .and_then(|item| item.as_str())
        .map(ToString::to_string);
    let should_restore = current.as_deref().is_some_and(is_local_proxy_url)
        || current.is_none() && original.openai_base_url.is_some();
    if !should_restore {
        return;
    }

    match original.openai_base_url.as_ref() {
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

fn restore_switch_provider_table(
    doc: &mut DocumentMut,
    original: &OriginalCodexConfig,
) -> Result<()> {
    let should_restore_table = doc
        .get("model_providers")
        .and_then(|item| item.as_table())
        .and_then(|table| table.get(SWITCH_PROVIDER_ID))
        .and_then(|item| item.as_table())
        .is_some_and(is_managed_switch_provider)
        || doc
            .get("model_provider")
            .and_then(|item| item.as_str())
            .is_some_and(|value| value == SWITCH_PROVIDER_ID);

    if !should_restore_table {
        return Ok(());
    }

    match original.switch_provider_table.as_ref() {
        Some(table_text) => insert_switch_provider_table(doc, table_text)?,
        None => remove_switch_provider_table(doc),
    }
    remove_empty_model_providers(doc);
    Ok(())
}

fn insert_switch_provider_table(doc: &mut DocumentMut, table_text: &str) -> Result<()> {
    if !doc.as_table().contains_key("model_providers") {
        doc["model_providers"] = Item::Table(Table::new());
    }
    let wrapper = format!("[model_providers.{SWITCH_PROVIDER_ID}]\n{table_text}");
    let parsed = parse_doc(&wrapper)?;
    let table = parsed
        .get("model_providers")
        .and_then(|item| item.as_table())
        .and_then(|table| table.get(SWITCH_PROVIDER_ID))
        .and_then(|item| item.as_table())
        .ok_or_else(|| anyhow!("stored codex-switch provider table is invalid"))?
        .clone();
    let model_providers = doc["model_providers"]
        .as_table_mut()
        .expect("model_providers is table");
    model_providers.insert(SWITCH_PROVIDER_ID, Item::Table(table));
    Ok(())
}

fn remove_switch_provider_table(doc: &mut DocumentMut) {
    if let Some(model_providers) = doc
        .get_mut("model_providers")
        .and_then(|item| item.as_table_mut())
    {
        model_providers.remove(SWITCH_PROVIDER_ID);
    }
}

fn remove_stale_switch_provider(doc: &mut DocumentMut) -> bool {
    let mut changed = false;
    let is_switch = doc
        .get("model_provider")
        .and_then(|item| item.as_str())
        .is_some_and(|value| value == SWITCH_PROVIDER_ID);
    if is_switch {
        doc.as_table_mut().remove("model_provider");
        changed = true;
    }

    let has_switch_provider = doc
        .get("model_providers")
        .and_then(|item| item.as_table())
        .and_then(|table| table.get(SWITCH_PROVIDER_ID))
        .is_some();
    if has_switch_provider {
        remove_switch_provider_table(doc);
        remove_empty_model_providers(doc);
        changed = true;
    }
    changed
}

fn is_managed_switch_provider(table: &Table) -> bool {
    table
        .get("base_url")
        .and_then(|item| item.as_str())
        .is_some_and(is_local_proxy_url)
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
    fn release_lease_cleans_old_switch_provider_table() {
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
        let original = OriginalCodexConfig {
            model_provider: None,
            model: Some("gpt-5-codex".to_string()),
            openai_base_url: None,
            switch_provider_table: None,
        };

        restore_model_provider(&mut doc, &original);
        restore_model(&mut doc, &original, "gpt-5.5");
        restore_openai_base_url(&mut doc, &original);
        restore_switch_provider_table(&mut doc, &original).unwrap();
        let text = doc.to_string();

        assert!(!text.contains("codex-switch"));
        assert!(text.contains(r#"model = "gpt-5-codex""#));
        assert!(!text.contains("openai_base_url"));
        assert!(text.contains("[projects.'d:\\codes\\demo']"));
    }

    #[test]
    fn stale_cleanup_removes_switch_without_lease() {
        let mut doc = parse_doc(
            r#"model_provider = "codex-switch"
model = "gpt-5.5"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:8787/v1"
"#,
        )
        .unwrap();

        assert!(remove_stale_switch_provider(&mut doc));
        let text = doc.to_string();

        assert!(!text.contains(r#"model_provider = "codex-switch""#));
        assert!(!text.contains("[model_providers.codex-switch]"));
        assert!(text.contains(r#"model = "gpt-5.5""#));
    }

    #[test]
    fn release_lease_restores_existing_switch_provider_table() {
        let mut doc = parse_doc(
            r#"model_provider = "codex-switch"
model = "gpt-5.5"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:8787/v1"
"#,
        )
        .unwrap();
        let original = OriginalCodexConfig {
            model_provider: Some("openai".to_string()),
            model: Some("gpt-5-codex".to_string()),
            openai_base_url: Some("https://api.openai.com/v1".to_string()),
            switch_provider_table: Some(
                r#"name = "User Provider"
base_url = "https://example.test/v1"
wire_api = "responses"
"#
                .to_string(),
            ),
        };

        restore_model_provider(&mut doc, &original);
        restore_model(&mut doc, &original, "gpt-5.5");
        restore_openai_base_url(&mut doc, &original);
        restore_switch_provider_table(&mut doc, &original).unwrap();
        let text = doc.to_string();

        assert!(text.contains(r#"model_provider = "openai""#));
        assert!(text.contains(r#"model = "gpt-5-codex""#));
        assert!(text.contains(r#"openai_base_url = "https://api.openai.com/v1""#));
        assert!(text.contains(r#"name = "User Provider""#));
        assert!(text.contains(r#"base_url = "https://example.test/v1""#));
    }

    #[test]
    fn lease_roundtrip_restores_original_provider_file() {
        let temp = tempfile::tempdir().unwrap();
        let codex_dir = temp.path().join("codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"model_provider = "openai"
model = "gpt-5-codex"

[projects.'d:\codes\demo']
trust_level = "trusted"
"#,
        )
        .unwrap();
        let paths = test_paths(temp.path());
        let lease = prepare_config_lease(
            &paths,
            Some(display_path(&codex_dir)),
            8787,
            &test_provider(),
        )
        .unwrap();

        apply_config_lease(&lease, &test_provider()).unwrap();
        let switched = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        assert!(switched.contains(r#"model_provider = "codex-switch""#));

        release_config_lease(&lease).unwrap();
        let restored = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
        assert!(restored.contains(r#"model_provider = "openai""#));
        assert!(restored.contains(r#"model = "gpt-5-codex""#));
        assert!(!restored.contains("[model_providers.codex-switch]"));
        assert!(restored.contains("[projects.'d:\\codes\\demo']"));
    }

    #[test]
    fn lease_roundtrip_removes_missing_original_provider_file() {
        let temp = tempfile::tempdir().unwrap();
        let codex_dir = temp.path().join("codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            "[mcp_servers.demo]\ncommand = \"node\"\n",
        )
        .unwrap();
        let paths = test_paths(temp.path());
        let lease = prepare_config_lease(
            &paths,
            Some(display_path(&codex_dir)),
            8787,
            &test_provider(),
        )
        .unwrap();

        apply_config_lease(&lease, &test_provider()).unwrap();
        release_config_lease(&lease).unwrap();
        let restored = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();

        assert!(!restored.contains("model_provider"));
        assert!(!restored.contains("[model_providers.codex-switch]"));
        assert!(restored.contains("[mcp_servers.demo]"));
    }

    #[test]
    fn stale_cleanup_file_removes_dead_switch_provider() {
        let temp = tempfile::tempdir().unwrap();
        let codex_dir = temp.path().join("codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        std::fs::write(
            codex_dir.join("config.toml"),
            r#"model_provider = "codex-switch"
model = "gpt-5.5"

[model_providers.codex-switch]
name = "codex-switch"
base_url = "http://127.0.0.1:8787/v1"
"#,
        )
        .unwrap();

        cleanup_stale_switch_config(Some(&display_path(&codex_dir))).unwrap();
        let cleaned = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();

        assert!(!cleaned.contains(r#"model_provider = "codex-switch""#));
        assert!(!cleaned.contains("[model_providers.codex-switch]"));
        assert!(cleaned.contains(r#"model = "gpt-5.5""#));
    }

    fn test_paths(root: &Path) -> AppPaths {
        let app_home = root.join("app");
        AppPaths {
            state_file: app_home.join("state").join("state.json"),
            backup_dir: app_home.join("backups"),
            log_dir: app_home.join("logs"),
            provider_icon_dir: app_home.join("icons").join("providers"),
            app_home,
        }
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
