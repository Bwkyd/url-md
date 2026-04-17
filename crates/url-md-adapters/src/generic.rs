//! GenericAdapter — 兜底,readability-style 抽取.
//!
//! 覆盖:任何没有特化 adapter 的 URL.

use std::collections::BTreeMap;

use scraper::{Html, Selector};
use serde_yaml::Value as YamlValue;
use time::OffsetDateTime;
use url::Url;

use url_md_core::{
    adapter::{Adapter, Article, ExtractError, MarkdownDoc, Strategy},
    fetcher::FetchedPage,
    parser::html_to_markdown,
};

pub struct GenericAdapter;

impl GenericAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GenericAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for GenericAdapter {
    fn name(&self) -> &'static str {
        "generic"
    }

    fn matches(&self, _url: &Url) -> bool {
        true // 兜底: 任何 URL 都能命中
    }

    fn strategy(&self, _url: &Url) -> Strategy {
        Strategy::Http
    }

    fn extract(&self, page: &FetchedPage) -> Result<Article, ExtractError> {
        let doc = Html::parse_document(&page.html);

        // title: 优先 <meta property="og:title">, 退到 <title>
        let title = pick_meta(&doc, "og:title")
            .or_else(|| {
                let sel = Selector::parse("title").unwrap();
                doc.select(&sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
            .unwrap_or_else(|| page.final_url.to_string());

        let author = pick_meta(&doc, "article:author").or_else(|| pick_meta(&doc, "author"));

        let cover_url = pick_meta(&doc, "og:image")
            .and_then(|s| Url::parse(&s).ok());

        // body: <article> 优先, 退到 <main>, 退到 <body>
        let body_html = pick_first_html(&doc, &["article", "main", "body"])
            .unwrap_or_else(|| page.html.clone());

        Ok(Article {
            title,
            author,
            publish_time: None,
            body_html,
            cover_url,
            metadata: BTreeMap::new(),
        })
    }

    fn to_markdown(&self, article: &Article) -> MarkdownDoc {
        let mut fm: BTreeMap<String, YamlValue> = BTreeMap::new();
        fm.insert("title".into(), YamlValue::String(article.title.clone()));
        if let Some(a) = &article.author {
            fm.insert("author".into(), YamlValue::String(a.clone()));
        }
        if let Some(u) = &article.cover_url {
            fm.insert("cover_url".into(), YamlValue::String(u.to_string()));
        }
        fm.insert("source_adapter".into(), YamlValue::String("generic".into()));
        fm.insert(
            "fetched_at".into(),
            YamlValue::String(
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            ),
        );

        let body = html_to_markdown(&article.body_html).trim().to_string();
        MarkdownDoc { frontmatter: fm, body }
    }
}

fn pick_meta(doc: &Html, property: &str) -> Option<String> {
    let selector = format!(
        r#"meta[property="{p}"], meta[name="{p}"]"#,
        p = property
    );
    let sel = Selector::parse(&selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn pick_first_html(doc: &Html, tags: &[&str]) -> Option<String> {
    for tag in tags {
        let sel = Selector::parse(tag).ok()?;
        if let Some(el) = doc.select(&sel).next() {
            return Some(el.inner_html());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use url_md_core::adapter::Adapter;

    #[test]
    fn matches_any_url() {
        let a = GenericAdapter::new();
        assert!(a.matches(&Url::parse("https://anything.example").unwrap()));
        assert!(a.matches(&Url::parse("http://a.b/path?q=1").unwrap()));
    }

    #[test]
    fn strategy_is_plain_http() {
        let a = GenericAdapter::new();
        let url = Url::parse("https://x.com").unwrap();
        assert!(matches!(
            a.strategy(&url),
            url_md_core::adapter::Strategy::Http
        ));
    }

    #[test]
    fn content_marker_is_none() {
        let a = GenericAdapter::new();
        assert!(a.content_marker().is_none());
    }
}
