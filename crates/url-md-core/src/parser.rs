//! DOM → Markdown 保真转换.
//!
//! Phase 1 MVP: 在 html2md 之上加一层 preprocess,修复懒加载图片
//! (微信/知乎 等把真实 URL 放在 `data-src` 里,原 `src` 是空或 1x1 占位).

use once_cell::sync::Lazy;
use regex::Regex;

/// 把一段 HTML 片段转为 Markdown 文本.
pub fn html_to_markdown(html: &str) -> String {
    let preprocessed = restore_lazy_image_src(html);
    html2md::parse_html(&preprocessed)
}

/// 扫描每个 `<img ...>` 标签: 如果有 `data-src="URL"`,把 URL 写到 `src`
/// (同时删掉原 `src` 与 `data-src`,避免重复属性).
pub fn restore_lazy_image_src(html: &str) -> String {
    RE_IMG
        .replace_all(html, |caps: &regex::Captures| rewrite_img(&caps[1]))
        .into_owned()
}

static RE_IMG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<img([^>]*?)/?>").unwrap());

fn rewrite_img(attrs: &str) -> String {
    let Some(url) = extract_attr(attrs, "data-src") else {
        return format!("<img{attrs}>");
    };
    let without_src = remove_attr(attrs, "src");
    let without_both = remove_attr(&without_src, "data-src");
    format!(r#"<img{without_both} src="{url}">"#)
}

fn extract_attr(attrs: &str, name: &str) -> Option<String> {
    // 匹配 `name="..."`(属性名前必须是空白或开头,避免 "src" 误中 "data-src")
    let pat = format!(r#"(?:^|\s){name}\s*=\s*"([^"]*)""#);
    let re = Regex::new(&pat).ok()?;
    re.captures(attrs)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|s| !s.is_empty())
}

fn remove_attr(attrs: &str, name: &str) -> String {
    let pat = format!(r#"(?:^|\s){name}\s*=\s*"[^"]*""#);
    let re = Regex::new(&pat).unwrap();
    re.replace_all(attrs, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_data_src_when_src_is_empty() {
        let html = r#"<img src="" data-src="https://example.com/x.jpg" alt="a" />"#;
        let out = restore_lazy_image_src(html);
        assert!(out.contains(r#"src="https://example.com/x.jpg""#), "got: {out}");
        assert!(!out.contains("data-src="), "data-src should be removed: {out}");
    }

    #[test]
    fn restores_data_src_when_no_src() {
        let html = r#"<img data-src="https://example.com/y.jpg" class="z" />"#;
        let out = restore_lazy_image_src(html);
        assert!(out.contains(r#"src="https://example.com/y.jpg""#), "got: {out}");
    }

    #[test]
    fn preserves_existing_real_src_without_data_src() {
        let html = r#"<img src="https://example.com/real.jpg" />"#;
        let out = restore_lazy_image_src(html);
        assert!(out.contains(r#"src="https://example.com/real.jpg""#));
    }

    #[test]
    fn does_not_confuse_src_with_data_src_attr_name() {
        // 确保 extract_attr("src") 不会错把 data-src 的值取出来
        let html = r#"<img data-src="real.jpg" />"#;
        let out = restore_lazy_image_src(html);
        assert!(out.contains(r#"src="real.jpg""#), "got: {out}");
    }

    #[test]
    fn html_to_markdown_produces_image_link() {
        let html = r#"<p>see <img data-src="https://example.com/z.jpg" alt="z"></p>"#;
        let md = html_to_markdown(html);
        assert!(md.contains("https://example.com/z.jpg"), "got: {md}");
    }

    #[test]
    fn handles_multiple_imgs() {
        let html = r#"<img data-src="a.jpg"><img src="b.jpg"><img data-src="c.jpg" />"#;
        let out = restore_lazy_image_src(html);
        assert!(out.contains(r#"src="a.jpg""#));
        assert!(out.contains(r#"src="b.jpg""#));
        assert!(out.contains(r#"src="c.jpg""#));
    }
}
