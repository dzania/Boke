use crate::db::{Database, DbResult};
use crate::models::{Article, ArticleQuery};
use std::sync::Arc;

pub struct ArticleService<D: Database> {
    db: Arc<D>,
    http_client: reqwest::Client,
}

impl<D: Database> ArticleService<D> {
    pub fn new(db: Arc<D>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Boke RSS Reader")
            .build()
            .expect("Failed to create HTTP client");

        Self { db, http_client }
    }

    pub async fn get_articles(&self, query: ArticleQuery) -> DbResult<Vec<Article>> {
        self.db.get_articles(&query).await
    }

    pub async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        self.db.get_article(id).await
    }

    pub async fn toggle_read(&self, id: i64) -> DbResult<()> {
        self.db.toggle_read(id).await
    }

    pub async fn mark_all_read(&self, feed_id: Option<i64>) -> DbResult<()> {
        self.db.mark_all_read(feed_id).await
    }

    pub async fn mark_all_unread(&self, feed_id: Option<i64>) -> DbResult<()> {
        self.db.mark_all_unread(feed_id).await
    }

    pub async fn toggle_favorite(&self, id: i64) -> DbResult<()> {
        self.db.toggle_favorite(id).await
    }

    pub async fn get_favorites_count(&self) -> DbResult<i64> {
        self.db.get_favorites_count().await
    }

    pub async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<Article>> {
        self.db.search_articles(query, limit).await
    }

    pub async fn fetch_article_content(&self, id: i64) -> anyhow::Result<String> {
        // Check if content is already cached
        if let Some(article) = self.db.get_article(id).await? {
            if let Some(content) = article.content
                && !content.is_empty()
            {
                return Ok(content);
            }

            // Fetch content from URL
            if let Some(link) = article.link {
                let response = self.http_client.get(&link).send().await?;
                let html = response.text().await?;
                let content = extract_main_content(&html);

                // Cache the content
                self.db.update_article_content(id, &content).await?;

                return Ok(content);
            }
        }

        Err(anyhow::anyhow!("Article not found or has no link"))
    }
}

fn extract_main_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

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
            // Remove unwanted elements
            let mut content = element.inner_html();

            // Basic cleanup
            content = content
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
                return content;
            }
        }
    }

    // Fallback: return body content
    if let Ok(selector) = Selector::parse("body")
        && let Some(element) = document.select(&selector).next()
    {
        return element.inner_html();
    }

    html.to_string()
}
