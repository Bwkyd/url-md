//! Adapter trait + 共享类型.
//!
//! 具体 adapter 实现(weixin / generic / ...) 在 `url-md-adapters` crate.

use std::collections::BTreeMap;

use async_trait::async_trait;
use serde_yaml::Value as YamlValue;
use time::OffsetDateTime;
use url::Url;

use crate::fetcher::FetchedPage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// 只走 reqwest 快路
    Http,
    /// 只走 CDP (登录墙 / JS-heavy)
    Cdp,
    /// 先 reqwest,失败(含 content marker 缺失)回退 CDP
    HttpFirstCdpFallback,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub author: Option<String>,
    pub publish_time: Option<OffsetDateTime>,
    pub body_html: String, // 已抽取的正文 HTML(非完整页面)
    pub cover_url: Option<Url>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MarkdownDoc {
    pub frontmatter: BTreeMap<String, YamlValue>,
    pub body: String,
}

impl MarkdownDoc {
    /// 序列化为 "---\n<frontmatter yaml>\n---\n\n<body>\n"
    pub fn render(&self) -> String {
        let mut out = String::new();
        if !self.frontmatter.is_empty() {
            out.push_str("---\n");
            if let Ok(yaml) = serde_yaml::to_string(&self.frontmatter) {
                out.push_str(&yaml);
            }
            out.push_str("---\n\n");
        }
        out.push_str(&self.body);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, thiserror::Error)]
#[error("adapter extract failed: {0}")]
pub struct ExtractError(pub String);

#[async_trait]
pub trait Adapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn matches(&self, url: &Url) -> bool;
    fn strategy(&self, url: &Url) -> Strategy;
    /// adapter 用来判定 Http 快路是否抓到真实内容(还是拦截页).
    fn content_marker(&self) -> Option<&str> {
        None
    }
    fn extract(&self, page: &FetchedPage) -> Result<Article, ExtractError>;
    fn to_markdown(&self, article: &Article) -> MarkdownDoc;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value as YamlValue;

    #[test]
    fn render_with_frontmatter_starts_with_triple_dash() {
        let mut fm = BTreeMap::new();
        fm.insert("title".to_string(), YamlValue::String("Hello".into()));
        let doc = MarkdownDoc {
            frontmatter: fm,
            body: "# Body\n".to_string(),
        };
        let out = doc.render();
        assert!(out.starts_with("---\n"), "got: {out}");
        assert!(out.contains("title: Hello"));
        assert!(out.contains("# Body"));
    }

    #[test]
    fn render_without_frontmatter_skips_dashes() {
        let doc = MarkdownDoc {
            frontmatter: BTreeMap::new(),
            body: "plain body".to_string(),
        };
        let out = doc.render();
        assert!(!out.starts_with("---"), "got: {out}");
        assert!(out.starts_with("plain body"));
    }

    #[test]
    fn render_ends_with_newline() {
        let doc = MarkdownDoc {
            frontmatter: BTreeMap::new(),
            body: "no trailing newline".to_string(),
        };
        assert!(doc.render().ends_with('\n'));
    }
}
