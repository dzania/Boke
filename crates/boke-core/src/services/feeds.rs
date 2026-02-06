use crate::db::{Database, DbResult, InsertResult};
use crate::feed::{FeedParser, discovery};
use crate::models::{FeedWithMeta, NewArticle, NewFeed};
use std::sync::Arc;

pub struct FeedService<D: Database> {
    db: Arc<D>,
    http_client: reqwest::Client,
}

impl<D: Database> FeedService<D> {
    pub fn new(db: Arc<D>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Boke RSS Reader")
            .build()
            .expect("Failed to create HTTP client");

        Self { db, http_client }
    }

    pub async fn get_feeds(&self) -> DbResult<Vec<FeedWithMeta>> {
        self.db.get_feeds().await
    }

    pub async fn add_feed(&self, url: &str) -> anyhow::Result<FeedWithMeta> {
        // Discover feed URL if needed
        let feed_url = match discovery::discover(url).await {
            Ok(feeds) if !feeds.is_empty() => feeds[0].url.clone(),
            _ => url.to_string(),
        };

        // Fetch and parse feed
        let response = self.http_client.get(&feed_url).send().await?;
        let body = response.text().await?;
        let parsed = FeedParser::parse(&body, &feed_url)?;

        // Insert feed into database
        let new_feed = NewFeed {
            title: parsed.title.clone(),
            folder_id: None,
            feed_url: parsed.feed_url.clone(),
            site_url: parsed.site_url().map(|s| s.to_string()),
            description: parsed.description.clone(),
            language: parsed.language.clone(),
            favicon_url: None,
            last_build_date: parsed.last_build_date(),
        };

        let feed_id = self.db.insert_feed(&new_feed).await?;

        // Insert articles
        for entry in parsed.items() {
            let new_article = NewArticle {
                feed_id,
                guid: entry.guid().to_string(),
                title: entry.title.clone(),
                link: if entry.link.is_empty() {
                    None
                } else {
                    Some(entry.link.clone())
                },
                author: entry.author.clone(),
                summary: entry.summary.clone(),
                content: entry.content.clone(),
                image_url: entry.image_url.clone(),
                published_at: entry.published_at(),
            };
            let _ = self.db.insert_article(&new_article).await;
        }

        // Fetch favicon in background (best effort)
        if let Some(site_url) = parsed.site_url()
            && let Ok(favicon) = self.fetch_favicon(site_url).await {
                let _ = self.db.update_feed_favicon(feed_id, &favicon).await;
            }

        // Return the feed with metadata
        let feeds = self.db.get_feeds().await?;
        feeds
            .into_iter()
            .find(|f| f.id == feed_id)
            .ok_or_else(|| anyhow::anyhow!("Feed not found after insert"))
    }

    pub async fn remove_feed(&self, feed_id: i64) -> DbResult<()> {
        self.db.delete_feed(feed_id).await
    }

    pub async fn refresh_feed(&self, feed_id: i64) -> anyhow::Result<RefreshResult> {
        let feed_url = self
            .db
            .get_feed_url(feed_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Feed not found"))?;

        let response = self.http_client.get(&feed_url).send().await?;
        let body = response.text().await?;
        let parsed = FeedParser::parse(&body, &feed_url)?;

        let mut new_count = 0;
        for entry in parsed.items() {
            let new_article = NewArticle {
                feed_id,
                guid: entry.guid().to_string(),
                title: entry.title.clone(),
                link: if entry.link.is_empty() {
                    None
                } else {
                    Some(entry.link.clone())
                },
                author: entry.author.clone(),
                summary: entry.summary.clone(),
                content: entry.content.clone(),
                image_url: entry.image_url.clone(),
                published_at: entry.published_at(),
            };
            if let InsertResult::Inserted(_) = self.db.insert_article(&new_article).await? {
                new_count += 1;
            }
        }

        self.db.update_feed_last_fetched(feed_id).await?;

        Ok(RefreshResult {
            feed_id,
            new_articles: new_count,
        })
    }

    pub async fn refresh_all_feeds(&self) -> anyhow::Result<Vec<RefreshResult>> {
        let feeds = self.db.get_feeds().await?;
        let mut results = Vec::new();

        for feed in feeds {
            match self.refresh_feed(feed.id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    log::warn!("Failed to refresh feed {}: {}", feed.id, e);
                    results.push(RefreshResult {
                        feed_id: feed.id,
                        new_articles: 0,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn fetch_favicon(&self, site_url: &str) -> anyhow::Result<String> {
        let url = url::Url::parse(site_url)?;
        let favicon_url = format!(
            "{}://{}/favicon.ico",
            url.scheme(),
            url.host_str().unwrap_or("")
        );

        let response = self.http_client.head(&favicon_url).send().await?;
        if response.status().is_success() {
            return Ok(favicon_url);
        }

        // Fallback: try to parse HTML for link rel="icon"
        let response = self.http_client.get(site_url).send().await?;
        let html = response.text().await?;

        // Simple extraction using scraper
        use scraper::{Html, Selector};
        let document = Html::parse_document(&html);

        // Try various link selectors
        let selectors = [
            r#"link[rel="icon"]"#,
            r#"link[rel="shortcut icon"]"#,
            r#"link[rel="apple-touch-icon"]"#,
        ];

        for sel_str in selectors {
            if let Ok(selector) = Selector::parse(sel_str)
                && let Some(element) = document.select(&selector).next()
                    && let Some(href) = element.value().attr("href") {
                        if href.starts_with("http") {
                            return Ok(href.to_string());
                        } else {
                            let base = url::Url::parse(site_url)?;
                            return Ok(base.join(href)?.to_string());
                        }
                    }
        }

        Err(anyhow::anyhow!("Favicon not found"))
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RefreshResult {
    pub feed_id: i64,
    pub new_articles: i64,
}
