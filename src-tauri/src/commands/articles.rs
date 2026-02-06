use regex::Regex;
use scraper::{Html, Selector};
use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use tauri::State;
use url::Url;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub created_at: String,
    pub feed_title: Option<String>,
}

const ARTICLE_COLUMNS: &str = "\
    a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content, \
    a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at, \
    f.title AS feed_title";

#[tauri::command]
pub async fn get_articles(
    feed_id: Option<i64>,
    offset: i64,
    limit: i64,
    unread_only: bool,
    favorites_only: bool,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Article>, String> {
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a JOIN feeds f ON a.feed_id = f.id WHERE 1=1"
    ));

    if let Some(fid) = feed_id {
        qb.push(" AND a.feed_id = ").push_bind(fid);
    }
    if unread_only {
        qb.push(" AND a.is_read = 0");
    }
    if favorites_only {
        qb.push(" AND a.is_favorite = 1");
    }

    qb.push(" ORDER BY a.published_at DESC NULLS LAST, a.created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    qb.build_query_as::<Article>()
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article(article_id: i64, pool: State<'_, SqlitePool>) -> Result<Article, String> {
    sqlx::query_as::<_, Article>(&format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a JOIN feeds f ON a.feed_id = f.id WHERE a.id = ?"
    ))
    .bind(article_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_read(article_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("UPDATE articles SET is_read = CASE WHEN is_read = 0 THEN 1 ELSE 0 END WHERE id = ?")
        .bind(article_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn mark_all_read(feed_id: Option<i64>, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if let Some(fid) = feed_id {
        sqlx::query("UPDATE articles SET is_read = 1 WHERE feed_id = ? AND is_read = 0")
            .bind(fid)
            .execute(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE articles SET is_read = 1 WHERE is_read = 0")
            .execute(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn mark_all_unread(feed_id: Option<i64>, pool: State<'_, SqlitePool>) -> Result<(), String> {
    if let Some(fid) = feed_id {
        sqlx::query("UPDATE articles SET is_read = 0 WHERE feed_id = ? AND is_read = 1")
            .bind(fid)
            .execute(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE articles SET is_read = 0 WHERE is_read = 1")
            .execute(pool.inner())
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_favorites_count(pool: State<'_, SqlitePool>) -> Result<i64, String> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles WHERE is_favorite = 1")
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(count)
}

#[tauri::command]
pub async fn toggle_favorite(article_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("UPDATE articles SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END WHERE id = ?")
        .bind(article_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_articles(query: String, limit: i64, pool: State<'_, SqlitePool>) -> Result<Vec<Article>, String> {
    sqlx::query_as::<_, Article>(&format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a \
         JOIN articles_fts fts ON a.id = fts.rowid \
         JOIN feeds f ON a.feed_id = f.id \
         WHERE articles_fts MATCH ? \
         ORDER BY bm25(articles_fts) \
         LIMIT ?"
    ))
    .bind(&query)
    .bind(limit)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

/// Fetch full article content from the web, extract the main body, and cache it in the DB.
#[tauri::command]
pub async fn fetch_article_content(
    article_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<String, String> {
    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT link, content FROM articles WHERE id = ?")
            .bind(article_id)
            .fetch_one(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

    let link = row.0.ok_or("Article has no link")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let html = client
        .get(&link)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let extracted = extract_article_content(&html);
    let content = resolve_relative_urls(&extracted, &link);

    // Cache the full content in the DB
    sqlx::query("UPDATE articles SET content = ? WHERE id = ?")
        .bind(&content)
        .bind(article_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(content)
}

/// Extract the main article content from an HTML page.
fn extract_article_content(html: &str) -> String {
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
        if let Ok(sel) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&sel).next() {
                let inner = el.inner_html();
                let cleaned = clean_html(&inner);
                if cleaned.len() > 200 {
                    return cleaned;
                }
            }
        }
    }

    // Fallback: use body
    if let Ok(body_sel) = Selector::parse("body") {
        if let Some(body) = doc.select(&body_sel).next() {
            return clean_html(&body.inner_html());
        }
    }

    String::new()
}

/// Remove noise elements and strip framework attributes from HTML.
fn clean_html(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let noise_tags = [
        "nav", "footer", "header", "aside", "script", "style", "noscript",
        "iframe", "form", "svg",
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

    // Strip class and style attributes — they reference CSS we don't have
    let class_re = Regex::new(r#"\s+class="[^"]*""#).unwrap();
    let style_re = Regex::new(r#"\s+style="[^"]*""#).unwrap();
    output = class_re.replace_all(&output, "").to_string();
    output = style_re.replace_all(&output, "").to_string();

    output
}

/// Resolve relative URLs (src, href, poster) in HTML to absolute using the given base URL.
/// Handles root-relative (/path), path-relative (img/foo.png), and leaves absolute URLs as-is.
pub(crate) fn resolve_relative_urls(html: &str, base_url: &str) -> String {
    let base = match Url::parse(base_url) {
        Ok(u) => u,
        Err(_) => return html.to_string(),
    };

    // Double-quoted attributes
    let re_dq = Regex::new(r#"(src|href|poster)\s*=\s*"([^"]+)""#).unwrap();
    let result = re_dq
        .replace_all(html, |caps: &regex::Captures| {
            resolve_attr(&base, &caps[0], &caps[1], &caps[2], '"')
        })
        .to_string();

    // Single-quoted attributes
    let re_sq = Regex::new(r#"(src|href|poster)\s*=\s*'([^']+)'"#).unwrap();
    re_sq
        .replace_all(&result, |caps: &regex::Captures| {
            resolve_attr(&base, &caps[0], &caps[1], &caps[2], '\'')
        })
        .to_string()
}

fn resolve_attr(base: &Url, full_match: &str, attr: &str, url_val: &str, quote: char) -> String {
    // Skip data URIs, anchors, javascript:, mailto:
    if url_val.starts_with("data:")
        || url_val.starts_with('#')
        || url_val.starts_with("javascript:")
        || url_val.starts_with("mailto:")
    {
        return full_match.to_string();
    }

    match base.join(url_val) {
        Ok(resolved) => format!("{}={1}{2}{1}", attr, quote, resolved.as_str()),
        Err(_) => full_match.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_root_relative_src() {
        let html = r#"<img src="/images/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(result, r#"<img src="https://example.com/images/photo.jpg">"#);
    }

    #[test]
    fn resolve_path_relative_src() {
        let html = r#"<img src="images/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(result, r#"<img src="https://example.com/blog/images/photo.jpg">"#);
    }

    #[test]
    fn resolve_parent_relative_src() {
        let html = r#"<img src="../images/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post/article");
        assert_eq!(result, r#"<img src="https://example.com/blog/images/photo.jpg">"#);
    }

    #[test]
    fn leave_absolute_urls_unchanged() {
        let html = r#"<img src="https://cdn.example.com/photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog");
        assert_eq!(result, r#"<img src="https://cdn.example.com/photo.jpg">"#);
    }

    #[test]
    fn leave_protocol_relative_urls_unchanged() {
        let html = r#"<img src="//cdn.example.com/photo.jpg">"#;
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
    fn skip_javascript_hrefs() {
        let html = r#"<a href="javascript:void(0)">Click</a>"#;
        let result = resolve_relative_urls(html, "https://example.com");
        assert_eq!(result, r#"<a href="javascript:void(0)">Click</a>"#);
    }

    #[test]
    fn skip_mailto_links() {
        let html = r#"<a href="mailto:test@example.com">Email</a>"#;
        let result = resolve_relative_urls(html, "https://example.com");
        assert_eq!(result, r#"<a href="mailto:test@example.com">Email</a>"#);
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
    fn resolve_poster_attribute() {
        let html = r#"<video poster="/thumb.jpg"></video>"#;
        let result = resolve_relative_urls(html, "https://example.com/videos/v1");
        assert_eq!(result, r#"<video poster="https://example.com/thumb.jpg"></video>"#);
    }

    #[test]
    fn invalid_base_url_returns_unchanged() {
        let html = r#"<img src="/photo.jpg">"#;
        let result = resolve_relative_urls(html, "not-a-url");
        assert_eq!(result, html);
    }

    #[test]
    fn empty_html_returns_empty() {
        let result = resolve_relative_urls("", "https://example.com");
        assert_eq!(result, "");
    }

    #[test]
    fn no_matching_attributes_returns_unchanged() {
        let html = "<p>Hello world</p>";
        let result = resolve_relative_urls(html, "https://example.com");
        assert_eq!(result, html);
    }

    #[test]
    fn multiple_images_all_resolved() {
        let html = r#"<img src="/a.jpg"><img src="/b.png"><img src="c.gif">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/post");
        assert_eq!(
            result,
            r#"<img src="https://example.com/a.jpg"><img src="https://example.com/b.png"><img src="https://example.com/blog/c.gif">"#
        );
    }

    #[test]
    fn resolve_with_trailing_slash_base() {
        let html = r#"<img src="photo.jpg">"#;
        let result = resolve_relative_urls(html, "https://example.com/blog/");
        assert_eq!(result, r#"<img src="https://example.com/blog/photo.jpg">"#);
    }
}
