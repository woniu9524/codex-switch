use anyhow::{anyhow, Result};

const SERVICE: &str = "codex-switch";

pub fn set_provider_key(provider_id: &str, api_key: &str) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, provider_id)
        .map_err(|error| anyhow!("系统凭据存储不可用: {error}"))?;
    entry
        .set_password(api_key)
        .map_err(|error| anyhow!("保存 API Key 到系统凭据失败: {error}"))?;
    Ok(format!("keyring://{SERVICE}/{provider_id}"))
}

pub fn get_provider_key(provider_id: &str) -> Result<String> {
    let entry = keyring::Entry::new(SERVICE, provider_id)
        .map_err(|error| anyhow!("系统凭据存储不可用: {error}"))?;
    entry
        .get_password()
        .map_err(|error| anyhow!("读取供应商 API Key 失败: {error}"))
}

pub fn provider_key_available(provider_id: &str) -> bool {
    get_provider_key(provider_id)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

pub fn delete_provider_key(provider_id: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, provider_id) {
        let _ = entry.delete_credential();
    }
}
