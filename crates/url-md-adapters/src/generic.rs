//! GenericAdapter — 兜底,readability-style 抽取.
//!
//! 覆盖:任何没有特化 adapter 的 URL.
//!
//! 抽取策略(按顺序):
//! 1. 多个 `<article>`(列表式首页) → 合并所有 article
//! 2. 单个 `<article>`(文章页) → 直接用
//! 3. `<main>`
//! 4. `<body>`
//! 5. 原始 HTML

use std::collections::BTreeMap;

use scraper::{Html, Selector};
use serde_yaml::Value as YamlValue;
use time::OffsetDateTime;
use url::Url;

use url_md_core::{
    adapter::{Adapter, Article, ExtractError, MarkdownDoc, Strategy},
    fetcher::FetchedPage,
    parser::html_to_markdown,
    text::{count_words, reading_time_minutes},
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
        true
    }

    fn strategy(&self, _url: &Url) -> Strategy {
        Strategy::Http
    }

    fn extract(&self, page: &FetchedPage) -> Result<Article, ExtractError> {
        let doc = Html::parse_document(&page.html);

        // title: og:title → <title>
        let title = pick_meta(&doc, "og:title")
            .or_else(|| {
                let sel = Selector::parse("title").unwrap();
                doc.select(&sel)
                    .next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
            })
            .unwrap_or_else(|| page.final_url.to_string());

        let author = pick_meta(&doc, "article:author").or_else(|| pick_meta(&doc, "author"));
        let cover_url = pick_meta(&doc, "og:image").and_then(|s| Url::parse(&s).ok());

        let (body_html, extract_method) = pick_body(&doc, &page.html);

        let mut metadata = BTreeMap::new();
        metadata.insert("source_url".into(), page.final_url.to_string());
        metadata.insert("extract_method".into(), extract_method);

        Ok(Article {
            title,
            author,
            publish_time: None,
            body_html,
            cover_url,
            metadata,
        })
    }

    fn to_markdown(&self, article: &Article) -> MarkdownDoc {
        let body = html_to_markdown(&article.body_html).trim().to_string();
        let wc = count_words(&body);
        let rt = reading_time_minutes(wc);

        let extract_method = article
            .metadata
            .get("extract_method")
            .cloned()
            .unwrap_or_else(|| "generic".into());

        let mut fm: BTreeMap<String, YamlValue> = BTreeMap::new();
        fm.insert("title".into(), YamlValue::String(article.title.clone()));
        if let Some(a) = &article.author {
            fm.insert("author".into(), YamlValue::String(a.clone()));
        }
        if let Some(u) = &article.cover_url {
            fm.insert("cover_url".into(), YamlValue::String(u.to_string()));
        }
        if let Some(src_url) = article.metadata.get("source_url") {
            fm.insert("source_url".into(), YamlValue::String(src_url.clone()));
        }
        fm.insert("source".into(), YamlValue::String("url".into()));
        fm.insert("extract_method".into(), YamlValue::String(extract_method));
        fm.insert("source_adapter".into(), YamlValue::String("generic".into()));
        fm.insert("word_count".into(), YamlValue::Number(serde_yaml::Number::from(wc)));
        fm.insert(
            "reading_time_minutes".into(),
            YamlValue::Number(serde_yaml::Number::from(rt)),
        );
        fm.insert(
            "fetched_at".into(),
            YamlValue::String(
                OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            ),
        );

        MarkdownDoc { frontmatter: fm, body }
    }
}

/// 返回 (body_html, extract_method_label).
fn pick_body(doc: &Html, raw_html: &str) -> (String, String) {
    // 1. 多 article → 合并
    let article_sel = Selector::parse("article").unwrap();
    let articles: Vec<String> = doc.select(&article_sel).map(|e| e.inner_html()).collect();
    if articles.len() >= 2 {
        // 列表式首页: 合并所有 article,用 --- 分隔
        let merged = articles.join("\n\n<hr/>\n\n");
        return (merged, format!("generic/articles-merged-{}", articles.len()));
    }
    if articles.len() == 1 {
        return (articles[0].clone(), "generic/article".into());
    }

    // 2. <main>
    let main_sel = Selector::parse("main").unwrap();
    if let Some(el) = doc.select(&main_sel).next() {
        return (el.inner_html(), "generic/main".into());
    }

    // 3. <body>
    let body_sel = Selector::parse("body").unwrap();
    if let Some(el) = doc.select(&body_sel).next() {
        return (el.inner_html(), "generic/body".into());
    }

    // 4. fallback: 原始 HTML
    (raw_html.to_string(), "generic/raw".into())
}

fn pick_meta(doc: &Html, property: &str) -> Option<String> {
    let selector = format!(r#"meta[property="{p}"], meta[name="{p}"]"#, p = property);
    let sel = Selector::parse(&selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;
    use url_md_core::fetcher::FetchedPage;

    fn page(html: &str) -> FetchedPage {
        FetchedPage {
            html: html.to_string(),
            final_url: Url::parse("https://example.com/").unwrap(),
            status: 200,
            fetched_at: OffsetDateTime::now_utc(),
        }
    }

    #[test]
    fn matches_any_url() {
        let a = GenericAdapter::new();
        assert!(a.matches(&Url::parse("https://anything.example").unwrap()));
    }

    #[test]
    fn strategy_is_plain_http() {
        let a = GenericAdapter::new();
        assert!(matches!(
            a.strategy(&Url::parse("https://x.com").unwrap()),
            Strategy::Http
        ));
    }

    #[test]
    fn picks_single_article() {
        let html = "<html><body><article><p>solo</p></article></body></html>";
        let (body, method) = pick_body(&Html::parse_document(html), html);
        assert!(body.contains("solo"));
        assert_eq!(method, "generic/article");
    }

    #[test]
    fn merges_multiple_articles() {
        let html = r#"
        <html><body>
          <article><h2>Post 1</h2></article>
          <article><h2>Post 2</h2></article>
          <article><h2>Post 3</h2></article>
        </body></html>"#;
        let (body, method) = pick_body(&Html::parse_document(html), html);
        assert!(body.contains("Post 1") && body.contains("Post 2") && body.contains("Post 3"));
        assert_eq!(method, "generic/articles-merged-3");
    }

    #[test]
    fn falls_back_to_main_when_no_article() {
        let html = "<html><body><main><p>main content</p></main></body></html>";
        let (body, method) = pick_body(&Html::parse_document(html), html);
        assert!(body.contains("main content"));
        assert_eq!(method, "generic/main");
    }

    #[test]
    fn falls_back_to_body_when_no_article_no_main() {
        let html = "<html><body><p>body content</p></body></html>";
        let (body, method) = pick_body(&Html::parse_document(html), html);
        assert!(body.contains("body content"));
        assert_eq!(method, "generic/body");
    }

    #[test]
    fn extract_includes_source_url_in_metadata() {
        let a = GenericAdapter::new();
        let p = page("<html><body><p>hi</p></body></html>");
        let art = a.extract(&p).unwrap();
        assert!(art.metadata.contains_key("source_url"));
        assert!(art.metadata.contains_key("extract_method"));
    }
}
