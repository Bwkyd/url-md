//! pipeline::fetch_and_convert — 三种 runtime 共享的唯一核心入口.
//!
//! 所有子命令(md / batch / serve --http / serve --mcp) 都必须经此函数,
//! 保证相同 URL + options 的输出字节级一致(real.md #3).

use std::time::Duration;

use url::Url;

use crate::adapter::{MarkdownDoc, Strategy};
use crate::error::{FetchError, PipelineError};
use crate::fetcher::{FetchOpts, Fetcher, FetcherKind, HttpFetcher};
use crate::router::Registry;

#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub timeout: Duration,
    pub force_strategy: Option<Strategy>,
    pub user_agent: Option<String>,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(45),
            force_strategy: None,
            user_agent: None,
        }
    }
}

/// 核心入口: 抓取 URL,路由到 Adapter,抽取 Article,渲染为 Markdown.
///
/// 这是整个 crate 的**唯一**对外抓取函数. CLI / HTTP server / MCP server
/// 必须调此函数,禁止旁路直接调 Fetcher/Adapter.
///
/// # 保证
/// - 相同 `(url, options, registry)` 输出字节级一致 (real.md #3).
/// - 错误以 [`PipelineError`] 统一枚举.
///
/// # Phase 1 限制
/// - 只实现 Http 快路 (Strategy::Http 与 HttpFirstCdpFallback 的快路部分).
/// - `Strategy::Cdp` 与快路失败的 CDP 回退在后续 phase 交付.
pub async fn fetch_and_convert(
    url: &str,
    options: &FetchOptions,
    registry: &Registry,
) -> Result<MarkdownDoc, PipelineError> {
    let parsed = Url::parse(url)?;
    let adapter = registry.route(&parsed)?;

    let strategy = options
        .force_strategy
        .unwrap_or_else(|| adapter.strategy(&parsed));

    let page = match strategy {
        Strategy::Http | Strategy::HttpFirstCdpFallback => {
            let fetcher = HttpFetcher::new().map_err(PipelineError::Fetch)?;
            let opts = FetchOpts {
                timeout: options.timeout,
                user_agent: options.user_agent.clone(),
                ..Default::default()
            };
            let page = fetcher.fetch(&parsed, &opts).await?;
            // 内容完整性检测
            if let Some(marker) = adapter.content_marker() {
                if !page.html.contains(marker) {
                    // Phase 1: 不做 CDP 回退,直接报 blocked
                    return Err(PipelineError::Fetch(FetchError::Blocked {
                        fetcher: FetcherKind::Http,
                        reason: format!("content marker `{marker}` missing"),
                    }));
                }
            }
            page
        }
        Strategy::Cdp => {
            return Err(PipelineError::Internal(
                "CDP fetcher not implemented in Phase 1".to_string(),
            ));
        }
    };

    let article = adapter
        .extract(&page)
        .map_err(|e| PipelineError::ExtractFailed {
            adapter: adapter.name().to_string(),
            reason: e.0,
        })?;
    Ok(adapter.to_markdown(&article))
}
