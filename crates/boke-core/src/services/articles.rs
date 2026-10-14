use crate::db::{Database, DbResult};
use crate::models::{Article, ArticleListItem, ArticleQuery, ArticleScope};
use crate::utils::{resolve_relative_urls, visible_text_length};
use scraper::{Html, Selector};
use std::sync::Arc;

use super::read_body_limited;
use std::time::Duration;
use url::Url;

/// Stored bodies with less visible text than this are treated as excerpts, and the reader fetches
/// the full page instead.
pub const CONTENT_SHORT_THRESHOLD_CHARS: usize = 800;

/// Upper bound on a fetched article page. Larger responses are rejected rather than buffered.
const PAGE_BYTES_MAX: usize = 8 * 1024 * 1024;

const PAGE_FETCH_TIMEOUT: Duration = Duration::from_secs(20);

pub struct ArticleService<D: Database> {
    db: Arc<D>,
    http_client: reqwest::Client,
}

impl<D: Database> ArticleService<D> {
    pub fn new(db: Arc<D>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(PAGE_FETCH_TIMEOUT)
            .user_agent("Boke RSS Reader")
            .build()
            .expect("HTTP client config is static and valid");

        Self { db, http_client }
    }

    pub async fn get_articles(&self, query: ArticleQuery) -> DbResult<Vec<ArticleListItem>> {
        self.db.get_articles(&query).await
    }

    pub async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        self.db.get_article(id).await
    }

    pub async fn set_read(&self, id: i64, is_read: bool) -> DbResult<()> {
        self.db.set_read(id, is_read).await
    }

    pub async fn set_favorite(&self, id: i64, is_favorite: bool) -> DbResult<()> {
        self.db.set_favorite(id, is_favorite).await
    }

    pub async fn mark_all_read(&self, scope: ArticleScope) -> DbResult<()> {
        self.db.mark_all_read(scope).await
    }

    pub async fn mark_all_unread(&self, scope: ArticleScope) -> DbResult<()> {
        self.db.mark_all_unread(scope).await
    }

    pub async fn get_favorites_count(&self) -> DbResult<i64> {
        self.db.get_favorites_count().await
    }

    pub async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<ArticleListItem>> {
        self.db.search_articles(query, limit).await
    }

    /// Returns the best available HTML body for an article.
    ///
    /// Uses the stored `content` (or `summary`) when it has at least
    /// [`CONTENT_SHORT_THRESHOLD_CHARS`] of visible text. Otherwise it fetches the article page,
    /// extracts the main content, resolves relative URLs against the page's real location, and
    /// caches the result when it is longer than what was stored.
    ///
    /// If the fetch fails but a stored body exists, the stored body is returned and the error is
    /// logged, so the reader still has something to show.
    ///
    /// # Errors
    /// Fails when the article does not exist, or when it has no stored body and the page fetch
    /// fails.
    pub async fn get_article_content(&self, id: i64) -> anyhow::Result<String> {
        let article = self
            .db
            .get_article(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Article {id} not found"))?;

        let stored = [article.content, article.summary]
            .into_iter()
            .flatten()
            .find(|body| !body.trim().is_empty());
        let stored_length = stored.as_deref().map_or(0, visible_text_length);
        if stored_length >= CONTENT_SHORT_THRESHOLD_CHARS {
            return Ok(stored.unwrap_or_default());
        }

        let Some(link) = article.link.filter(|link| !link.is_empty()) else {
            return Ok(stored.unwrap_or_default());
        };

        match self.fetch_page_content(&link).await {
            Ok(fetched) if visible_text_length(&fetched) > stored_length => {
                self.db.update_article_content(id, &fetched).await?;
                Ok(fetched)
            }
            Ok(_) => Ok(stored.unwrap_or_default()),
            Err(error) => match stored {
                Some(stored) => {
                    log::warn!("Failed to fetch full content for article {id}: {error}");
                    Ok(stored)
                }
                None => Err(error),
            },
        }
    }

    async fn fetch_page_content(&self, link: &str) -> anyhow::Result<String> {
        let response = self
            .http_client
            .get(link)
            .send()
            .await?
            .error_for_status()?;
        // Redirects change the document's location, so resolve against where we ended up.
        let page_url = response.url().clone();
        let html = read_body_limited(response, PAGE_BYTES_MAX).await?;

        Ok(extract_resolved_content(&html, &page_url))
    }
}

/// Extracts the main content of a page and absolutizes its URLs.
///
/// The base is the page's `<base href>` when present, else `page_url`.
fn extract_resolved_content(html: &str, page_url: &Url) -> String {
    let document = Html::parse_document(html);
    let base = document_base_url(&document, page_url);
    let content = extract_main_content(&document).unwrap_or_else(|| html.to_string());
    resolve_relative_urls(&content, &base)
}

fn document_base_url(document: &Html, page_url: &Url) -> Url {
    let selector = Selector::parse("base[href]").expect("static selector is valid");
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("href"))
        .and_then(|href| page_url.join(href.trim()).ok())
        .unwrap_or_else(|| page_url.clone())
}

fn extract_main_content(document: &Html) -> Option<String> {
    // Try various selectors for main content
    let selectors = [
        "article",
        "[role='main']",
        "main",
        ".post-content",
        ".article-content",
        ".entry-content",
        ".content",
        "#content",
    ];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str)
            && let Some(element) = document.select(&selector).next()
        {
            let content = element
                .inner_html()
                .lines()
                .filter(|line| {
                    let lower = line.to_lowercase();
                    !lower.contains("<nav")
                        && !lower.contains("<footer")
                        && !lower.contains("<script")
                        && !lower.contains("<style")
                        && !lower.contains("<iframe")
                        && !lower.contains("<aside")
                })
                .collect::<Vec<_>>()
                .join("\n");

            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    // Fallback: return body content
    let selector = Selector::parse("body").expect("static selector is valid");
    document
        .select(&selector)
        .next()
        .map(|element| element.inner_html())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(value: &str) -> Url {
        Url::parse(value).expect("test URL must parse")
    }

    #[test]
    fn extracted_content_resolves_against_page_url() {
        let html = r#"<html><body><article><img src="img/a.png"></article></body></html>"#;
        let content = extract_resolved_content(html, &url("https://example.com/blog/post/"));
        assert_eq!(
            content,
            r#"<img src="https://example.com/blog/post/img/a.png">"#
        );
    }

    #[test]
    fn base_href_takes_precedence_over_page_url() {
        let html = r#"<html><head><base href="https://cdn.example.org/assets/"></head>
            <body><article><img src="a.png"></article></body></html>"#;
        let content = extract_resolved_content(html, &url("https://example.com/post"));
        assert_eq!(
            content,
            r#"<img src="https://cdn.example.org/assets/a.png">"#
        );
    }

    #[test]
    fn relative_base_href_is_resolved_against_page_url() {
        let html = r#"<html><head><base href="/static/"></head>
            <body><article><img src="a.png"></article></body></html>"#;
        let content = extract_resolved_content(html, &url("https://example.com/post"));
        assert_eq!(content, r#"<img src="https://example.com/static/a.png">"#);
    }

    #[test]
    fn falls_back_to_body_without_known_container() {
        let html = r#"<html><body><p>Hi</p><img src="/a.png"></body></html>"#;
        let content = extract_resolved_content(html, &url("https://example.com/post"));
        assert_eq!(content, r#"<p>Hi</p><img src="https://example.com/a.png">"#);
    }
}
