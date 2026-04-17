//! `url-md md <url>` — 单 URL 转 Markdown.

use std::path::PathBuf;
use std::time::Duration;

use clap::Args as ClapArgs;
use url_md_adapters::register_all;
use url_md_core::{fetch_and_convert, FetchOptions, PipelineError, Registry};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Target URL
    pub url: String,

    /// Write to file (default: stdout). If directory, auto-name `{date}-{host}-{slug}.md`.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Total timeout seconds (default: 45)
    #[arg(long, default_value_t = 45)]
    pub timeout: u64,

    /// Suppress stderr progress
    #[arg(long)]
    pub quiet: bool,
}

pub async fn run(args: Args) -> Result<(), u8> {
    let mut registry = Registry::new();
    register_all(&mut registry);

    let options = FetchOptions {
        timeout: Duration::from_secs(args.timeout),
        force_strategy: None,
        user_agent: None,
    };

    if !args.quiet {
        eprintln!("fetching {}...", args.url);
    }

    let doc = fetch_and_convert(&args.url, &options, &registry)
        .await
        .map_err(|e| {
            eprintln!("error: {e}");
            error_to_exit_code(&e)
        })?;

    let rendered = doc.render();

    match args.output {
        None => print!("{}", rendered),
        Some(path) => {
            let final_path = if path.is_dir() {
                path.join(auto_filename(&args.url, &doc))
            } else {
                path
            };
            std::fs::write(&final_path, rendered).map_err(|e| {
                eprintln!("error: write {}: {e}", final_path.display());
                30u8
            })?;
            if !args.quiet {
                eprintln!("wrote {}", final_path.display());
            }
        }
    }

    Ok(())
}

fn auto_filename(url: &str, doc: &url_md_core::adapter::MarkdownDoc) -> String {
    use time::OffsetDateTime;
    let date = OffsetDateTime::now_utc()
        .date()
        .to_string(); // YYYY-MM-DD
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".into())
        .replace('.', "_");
    let slug = doc
        .frontmatter
        .get("title")
        .and_then(|v| v.as_str())
        .map(slugify)
        .unwrap_or_else(|| short_hash(url));
    format!("{date}-{host}-{slug}.md")
}

fn slugify(s: &str) -> String {
    let s: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect();
    let s = s.trim_matches('-').to_lowercase();
    let s: String = s
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if s.is_empty() {
        "untitled".into()
    } else if s.len() > 60 {
        s[..60].to_string()
    } else {
        s
    }
}

fn short_hash(url: &str) -> String {
    // 简单非加密 hash,够做文件名去重
    let mut acc: u64 = 5381;
    for b in url.bytes() {
        acc = acc.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:x}", acc)
}

fn error_to_exit_code(e: &PipelineError) -> u8 {
    match e {
        PipelineError::InvalidUrl(_) => 30,
        PipelineError::AdapterNotFound { .. } => 20,
        PipelineError::Paywalled => 12,
        PipelineError::AuthRequired { .. } => 13,
        PipelineError::Fetch(_) => 10,
        PipelineError::ExtractFailed { .. } => 20,
        PipelineError::Internal(_) => 99,
    }
}

