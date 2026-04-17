//! `url-md md <url>` — 单 URL 转 Markdown.

use std::path::PathBuf;
use std::time::Duration;

use clap::Args as ClapArgs;
use url_md_adapters::register_all;
use url_md_core::downloader::localize_images;
use url_md_core::{fetch_and_convert, FetchOptions, PipelineError, Registry};

/// 三档日志: Quiet(无输出) / Default(关键节点) / Verbose(每阶段).
#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Quiet,
    Default,
    Verbose,
}

impl LogLevel {
    fn from_args(verbose: bool, quiet: bool) -> Self {
        if verbose {
            LogLevel::Verbose
        } else if quiet {
            LogLevel::Quiet
        } else {
            LogLevel::Default
        }
    }

    fn default(&self, msg: &str) {
        match self {
            LogLevel::Default | LogLevel::Verbose => eprintln!("{msg}"),
            LogLevel::Quiet => {}
        }
    }

    fn verbose(&self, msg: &str) {
        if matches!(self, LogLevel::Verbose) {
            eprintln!("{msg}");
        }
    }
}

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Target URL
    pub url: String,

    /// Write to file (default: stdout). If directory, auto-name `{date}-{host}-{slug}.md`.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Custom image directory (default: <output_dir>/assets/ when -o is set).
    #[arg(long, value_name = "DIR")]
    pub assets: Option<PathBuf>,

    /// Disable image download (default: on when -o is set)
    #[arg(long)]
    pub no_assets: bool,

    /// Total timeout seconds (default: 45)
    #[arg(long, default_value_t = 45)]
    pub timeout: u64,

    /// Suppress stderr progress
    #[arg(long)]
    pub quiet: bool,

    /// Verbose stage-by-stage progress on stderr (overrides --quiet if both set)
    #[arg(short, long)]
    pub verbose: bool,
}

pub async fn run(args: Args) -> Result<(), u8> {
    let mut registry = Registry::new();
    register_all(&mut registry);

    let options = FetchOptions {
        timeout: Duration::from_secs(args.timeout),
        force_strategy: None,
        user_agent: None,
    };

    // 三档日志: --verbose > default > --quiet
    let log = LogLevel::from_args(args.verbose, args.quiet);

    log.default(&format!("fetching {}...", args.url));
    log.verbose("  routing adapter · fetching HTML · extracting · rendering markdown");

    let doc = fetch_and_convert(&args.url, &options, &registry)
        .await
        .map_err(|e| {
            eprintln!("error: {e}");
            error_to_exit_code(&e)
        })?;

    log.verbose(&format!(
        "  adapter={} · title={}",
        doc.frontmatter
            .get("source_adapter")
            .and_then(|v| v.as_str())
            .unwrap_or("?"),
        doc.frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("(untitled)")
    ));

    let rendered = doc.render();

    match args.output {
        None => {
            if (args.assets.is_some() || args.no_assets) && !args.quiet {
                eprintln!("warning: --assets/--no-assets ignored without -o/--output");
            }
            print!("{}", rendered);
        }
        Some(path) => {
            let final_path = if path.is_dir() {
                path.join(auto_filename(&args.url, &doc))
            } else {
                path
            };
            let markdown_parent = final_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            std::fs::create_dir_all(&markdown_parent).map_err(|e| {
                eprintln!("error: mkdir {}: {e}", markdown_parent.display());
                30u8
            })?;

            // 默认下图:有 -o 就自动在 sibling 建 assets/ 下图;--no-assets 关闭;--assets 显式指定
            let resolved_assets: Option<PathBuf> = if args.no_assets {
                None
            } else {
                Some(
                    args.assets
                        .clone()
                        .unwrap_or_else(|| markdown_parent.join("assets")),
                )
            };

            let final_markdown = if let Some(assets_dir) = &resolved_assets {
                log.default(&format!("downloading images to {}...", assets_dir.display()));
                match localize_images(&rendered, assets_dir, &markdown_parent).await {
                    Ok((md, stats)) => {
                        log.default(&format!(
                            "images: {}/{} downloaded ({} failed)",
                            stats.downloaded, stats.total, stats.failed
                        ));
                        md
                    }
                    Err(e) => {
                        eprintln!("warning: image localize failed: {e} (keeping remote URLs)");
                        rendered
                    }
                }
            } else {
                rendered
            };

            std::fs::write(&final_path, final_markdown).map_err(|e| {
                eprintln!("error: write {}: {e}", final_path.display());
                30u8
            })?;
            log.default(&format!("wrote {}", final_path.display()));
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

