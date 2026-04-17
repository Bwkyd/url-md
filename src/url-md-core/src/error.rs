//! PipelineError — 统一错误模型,三种 runtime 返回字段对齐.

use thiserror::Error;

use crate::fetcher::FetcherKind;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network timeout via {fetcher:?}")]
    Timeout { fetcher: FetcherKind },

    #[error("http status {code}")]
    HttpStatus { code: u16 },

    #[error("blocked by anti-bot via {fetcher:?}: {reason}")]
    Blocked { fetcher: FetcherKind, reason: String },

    #[error("content missing marker: {marker}")]
    ContentMissing { marker: String },

    #[error("CDP unavailable: {reason}")]
    CdpUnavailable { reason: String },

    #[error("fetch internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("no adapter matches host `{host}` (and generic fallback unavailable)")]
    AdapterNotFound { host: String },

    #[error("paywalled — not attempting to bypass (per real.md #4)")]
    Paywalled,

    #[error("auth required for {site}: {hint}")]
    AuthRequired { site: String, hint: String },

    #[error(transparent)]
    Fetch(#[from] FetchError),

    #[error("extract failed in adapter `{adapter}`: {reason}")]
    ExtractFailed { adapter: String, reason: String },

    #[error("internal: {0}")]
    Internal(String),
}
