//! Fetcher 抽象 + HttpFetcher 快路实现.
//!
//! CdpFetcher 是 scaffold(spec/dev/cdp-fetcher.spec.md v0.1.0-scaffold):
//! 接口已锁,实际实现等 fork branch `Bwkyd/agent-browser:feat/lib-target` 稳定 + CfT 安装链路打通后落地.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use time::OffsetDateTime;
use url::Url;

use crate::error::FetchError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetcherKind {
    Http,
    Cdp,
}

#[derive(Debug, Clone)]
pub struct FetchOpts {
    pub timeout: Duration,
    pub user_agent: Option<String>,
    pub headers: HashMap<String, String>,
}

impl Default for FetchOpts {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: None,
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub html: String,
    pub final_url: Url,
    pub status: u16,
    pub fetched_at: OffsetDateTime,
}

#[async_trait]
pub trait Fetcher: Send + Sync {
    async fn fetch(&self, url: &Url, opts: &FetchOpts) -> Result<FetchedPage, FetchError>;
    fn kind(&self) -> FetcherKind;
}

// -----------------------------------------------------------------------------
// HttpFetcher (reqwest 快路)
// -----------------------------------------------------------------------------

const DEFAULT_UA: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct HttpFetcher {
    client: reqwest::Client,
}

impl HttpFetcher {
    pub fn new() -> Result<Self, FetchError> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .build()
            .map_err(|e| FetchError::Internal(e.to_string()))?;
        Ok(Self { client })
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new().expect("build http client")
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(&self, url: &Url, opts: &FetchOpts) -> Result<FetchedPage, FetchError> {
        let ua = opts.user_agent.as_deref().unwrap_or(DEFAULT_UA);
        let mut req = self
            .client
            .get(url.clone())
            .timeout(opts.timeout)
            .header(reqwest::header::USER_AGENT, ua);
        for (k, v) in &opts.headers {
            req = req.header(k, v);
        }
        let resp = req.send().await.map_err(|e| {
            if e.is_timeout() {
                FetchError::Timeout { fetcher: FetcherKind::Http }
            } else {
                FetchError::Internal(e.to_string())
            }
        })?;
        let status = resp.status();
        if !status.is_success() {
            return Err(FetchError::HttpStatus { code: status.as_u16() });
        }
        let final_url = resp.url().clone();
        let html = resp
            .text()
            .await
            .map_err(|e| FetchError::Internal(format!("body: {e}")))?;
        Ok(FetchedPage {
            html,
            final_url,
            status: status.as_u16(),
            fetched_at: OffsetDateTime::now_utc(),
        })
    }

    fn kind(&self) -> FetcherKind {
        FetcherKind::Http
    }
}

// -----------------------------------------------------------------------------
// CdpFetcher (Phase 2 scaffold · 真实现见 spec/dev/cdp-fetcher.spec.md)
// -----------------------------------------------------------------------------

/// CDP 回退 fetcher(占位).
///
/// 真实现需消费 `Bwkyd/agent-browser` fork 的 lib target,本轮 session 未启用.
/// 当前调用 `fetch()` 总是返回 `FetchError::CdpUnavailable` 让 pipeline 报错退出.
pub struct CdpFetcher {
    _placeholder: (),
}

impl CdpFetcher {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for CdpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetcher for CdpFetcher {
    async fn fetch(&self, _url: &Url, _opts: &FetchOpts) -> Result<FetchedPage, FetchError> {
        Err(FetchError::CdpUnavailable {
            reason: "CdpFetcher scaffold · real impl deferred (see spec/dev/cdp-fetcher.spec.md)"
                .to_string(),
        })
    }

    fn kind(&self) -> FetcherKind {
        FetcherKind::Cdp
    }
}
