//! url-md-core — Phase 1 MVP
//!
//! 核心抽象:
//! - [`Fetcher`] trait + [`HttpFetcher`] (reqwest 快路)
//! - [`Adapter`] trait (站点特化)
//! - [`router`] — URL → Adapter 路由
//! - [`parser`] — DOM → Markdown 保真转换
//! - [`pipeline::fetch_and_convert`] — 三种 runtime 共享的唯一入口
//! - [`PipelineError`] — 统一错误枚举
//!
//! 此 crate 为 **无状态 core**,不包含 CLI / HTTP / MCP 入口实现。

pub mod adapter;
pub mod error;
pub mod fetcher;
pub mod parser;
pub mod pipeline;
pub mod router;

pub use adapter::{Adapter, Article, MarkdownDoc, Strategy};
pub use error::{FetchError, PipelineError};
pub use fetcher::{FetchedPage, Fetcher, FetcherKind, HttpFetcher};
pub use pipeline::{fetch_and_convert, FetchOptions};
pub use router::Registry;
