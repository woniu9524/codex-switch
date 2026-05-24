use super::models::{ImportPreview, DEFAULT_MODEL};
use anyhow::{anyhow, Result};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

enum ImportScheme {
    CodexSwitch,
    CcSwitch,
}

impl ImportScheme {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "codexswitch" => Ok(Self::CodexSwitch),
            "ccswitch" => Ok(Self::CcSwitch),
            _ => Err(anyhow!("scheme 只支持 codexswitch 或 ccswitch")),
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::CodexSwitch => "codexswitch",
            Self::CcSwitch => "ccswitch",
        }
    }

    fn source(&self) -> String {
        format!("{}://v1/import", self.as_str())
    }
}

pub struct ProviderImport {
    pub preview: ImportPreview,
    pub api_key: Option<String>,
}

pub fn parse_import_url(raw_url: &str) -> Result<ImportPreview> {
    Ok(parse_import(raw_url)?.preview)
}

pub fn parse_import(raw_url: &str) -> Result<ProviderImport> {
    let url = Url::parse(raw_url.trim()).map_err(|error| anyhow!("导入链接格式错误: {error}"))?;
    let scheme = ImportScheme::parse(url.scheme())?;

    let version = url.host_str().ok_or_else(|| anyhow!("缺少协议版本"))?;
    if version != "v1" {
        return Err(anyhow!("协议版本不支持: {version}"));
    }
    if url.path() != "/import" {
        return Err(anyhow!("path 必须是 /import"));
    }

    let mut params: HashMap<String, String> = url.query_pairs().into_owned().collect();
    require_provider_resource(&params)?;
    require_codex_app(&params)?;
    merge_config_fields(&mut params)?;

    let name = required(&params, "name")?;
    let endpoint = first_endpoint(&required(&params, "endpoint")?)?;
    validate_http_url(&endpoint, "endpoint")?;

    if let Some(homepage) = optional(&params, "homepage") {
        validate_http_url(&homepage, "homepage")?;
    }

    let api_key = api_key_param(&params);
    let model = optional(&params, "model").unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let enabled_hint = optional_bool(&params, "enabled")?.unwrap_or(false);

    Ok(ProviderImport {
        preview: ImportPreview {
            scheme: scheme.as_str().to_string(),
            source: scheme.source(),
            name,
            endpoint,
            api_key_masked: api_key.as_ref().map(|value| mask_key(value)),
            has_api_key: api_key.is_some(),
            model,
            homepage: optional(&params, "homepage"),
            notes: optional(&params, "notes"),
            icon: optional(&params, "icon"),
            enabled_hint,
        },
        api_key,
    })
}

fn require_provider_resource(params: &HashMap<String, String>) -> Result<()> {
    if optional(params, "resource").as_deref() == Some("provider") {
        Ok(())
    } else {
        Err(anyhow!("resource 只支持 provider"))
    }
}

fn require_codex_app(params: &HashMap<String, String>) -> Result<()> {
    match optional(params, "app") {
        Some(app) if app != "codex" => Err(anyhow!("app 必须是 codex")),
        _ => Ok(()),
    }
}

fn required(params: &HashMap<String, String>, name: &str) -> Result<String> {
    optional(params, name).ok_or_else(|| anyhow!("{name} 为必填字段"))
}

fn optional(params: &HashMap<String, String>, name: &str) -> Option<String> {
    params
        .get(name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn api_key_param(params: &HashMap<String, String>) -> Option<String> {
    optional(params, "apiKey").or_else(|| optional(params, "api_key"))
}

fn optional_bool(params: &HashMap<String, String>, name: &str) -> Result<Option<bool>> {
    let Some(value) = optional(params, name) else {
        return Ok(None);
    };
    match value.as_str() {
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        _ => Err(anyhow!("{name} 只支持 true 或 false")),
    }
}

fn first_endpoint(value: &str) -> Result<String> {
    value
        .split(',')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("endpoint 不能为空"))
}

fn merge_config_fields(params: &mut HashMap<String, String>) -> Result<()> {
    let Some(config_value) = optional(params, "config") else {
        return Ok(());
    };
    let format = optional(params, "configFormat")
        .unwrap_or_else(|| "json".to_string())
        .to_ascii_lowercase();
    let decoded = decode_config_param(&config_value);
    let decoded_text =
        String::from_utf8(decoded).map_err(|error| anyhow!("config 不是 UTF-8: {error}"))?;

    match format.as_str() {
        "json" => merge_json_config(params, &decoded_text),
        "toml" => merge_toml_config(params, &decoded_text),
        other => Err(anyhow!("configFormat 只支持 json 或 toml，当前是 {other}")),
    }
}

fn merge_json_config(params: &mut HashMap<String, String>, text: &str) -> Result<()> {
    let value = serde_json::from_str::<Value>(text)
        .map_err(|error| anyhow!("config JSON 解析失败: {error}"))?;
    insert_api_key_if_missing(params, extract_json_api_key(&value));
    if let Some(config_text) = value.get("config").and_then(|value| value.as_str()) {
        merge_toml_config(params, config_text)?;
    }
    insert_if_missing(params, "endpoint", || {
        value
            .get("baseUrl")
            .or_else(|| value.get("base_url"))
            .or_else(|| value.get("endpoint"))
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
    });
    Ok(())
}

fn merge_toml_config(params: &mut HashMap<String, String>, text: &str) -> Result<()> {
    let value = toml::from_str::<toml::Value>(text)
        .map_err(|error| anyhow!("config TOML 解析失败: {error}"))?;
    insert_api_key_if_missing(params, extract_toml_api_key(&value));
    insert_if_missing(params, "endpoint", || extract_codex_base_url(&value));
    insert_if_missing(params, "model", || {
        value
            .get("model")
            .and_then(|value| value.as_str())
            .map(ToString::to_string)
    });
    Ok(())
}

fn insert_if_missing(
    params: &mut HashMap<String, String>,
    key: &str,
    value: impl FnOnce() -> Option<String>,
) {
    if optional(params, key).is_some() {
        return;
    }
    if let Some(value) = value().filter(|value| !value.trim().is_empty()) {
        params.insert(key.to_string(), value);
    }
}

fn insert_api_key_if_missing(params: &mut HashMap<String, String>, api_key: Option<String>) {
    if api_key_param(params).is_some() {
        return;
    }
    if let Some(api_key) = api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        params.insert("apiKey".to_string(), api_key);
    }
}

