//! WeixinAdapter — mp.weixin.qq.com 文章专用.
//!
//! 覆盖永久链 `/s/*`. M1 Spike 验证 reqwest 快路 100% 命中,默认 Strategy::Http.

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

pub struct WeixinAdapter;

impl WeixinAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WeixinAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for WeixinAdapter {
    fn name(&self) -> &'static str {
        "weixin"
    }

    fn matches(&self, url: &Url) -> bool {
        match url.host_str() {
            Some(h) => h == "mp.weixin.qq.com",
            None => false,
        }
    }

    fn strategy(&self, url: &Url) -> Strategy {
        // 路由策略(spec/dev/cdp-fetcher.spec.md scaffold):
        // - 永久链 /s/* → Http 快路(M1 Spike 实测 100% 命中)
        // - profile / 视频号 / 历史推送 → 强制 Cdp(快路必失败)
        //   Phase 2 实际 CdpFetcher 落地前会返回 CdpUnavailable.
        match url.path() {
            p if p.starts_with("/s/") => Strategy::HttpFirstCdpFallback,
            p if is_cdp_only_path(p) => Strategy::Cdp,
            _ => Strategy::HttpFirstCdpFallback,
        }
    }

    fn content_marker(&self) -> Option<&str> {
        Some(r#"id="js_content""#)
    }

    fn extract(&self, page: &FetchedPage) -> Result<Article, ExtractError> {
        let doc = Html::parse_document(&page.html);

        let title = pick_text(&doc, "h1#activity-name")
            .or_else(|| pick_meta(&doc, "og:title"))
            .unwrap_or_else(|| "未找到标题".to_string());

        let author = pick_text(&doc, "#js_author_name")
            .or_else(|| pick_text(&doc, "#js_name"));

        let publish_time_str = pick_text(&doc, "#publish_time");

        let cover_url = pick_meta(&doc, "og:image")
            .and_then(|s| Url::parse(&s).ok());

        let body_html = pick_html(&doc, "#js_content")
            .ok_or_else(|| ExtractError("#js_content 未找到,可能是反爬拦截页".to_string()))?;

        let mut metadata = BTreeMap::new();
        if let Some(pt) = publish_time_str {
            metadata.insert("publish_time_raw".into(), pt);
        }
        metadata.insert("source_url".into(), page.final_url.to_string());

        Ok(Article {
            title,
            author,
            publish_time: None, // 微信的 publish_time 格式需专门解析,留待后续
            body_html,
            cover_url,
            metadata,
        })
    }

    fn to_markdown(&self, article: &Article) -> MarkdownDoc {
        let body = html_to_markdown(&article.body_html).trim().to_string();
        let wc = count_words(&body);
        let rt = reading_time_minutes(wc);

        let mut fm: BTreeMap<String, YamlValue> = BTreeMap::new();
        fm.insert("title".into(), YamlValue::String(article.title.clone()));
        if let Some(a) = &article.author {
            fm.insert("author".into(), YamlValue::String(a.clone()));
        }
        if let Some(pt) = article.metadata.get("publish_time_raw") {
            fm.insert("publish_time".into(), YamlValue::String(pt.clone()));
        }
        if let Some(u) = &article.cover_url {
            fm.insert("cover_url".into(), YamlValue::String(u.to_string()));
        }
        if let Some(src_url) = article.metadata.get("source_url") {
            fm.insert("source_url".into(), YamlValue::String(src_url.clone()));
        }
        fm.insert("source".into(), YamlValue::String("url".into()));
        fm.insert("extract_method".into(), YamlValue::String("weixin".into()));
        fm.insert("source_adapter".into(), YamlValue::String("weixin".into()));
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

/// Phase 2 scaffold: 这些 path 必须走 CDP(快路必失败).
/// CdpFetcher 真实现见 spec/dev/cdp-fetcher.spec.md.
fn is_cdp_only_path(path: &str) -> bool {
    // 公众号主页 / 视频号 / 历史推送列表 — 都是 JS 渲染 + 反爬严格.
    path.starts_with("/mp/profile_ext")
        || path.starts_with("/mp/homepage")
        || path.starts_with("/cgi-bin/")
        || path.starts_with("/finder/")
        || path.starts_with("/sph/")
}

fn pick_text(doc: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel).next().map(|e| {
        e.text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }).filter(|s| !s.is_empty())
}

fn pick_html(doc: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel).next().map(|e| e.inner_html())
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
    use url_md_core::adapter::Adapter;

    #[test]
    fn matches_weixin_permanent_link() {
        let a = WeixinAdapter::new();
        assert!(a.matches(&Url::parse("https://mp.weixin.qq.com/s/xxx").unwrap()));
    }

    #[test]
    fn rejects_non_weixin() {
        let a = WeixinAdapter::new();
        assert!(!a.matches(&Url::parse("https://example.com/s/xxx").unwrap()));
        assert!(!a.matches(&Url::parse("https://zhihu.com").unwrap()));
    }

    #[test]
    fn content_marker_checks_js_content() {
        let a = WeixinAdapter::new();
        assert_eq!(a.content_marker(), Some(r#"id="js_content""#));
    }

    #[test]
    fn strategy_is_http_first_cdp_fallback() {
        let a = WeixinAdapter::new();
        let url = Url::parse("https://mp.weixin.qq.com/s/xxx").unwrap();
        assert!(matches!(
            a.strategy(&url),
            url_md_core::adapter::Strategy::HttpFirstCdpFallback
        ));
    }

    #[test]
    fn strategy_profile_forces_cdp() {
        let a = WeixinAdapter::new();
        let url = Url::parse("https://mp.weixin.qq.com/mp/profile_ext?__biz=xx").unwrap();
        assert!(matches!(
            a.strategy(&url),
            url_md_core::adapter::Strategy::Cdp
        ));
    }

    #[test]
    fn strategy_video_account_forces_cdp() {
        let a = WeixinAdapter::new();
        let url = Url::parse("https://mp.weixin.qq.com/finder/abcdef").unwrap();
        assert!(matches!(
            a.strategy(&url),
            url_md_core::adapter::Strategy::Cdp
        ));
    }
}
