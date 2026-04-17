//! url-md — Rust CLI 把任意 URL 转为 Markdown.
//!
//! 用法:
//!   url-md md <URL>       # 完整子命令
//!   url-md <URL>          # 别名(等价于 md)

mod cmd;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "url-md",
    version,
    about = "Convert any URL to clean Markdown · MCP-native"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch a single URL and output Markdown
    Md(cmd::md::Args),
    /// Run as MCP server over stdio (for Claude Code / Cursor / Cline)
    Serve(cmd::serve::Args),
}

/// 如果第一个非 flag 参数是 URL 且不是已知子命令,自动前置 `md` 子命令.
/// 即 `url-md https://...` ≡ `url-md md https://...`;
/// `url-md --verbose https://...` ≡ `url-md md --verbose https://...`.
fn desugar_argv() -> Vec<String> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return args;
    }
    let first_is_subcommand = is_known_subcommand(&args[1]);
    let has_url_positional = args[1..]
        .iter()
        .any(|a| !a.starts_with('-') && looks_like_url(a));
    if !first_is_subcommand && has_url_positional {
        // 前置 "md" 到所有 user args 之前,确保后续 --flag 归属 md 子命令
        args.insert(1, "md".to_string());
    }
    args
}

fn is_known_subcommand(s: &str) -> bool {
    matches!(
        s,
        "md" | "serve" | "help" | "-h" | "--help" | "-V" | "--version"
    )
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse_from(desugar_argv());
    let result = match cli.command {
        Command::Md(args) => cmd::md::run(args).await,
        Command::Serve(args) => cmd::serve::run(args).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_like_url_accepts_http() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("http://a.b"));
    }

    #[test]
    fn looks_like_url_rejects_others() {
        assert!(!looks_like_url("md"));
        assert!(!looks_like_url("/path/file"));
        assert!(!looks_like_url("example.com")); // 保守: 不带 scheme 不算
    }
}