fn extract_codex_base_url(value: &toml::Value) -> Option<String> {
    let active = value.get("model_provider").and_then(|value| value.as_str());
    if let Some(active) = active {
        if let Some(base_url) = value
            .get("model_providers")
            .and_then(|providers| providers.get(active))
            .and_then(|provider| provider.get("base_url"))
            .and_then(|value| value.as_str())
        {
            return Some(base_url.to_string());
        }
    }

    value
        .get("model_providers")
        .and_then(|providers| providers.as_table())
        .and_then(|providers| {
            providers.values().find_map(|provider| {
                provider
                    .get("base_url")
                    .and_then(|value| value.as_str())
                    .map(ToString::to_string)
            })
        })
}

fn decode_config_param(value: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| STANDARD.decode(value))
        .unwrap_or_else(|_| value.as_bytes().to_vec())
}

fn extract_json_api_key(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => object.iter().find_map(|(key, value)| {
            direct_json_api_key_value(key, value).or_else(|| extract_json_api_key(value))
        }),
        Value::Array(values) => values.iter().find_map(extract_json_api_key),
        _ => None,
    }
}

fn extract_toml_api_key(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::Table(table) => table.iter().find_map(|(key, value)| {
            direct_toml_api_key_value(key, value).or_else(|| extract_toml_api_key(value))
        }),
        toml::Value::Array(values) => values.iter().find_map(extract_toml_api_key),
        _ => None,
    }
}

fn direct_json_api_key_value(key: &str, value: &Value) -> Option<String> {
    direct_key_candidate(key, value.as_str()?)
}

fn direct_toml_api_key_value(key: &str, value: &toml::Value) -> Option<String> {
    direct_key_candidate(key, value.as_str()?)
}

fn direct_key_candidate(key: &str, value: &str) -> Option<String> {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    if normalized == "authorization" {
        return parse_bearer_token(value);
    }
    if normalized == "apikey" || normalized == "api_key" || normalized.ends_with("_api_key") {
        return Some(value.to_string());
    }
    None
}

fn parse_bearer_token(value: &str) -> Option<String> {
    value
        .trim()
        .strip_prefix("Bearer ")
        .or_else(|| value.trim().strip_prefix("bearer "))
        .map(ToString::to_string)
}

fn validate_http_url(value: &str, field: &str) -> Result<()> {
    let url = Url::parse(value).map_err(|error| anyhow!("{field} 不是有效 URL: {error}"))?;
    if !matches!(url.scheme(), "https" | "http") || url.host_str().is_none() {
        return Err(anyhow!("{field} 必须是 http(s) URL"));
    }
    Ok(())
}

fn mask_key(value: &str) -> String {
    if value.len() <= 8 {
        return "****".to_string();
    }
    format!("{}********{}", &value[..3], &value[value.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codexswitch_provider() {
        let url = "codexswitch://v1/import?resource=provider&name=DeepSeek&endpoint=https%3A%2F%2Fapi.deepseek.com%2Fv1&apiKey=sk-test-123456&model=deepseek-chat&enabled=true";
        let preview = parse_import_url(url).unwrap();

        assert_eq!(preview.scheme, "codexswitch");
        assert_eq!(preview.source, "codexswitch://v1/import");
        assert_eq!(preview.name, "DeepSeek");
        assert!(preview.has_api_key);
        assert!(preview.enabled_hint);
        assert_eq!(
            parse_import(url).unwrap().api_key,
            Some("sk-test-123456".to_string())
        );
    }

    #[test]
    fn parses_compatible_ccswitch_codex_provider() {
        let url = "ccswitch://v1/import?resource=provider&app=codex&name=Moonshot&endpoint=https%3A%2F%2Fapi.moonshot.cn%2Fv1";
        let preview = parse_import_url(url).unwrap();

        assert_eq!(preview.scheme, "ccswitch");
        assert_eq!(preview.source, "ccswitch://v1/import");
        assert_eq!(preview.name, "Moonshot");
        assert_eq!(preview.model, DEFAULT_MODEL);
    }

    #[test]
    fn rejects_non_codex_ccswitch_provider() {
        let url = "ccswitch://v1/import?resource=provider&app=claude&name=Bad&endpoint=https%3A%2F%2Fapi.example.com%2Fv1";

        assert!(parse_import_url(url)
            .unwrap_err()
            .to_string()
            .contains("app 必须是 codex"));
    }

    #[test]
    fn rejects_missing_name() {
        let url =
            "codexswitch://v1/import?resource=provider&endpoint=https%3A%2F%2Fapi.example.com%2Fv1";

        assert!(parse_import_url(url)
            .unwrap_err()
            .to_string()
            .contains("name"));
    }

    #[test]
    fn rejects_invalid_enabled_hint() {
        let url = "codexswitch://v1/import?resource=provider&name=Bad&endpoint=https%3A%2F%2Fapi.example.com%2Fv1&enabled=1";

        assert!(parse_import_url(url)
            .unwrap_err()
            .to_string()
            .contains("enabled"));
    }
}
