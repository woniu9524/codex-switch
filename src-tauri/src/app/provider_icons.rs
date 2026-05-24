use super::store::{display_path, AppPaths};
use anyhow::{Context, Result};
use reqwest::header::CONTENT_TYPE;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

const MAX_ICON_BYTES: usize = 512 * 1024;
const ICON_EXTENSIONS: [&str; 5] = ["ico", "png", "svg", "jpg", "jpeg"];

struct IconDownload {
    bytes: Vec<u8>,
    extension: &'static str,
}

pub async fn resolve_provider_icon(
    paths: &AppPaths,
    provider_id: &str,
    homepage: Option<&str>,
    endpoint: &str,
) -> Option<String> {
    let domain = icon_domain(homepage, endpoint)?;
    if let Some(icon_path) = existing_icon_path(paths, provider_id) {
        return Some(display_path(&icon_path));
    }

    match download_icon(&domain).await {
        Ok(icon) => {
            let icon_path = paths
                .provider_icon_dir
                .join(format!("{provider_id}.{}", icon.extension));
            if write_icon(&icon_path, &icon.bytes).is_ok() {
                Some(display_path(&icon_path))
            } else {
                None
            }
        }
        Err(error) => {
            log::debug!("provider icon lookup failed for {domain}: {error}");
            None
        }
    }
}

fn existing_icon_path(paths: &AppPaths, provider_id: &str) -> Option<PathBuf> {
    ICON_EXTENSIONS
        .iter()
        .map(|extension| {
            paths
                .provider_icon_dir
                .join(format!("{provider_id}.{extension}"))
        })
        .find(|path| path.exists())
}

fn icon_domain(homepage: Option<&str>, endpoint: &str) -> Option<String> {
    homepage
        .and_then(host_from_url)
        .or_else(|| host_from_url(endpoint))
}

fn host_from_url(value: &str) -> Option<String> {
    Url::parse(value.trim())
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .filter(|host| !host.is_empty())
}

async fn download_icon(domain: &str) -> Result<IconDownload> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(4))
        .timeout(Duration::from_secs(8))
        .build()?;
    let candidates = [
        format!("https://{domain}/favicon.ico"),
        format!("https://www.google.com/s2/favicons?domain={domain}&sz=64"),
    ];

    for url in candidates {
        match try_icon_url(&client, &url).await {
            Ok(bytes) => return Ok(bytes),
            Err(error) => log::debug!("favicon candidate failed {url}: {error}"),
        }
    }
    anyhow::bail!("没有找到可用图标")
}

async fn try_icon_url(client: &reqwest::Client, url: &str) -> Result<IconDownload> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("请求图标失败: {url}"))?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("图标请求返回 {}", status.as_u16());
    }
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !content_type.is_empty()
        && !content_type.starts_with("image/")
        && !content_type.contains("octet-stream")
    {
        anyhow::bail!("图标响应不是图片");
    }

    let bytes = response.bytes().await?;
    if bytes.is_empty() || bytes.len() > MAX_ICON_BYTES {
        anyhow::bail!("图标大小不合适");
    }
    let bytes = bytes.to_vec();
    let extension = icon_extension(&content_type, &bytes)?;
    Ok(IconDownload { bytes, extension })
}

fn icon_extension(content_type: &str, bytes: &[u8]) -> Result<&'static str> {
    if content_type.contains("svg") {
        return Ok("svg");
    }
    if content_type.contains("png") {
        return Ok("png");
    }
    if content_type.contains("jpeg") || content_type.contains("jpg") {
        return Ok("jpg");
    }
    if content_type.contains("icon") || content_type.contains("ico") {
        return Ok("ico");
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok("png");
    }
    if bytes.starts_with(b"\0\0\x01\0") {
        return Ok("ico");
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Ok("jpg");
    }
    let text_start = String::from_utf8_lossy(&bytes[..bytes.len().min(128)]).to_ascii_lowercase();
    if text_start.contains("<svg") || text_start.contains("<?xml") {
        return Ok("svg");
    }
    anyhow::bail!("图标格式不支持")
}

fn write_icon(path: &PathBuf, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::icon_domain;

    #[test]
    fn icon_domain_prefers_homepage() {
        assert_eq!(
            icon_domain(
                Some("https://example.com/about"),
                "https://api.other.test/v1"
            ),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn icon_domain_falls_back_to_endpoint() {
        assert_eq!(
            icon_domain(None, "https://api.example.com/v1"),
            Some("api.example.com".to_string())
        );
    }
}
