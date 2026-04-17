//! 图片本地化: 把 Markdown 里所有外链图片下载到指定目录,改写为相对路径.
//!
//! 典型用法(CLI):
//!
//! ```bash
//! url-md md <URL> -o out/article.md --assets out/assets/
//! ```
//!
//! 抓完 Markdown 后,regex 扫 `![alt](URL)`,并发下载图片到 `assets/`,
//! 按 `img-0001.ext` 序号命名,同 URL 只下载一次,然后把 Markdown 里的
//! URL 替换为相对路径(相对于 Markdown 文件所在目录).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use tokio::sync::Semaphore;

/// 最大并发下载数. 微信 CDN(mmbiz.qpic.cn)对单 IP 有软限制,
/// 超过会触发 429 / 连接拒绝. 保守设为 8,可由环境变量 URL_MD_IMAGE_CONCURRENCY 覆盖.
fn max_concurrency() -> usize {
    std::env::var("URL_MD_IMAGE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0 && *n <= 64)
        .unwrap_or(8)
}

/// `![alt](URL)` 中的 URL 必须是 http/https,本地路径不处理.
static RE_IMG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!\[([^\]]*)\]\((https?://[^)\s]+)\)").unwrap());

#[derive(Debug, Clone, Copy, Default)]
pub struct DownloadStats {
    pub total: usize,
    pub downloaded: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// 从 Markdown 抽所有图片的 (alt, url),**按出现顺序,去重保序**.
pub fn extract_image_urls(markdown: &str) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for caps in RE_IMG.captures_iter(markdown) {
        let alt = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let url = caps.get(2).unwrap().as_str().to_string();
        if seen.insert(url.clone()) {
            out.push((alt, url));
        }
    }
    out
}

/// 从 URL 推断文件扩展名. 优先级:
///   1. 查询串 `wx_fmt=jpeg|png|gif|webp` (微信/腾讯 CDN 惯例)
///   2. 路径末尾 `.ext`
///   3. Content-Type(在 fetch 时另查)
///
/// 回退: `jpg`
pub fn guess_extension(url: &str) -> &'static str {
    // pass 1: wx_fmt=
    if let Some(pos) = url.to_lowercase().find("wx_fmt=") {
        let rest = &url[pos + "wx_fmt=".len()..];
        let ext = rest
            .split(|c: char| !c.is_ascii_alphanumeric())
            .next()
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "jpeg" | "jpg" => return "jpg",
            "png" => return "png",
            "gif" => return "gif",
            "webp" => return "webp",
            "svg" => return "svg",
            _ => {}
        }
    }
    // pass 2: 路径末尾的 .ext
    let path = url.split('?').next().unwrap_or(url);
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpeg" | "jpg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        "svg" => "svg",
        _ => "jpg",
    }
}

/// 按顺序生成本地文件名: `img-0001.jpg`, `img-0002.png`, ...
fn local_filename(index: usize, ext: &str) -> String {
    format!("img-{:04}.{ext}", index + 1)
}

