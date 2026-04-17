//! url-md — Rust CLI 把任意 URL 转为 Markdown.
//!
//! Phase 1 MVP: 只实现 `url-md md <url>`. `batch` / `serve` / `login` 后续交付.

mod cmd;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "url-md",
    version,
    about = "Convert any URL to clean Markdown (Phase 1 MVP)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Fetch a single URL and output Markdown
    Md(cmd::md::Args),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Md(args) => cmd::md::run(args).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}
