//! `url-md serve --mcp` — MCP server (stdio transport).
//!
//! 实现 [`spec/dev/serve-mcp.spec.md`](../../../../spec/dev/serve-mcp.spec.md):
//! - transport: stdio (JSON-RPC 2.0)
//! - 1 个 tool: `md(url, timeout_seconds?)`
//! - 调 `pipeline::fetch_and_convert()` 单入口,保证 real §3 字节级一致

use std::sync::Arc;
use std::time::Duration;

use clap::Args as ClapArgs;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url_md_adapters::register_all;
use url_md_core::{FetchOptions, Registry, fetch_and_convert};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Run as MCP server over stdio (default and only mode in v0.2.0)
    #[arg(long, default_value_t = true)]
    pub mcp: bool,

    /// Default fetch timeout (seconds, can be overridden per-call via tool args)
    #[arg(long, default_value_t = 45)]
    pub timeout: u64,
}

/// Tool input schema for `md`.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MdRequest {
    /// Target URL (must start with http:// or https://)
    pub url: String,
    /// Override default timeout in seconds (1-300)
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// MCP server holding shared adapter registry.
#[derive(Clone)]
pub struct UrlMdServer {
    registry: Arc<Registry>,
    default_timeout: Duration,
    tool_router: ToolRouter<Self>,
}

#[tool_router(router = tool_router)]
impl UrlMdServer {
    pub fn new(default_timeout: Duration) -> Self {
        let mut registry = Registry::new();
        register_all(&mut registry);
        Self {
            registry: Arc::new(registry),
            default_timeout,
            tool_router: Self::tool_router(),
        }
    }

    /// Convert a URL to clean Markdown with YAML frontmatter.
    /// Output is byte-identical to `url-md md <url>` (real §3).
    #[tool(
        name = "md",
        description = "Convert a URL to clean Markdown with YAML frontmatter. Same output as `url-md md <url>` CLI."
    )]
    pub async fn md(&self, params: Parameters<MdRequest>) -> Result<String, McpError> {
        let req = params.0;
        let timeout = req
            .timeout_seconds
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);
        let options = FetchOptions {
            timeout,
            force_strategy: None,
            user_agent: None,
        };
        let doc = fetch_and_convert(&req.url, &options, &self.registry)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(doc.render())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for UrlMdServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
            .with_server_info(Implementation::new("url-md", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "url-md MCP server. Tool `md(url)` converts any URL to clean Markdown.",
            )
    }
}

pub async fn run(args: Args) -> Result<(), u8> {
    if !args.mcp {
        eprintln!("error: only --mcp transport is supported in v0.2.0");
        return Err(20);
    }
    let server = UrlMdServer::new(Duration::from_secs(args.timeout));
    let service = server.serve(stdio()).await.map_err(|e| {
        eprintln!("error: serve start failed: {e}");
        99u8
    })?;
    service.waiting().await.map_err(|e| {
        eprintln!("error: serve wait failed: {e}");
        99u8
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_constructs_with_registry() {
        let server = UrlMdServer::new(Duration::from_secs(30));
        assert_eq!(server.default_timeout, Duration::from_secs(30));
    }

    #[test]
    fn md_request_deserializes_url_only() {
        let req: MdRequest =
            rmcp::serde_json::from_str(r#"{"url":"https://example.com"}"#).unwrap();
        assert_eq!(req.url, "https://example.com");
        assert_eq!(req.timeout_seconds, None);
    }

    #[test]
    fn md_request_deserializes_with_timeout() {
        let req: MdRequest = rmcp::serde_json::from_str(
            r#"{"url":"https://example.com","timeout_seconds":60}"#,
        )
        .unwrap();
        assert_eq!(req.timeout_seconds, Some(60));
    }
}