/// 下载所有图片到 `assets_dir`,返回 (重写后的 Markdown, 统计).
///
/// `markdown_parent` 是最终输出 Markdown 文件所在目录,用于计算图片的相对路径.
/// 如果 assets_dir 在 markdown_parent 之内,引用为 `assets/img-0001.jpg`;
/// 否则使用绝对路径.
pub async fn localize_images(
    markdown: &str,
    assets_dir: &Path,
    markdown_parent: &Path,
) -> Result<(String, DownloadStats), std::io::Error> {
    let images = extract_image_urls(markdown);
    let mut stats = DownloadStats {
        total: images.len(),
        ..Default::default()
    };
    if images.is_empty() {
        return Ok((markdown.to_string(), stats));
    }

    std::fs::create_dir_all(assets_dir)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // 并发下载,semaphore 限制同时发起的连接数,避免触发 CDN 速率限制
    let sem = Arc::new(Semaphore::new(max_concurrency()));
    let mut handles = Vec::with_capacity(images.len());
    for (idx, (_, url)) in images.iter().enumerate() {
        let ext = guess_extension(url);
        let filename = local_filename(idx, ext);
        let dst = assets_dir.join(&filename);
        let url_owned = url.clone();
        let client = client.clone();
        let sem = sem.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.ok()?;
            Some(download_one(&client, &url_owned, &dst).await)
        }));
    }

    let mut url_to_local: HashMap<String, String> = HashMap::new();
    for (idx, handle) in handles.into_iter().enumerate() {
        let url = &images[idx].1;
        let ext = guess_extension(url);
        let filename = local_filename(idx, ext);
        match handle.await {
            Ok(Some(Ok(()))) => {
                stats.downloaded += 1;
                let rel = relative_ref(assets_dir, markdown_parent, &filename);
                url_to_local.insert(url.clone(), rel);
            }
            _ => {
                stats.failed += 1;
                // 失败保留原 URL,不改写
            }
        }
    }

    // 改写 Markdown
    let rewritten = RE_IMG
        .replace_all(markdown, |caps: &regex::Captures| {
            let alt = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let url = caps.get(2).unwrap().as_str();
            match url_to_local.get(url) {
                Some(local) => format!("![{alt}]({local})"),
                None => caps[0].to_string(),
            }
        })
        .into_owned();

    Ok((rewritten, stats))
}

async fn download_one(
    client: &reqwest::Client,
    url: &str,
    dst: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let bytes = resp.bytes().await?;
    std::fs::write(dst, &bytes)?;
    Ok(())
}

/// 计算 `assets_dir/filename` 相对于 `markdown_parent` 的引用路径.
fn relative_ref(assets_dir: &Path, markdown_parent: &Path, filename: &str) -> String {
    let assets_abs = canonical(assets_dir);
    let parent_abs = canonical(markdown_parent);
    if let Ok(rel) = assets_abs.strip_prefix(&parent_abs) {
        let rel_str = rel.to_string_lossy();
        if rel_str.is_empty() {
            return filename.to_string();
        }
        return format!("{rel_str}/{filename}");
    }
    // 不是后代目录 → 用绝对路径
    format!("{}/{filename}", assets_abs.to_string_lossy())
}

fn canonical(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_image() {
        let md = "![alt](https://example.com/x.jpg)";
        let out = extract_image_urls(md);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "alt");
        assert_eq!(out[0].1, "https://example.com/x.jpg");
    }

    #[test]
    fn extract_preserves_order_and_dedupes() {
        let md = "![a](https://e.com/1.jpg)\n![b](https://e.com/2.jpg)\n![c](https://e.com/1.jpg)";
        let out = extract_image_urls(md);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].1, "https://e.com/1.jpg");
        assert_eq!(out[1].1, "https://e.com/2.jpg");
    }

    #[test]
    fn skips_non_http_urls() {
        let md = "![a](./local.jpg) ![b](https://e.com/x.png)";
        let out = extract_image_urls(md);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].1, "https://e.com/x.png");
    }

    #[test]
    fn guess_ext_from_wx_fmt() {
        assert_eq!(guess_extension("https://e.com/x/0?wx_fmt=jpeg"), "jpg");
        assert_eq!(guess_extension("https://e.com/x/0?wx_fmt=png"), "png");
        assert_eq!(guess_extension("https://e.com/x/0?wx_fmt=gif&k=v"), "gif");
        assert_eq!(guess_extension("https://e.com/x/0?wx_fmt=webp"), "webp");
    }

    #[test]
    fn guess_ext_from_path() {
        assert_eq!(guess_extension("https://e.com/x.png"), "png");
        assert_eq!(guess_extension("https://e.com/path/y.JPEG"), "jpg");
        assert_eq!(guess_extension("https://e.com/x.gif?v=1"), "gif");
    }

    #[test]
    fn guess_ext_fallback_to_jpg() {
        assert_eq!(guess_extension("https://e.com/noext"), "jpg");
    }

    #[test]
    fn local_filename_uses_4_digit_padding() {
        assert_eq!(local_filename(0, "jpg"), "img-0001.jpg");
        assert_eq!(local_filename(9, "png"), "img-0010.png");
        assert_eq!(local_filename(999, "gif"), "img-1000.gif");
    }
}
