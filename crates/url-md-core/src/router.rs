//! Registry: 注册 + 路由到 adapter.

use std::sync::Arc;

use url::Url;

use crate::adapter::Adapter;
use crate::error::PipelineError;

/// Adapter 注册表. 按注册顺序匹配,第一个 `matches()` 返回 true 的 adapter 生效.
///
/// 最后应注册一个兜底 generic adapter,避免 `AdapterNotFound`.
#[derive(Default, Clone)]
pub struct Registry {
    adapters: Vec<Arc<dyn Adapter>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { adapters: Vec::new() }
    }

    pub fn register<A: Adapter + 'static>(&mut self, adapter: A) {
        self.adapters.push(Arc::new(adapter));
    }

    pub fn route(&self, url: &Url) -> Result<Arc<dyn Adapter>, PipelineError> {
        for a in &self.adapters {
            if a.matches(url) {
                return Ok(a.clone());
            }
        }
        Err(PipelineError::AdapterNotFound {
            host: url.host_str().unwrap_or("").to_string(),
        })
    }

    pub fn len(&self) -> usize {
        self.adapters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::{Adapter, Article, ExtractError, MarkdownDoc, Strategy};
    use crate::fetcher::FetchedPage;
    use async_trait::async_trait;
    use std::collections::BTreeMap;

    struct FakeAdapter {
        name: &'static str,
        host: &'static str,
    }

    #[async_trait]
    impl Adapter for FakeAdapter {
        fn name(&self) -> &'static str {
            self.name
        }
        fn matches(&self, url: &Url) -> bool {
            url.host_str() == Some(self.host)
        }
        fn strategy(&self, _: &Url) -> Strategy {
            Strategy::Http
        }
        fn extract(&self, _: &FetchedPage) -> Result<Article, ExtractError> {
            unreachable!()
        }
        fn to_markdown(&self, _: &Article) -> MarkdownDoc {
            MarkdownDoc {
                frontmatter: BTreeMap::new(),
                body: String::new(),
            }
        }
    }

    struct CatchAllAdapter;
    #[async_trait]
    impl Adapter for CatchAllAdapter {
        fn name(&self) -> &'static str {
            "catchall"
        }
        fn matches(&self, _: &Url) -> bool {
            true
        }
        fn strategy(&self, _: &Url) -> Strategy {
            Strategy::Http
        }
        fn extract(&self, _: &FetchedPage) -> Result<Article, ExtractError> {
            unreachable!()
        }
        fn to_markdown(&self, _: &Article) -> MarkdownDoc {
            MarkdownDoc {
                frontmatter: BTreeMap::new(),
                body: String::new(),
            }
        }
    }

    #[test]
    fn empty_registry_returns_adapter_not_found() {
        let reg = Registry::new();
        let url = Url::parse("https://example.com").unwrap();
        match reg.route(&url) {
            Err(PipelineError::AdapterNotFound { .. }) => (),
            Err(e) => panic!("wrong error: {e:?}"),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn first_matching_adapter_wins() {
        let mut reg = Registry::new();
        reg.register(FakeAdapter { name: "a", host: "foo.com" });
        reg.register(FakeAdapter { name: "b", host: "bar.com" });
        let picked = reg.route(&Url::parse("https://foo.com").unwrap()).unwrap();
        assert_eq!(picked.name(), "a");
        let picked = reg.route(&Url::parse("https://bar.com").unwrap()).unwrap();
        assert_eq!(picked.name(), "b");
    }

    #[test]
    fn catchall_at_end_serves_as_fallback() {
        let mut reg = Registry::new();
        reg.register(FakeAdapter { name: "weixin", host: "weixin.com" });
        reg.register(CatchAllAdapter);
        let picked = reg
            .route(&Url::parse("https://randomsite.example").unwrap())
            .unwrap();
        assert_eq!(picked.name(), "catchall");
    }

    #[test]
    fn registration_order_precedence() {
        // 特化 adapter 必须在 catchall 之前,否则永远不会被命中
        let mut reg = Registry::new();
        reg.register(CatchAllAdapter);
        reg.register(FakeAdapter { name: "weixin", host: "weixin.com" });
        // 错误顺序下,weixin.com 会被 catchall 捕获
        let picked = reg.route(&Url::parse("https://weixin.com").unwrap()).unwrap();
        assert_eq!(picked.name(), "catchall", "catchall-first 会劫持所有 URL");
    }
}
