//! Utility functions for HTML processing and content extraction.

use scraper::{Html, Selector};
use url::Url;

/// Resolve relative URLs (src, href, poster) in HTML to absolute using the given base URL.
/// Handles root-relative (/path), path-relative (img/foo.png), and leaves absolute URLs as-is.
pub fn resolve_relative_urls(html: &str, base_url: &str) -> String {
    let base = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return html.to_string(),
    };

    let mut result = html.to_string();

    // Process double-quoted attributes
    result = resolve_urls_with_quote(&result, &base, '"');

    // Process single-quoted attributes
    result = resolve_urls_with_quote(&result, &base, '\'');

    result
}

fn resolve_urls_with_quote(html: &str, base: &Url, quote: char) -> String {
    let mut result = html.to_string();
    let attrs = ["src", "href", "poster"];

    for attr in attrs {
        let pattern = format!("{}={}", attr, quote);
        let mut search_start = 0;

        while let Some(start) = result[search_start..].find(&pattern) {
            let abs_start = search_start + start;
            let value_start = abs_start + pattern.len();

            if let Some(end_offset) = result[value_start..].find(quote) {
                let value_end = value_start + end_offset;
                let url_val = &result[value_start..value_end];

                // Skip special URLs
                if !url_val.starts_with("data:")
                    && !url_val.starts_with('#')
                    && !url_val.starts_with("javascript:")
                    && !url_val.starts_with("mailto:")
                    && let Ok(resolved) = base.join(url_val) {
                        let replacement =
                            format!("{}={}{}{}", attr, quote, resolved.as_str(), quote);
                        let full_match_end = value_end + 1;
                        result = format!(
                            "{}{}{}",
                            &result[..abs_start],
                            replacement,
                            &result[full_match_end..]
                        );
                        search_start = abs_start + replacement.len();
                        continue;
                    }
                search_start = value_end + 1;
            } else {
                break;
            }
        }
    }

    result
}

/// Extract the main article content from an HTML page.
pub fn extract_article_content(html: &str) -> String {
    let doc = Html::parse_document(html);

    // Try selectors in order of specificity
    let selectors = [
        "article",
        "[role='main']",
        "main",
        ".post-content",
        ".entry-content",
        ".article-content",
        ".article-body",
        ".prose",
        ".content",
    ];

    for sel_str in &selectors {
        if let Ok(sel) = Selector::parse(sel_str)
            && let Some(el) = doc.select(&sel).next() {
                let inner = el.inner_html();
                let cleaned = clean_html(&inner);
                if cleaned.len() > 200 {
                    return cleaned;
                }
            }
    }

    // Fallback: use body
    if let Ok(body_sel) = Selector::parse("body")
        && let Some(body) = doc.select(&body_sel).next() {
            return clean_html(&body.inner_html());
        }

    String::new()
}

/// Remove noise elements from HTML.
fn clean_html(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let noise_tags = [
        "nav", "footer", "header", "aside", "script", "style", "noscript", "iframe", "form", "svg",
    ];

    let mut output = html.to_string();
    for tag in &noise_tags {
        if let Ok(sel) = Selector::parse(tag) {
            for el in doc.select(&sel) {
                let outer = el.html();
                output = output.replace(&outer, "");
            }
        }
    }

    // Strip class and style attributes
    output = remove_attribute(&output, "class");
    output = remove_attribute(&output, "style");

    output
}

fn remove_attribute(html: &str, attr: &str) -> String {
    let mut result = html.to_string();
    let pattern_dq = format!(" {}=\"", attr);
    let pattern_sq = format!(" {}='", attr);

    // Remove double-quoted attributes
    while let Some(start) = result.find(&pattern_dq) {
        let value_start = start + pattern_dq.len();
        if let Some(end_offset) = result[value_start..].find('"') {
            let end = value_start + end_offset + 1;
            result = format!("{}{}", &result[..start], &result[end..]);
        } else {
            break;
        }
    }

    // Remove single-quoted attributes
    while let Some(start) = result.find(&pattern_sq) {
        let value_start = start + pattern_sq.len();
        if let Some(end_offset) = result[value_start..].find('\'') {
            let end = value_start + end_offset + 1;
            result = format!("{}{}", &result[..start], &result[end..]);
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_root_relative_src() {
        let html = r#"<img src="/images/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(
            result,
            r#"<img src="https://example.com/images/photo.jpg">"#
        );
    }

    #[test]
    fn resolve_path_relative_src() {
        let html = r#"<img src="images/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(
            result,
            r#"<img src="https://example.com/blog/images/photo.jpg">"#
        );
    }

    #[test]
    fn leave_absolute_urls_unchanged() {
        let html = r#"<img src="https://cdn.example.com/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog");
        assert_eq!(result, r#"<img src="https://cdn.example.com/photo.jpg">"#);
    }

    #[test]
    fn skip_data_uris() {
        let html = r#"<img src="data:image/png;base64,abc123">"#;
        let result = resolve_relative_urls(html, "https://example.com");
        assert_eq!(result, r#"<img src="data:image/png;base64,abc123">"#);
    }

    #[test]
    fn skip_anchor_links() {
        let html = r##"<a href="#section">Link</a>"##;
        let result = resolve_relative_urls(html, "https://example.com/page");
        assert_eq!(result, r##"<a href="#section">Link</a>"##);
    }

    #[test]
    fn resolve_href_and_src_together() {
        let html = r#"<a href="/page"><img src="/img/pic.png"></a>"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(
            result,
            r#"<a href="https://example.com/page"><img src="https://example.com/img/pic.png"></a>"#
        );
    }

    #[test]
    fn resolve_single_quoted_attributes() {
        let html = "<img src='/images/photo.jpg'>";
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(result, "<img src='https://example.com/images/photo.jpg'>");
    }

    #[test]
    fn invalid_base_url_returns_unchanged() {
        let html = r#"<img src="/photo.jpg">"#;
        let result = resolve_relative_urls(html, "not-a-url");
        assert_eq!(result, html);
    }
}
