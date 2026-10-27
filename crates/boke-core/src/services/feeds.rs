use crate::db::{Database, DbResult};
use crate::feed::{FeedParser, ParsedFeed, discovery};
use crate::models::{FeedWithMeta, NewArticle, NewFeed};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

use super::read_body_limited;

/// Maximum number of feeds fetched at the same time during "refresh all".
const REFRESH_CONCURRENCY_MAX: usize = 8;

/// Upper bound on a feed document. Larger responses are rejected rather than buffered.
const FEED_BYTES_MAX: usize = 16 * 1024 * 1024;

/// Upper bound on entries stored from a single fetch.
const FEED_ENTRIES_MAX: usize = 1000;

const FEED_FETCH_TIMEOUT: Duration = Duration::from_secs(15);

pub struct FeedService<D: Database> {
    db: Arc<D>,
    http_client: reqwest::Client,
}

// Not derived: `#[derive(Clone)]` would add a needless `D: Clone` bound on the impl. Both fields
// are cheap reference-counted handles.
impl<D: Database> Clone for FeedService<D> {
    fn clone(&self) -> Self {
        Self {
            db: Arc::clone(&self.db),
            http_client: self.http_client.clone(),
        }
    }
}

impl<D: Database> FeedService<D> {
    pub fn new(db: Arc<D>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(FEED_FETCH_TIMEOUT)
            .user_agent("Boke RSS Reader")
            .build()
            .expect("HTTP client config is static and valid");

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

        let parsed = self.fetch_feed(&feed_url).await?;

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
        self.db
            .insert_articles(&new_articles(feed_id, &parsed))
            .await?;

        // The favicon is cosmetic; a missing one must not fail adding the feed.
        if let Some(site_url) = parsed.site_url() {
            match self.fetch_favicon(site_url).await {
                Ok(favicon) => self.db.update_feed_favicon(feed_id, &favicon).await?,
                Err(error) => log::info!("No favicon for {site_url}: {error}"),
            }
        }

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

        let parsed = self.fetch_feed(&feed_url).await?;
        let new_count = self
            .db
            .insert_articles(&new_articles(feed_id, &parsed))
            .await?;
        self.db.update_feed_last_fetched(feed_id).await?;

        Ok(RefreshResult {
            feed_id,
            new_articles: new_count,
            error: None,
        })
    }

    /// Refreshes every feed, at most [`REFRESH_CONCURRENCY_MAX`] at a time.
    ///
    /// A failing feed does not fail the batch; its result carries the error message instead.
    /// The spawned tasks are owned by a local `JoinSet`, so cancelling this future aborts them.
    pub async fn refresh_all_feeds(&self) -> anyhow::Result<Vec<RefreshResult>> {
        let feeds = self.db.get_feeds().await?;
        let mut results = Vec::with_capacity(feeds.len());
        let mut pending_ids = feeds.iter().map(|feed| feed.id);
        let mut tasks = JoinSet::new();

        for feed_id in pending_ids.by_ref().take(REFRESH_CONCURRENCY_MAX) {
            self.spawn_refresh(&mut tasks, feed_id);
        }
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(result) => results.push(result),
                Err(error) => log::error!("Feed refresh task failed: {error}"),
            }
            if let Some(feed_id) = pending_ids.next() {
                self.spawn_refresh(&mut tasks, feed_id);
            }
            assert!(tasks.len() <= REFRESH_CONCURRENCY_MAX);
        }

        Ok(results)
    }

    fn spawn_refresh(&self, tasks: &mut JoinSet<RefreshResult>, feed_id: i64) {
        let service = self.clone();
        tasks.spawn(async move {
            match service.refresh_feed(feed_id).await {
                Ok(result) => result,
                Err(error) => {
                    log::warn!("Failed to refresh feed {feed_id}: {error}");
                    RefreshResult {
                        feed_id,
                        new_articles: 0,
                        error: Some(error.to_string()),
                    }
                }
            }
        });
    }

    async fn fetch_feed(&self, feed_url: &str) -> anyhow::Result<ParsedFeed> {
        let response = self
            .http_client
            .get(feed_url)
            .send()
            .await?
            .error_for_status()?;
        let body = read_body_limited(response, FEED_BYTES_MAX).await?;
        FeedParser::parse(&body, feed_url)
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
                && let Some(href) = element.value().attr("href")
            {
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
    pub new_articles: u64,
    /// Why the refresh failed, when it did.
    pub error: Option<String>,
}

fn new_articles(feed_id: i64, parsed: &ParsedFeed) -> Vec<NewArticle> {
    let entries = parsed.items();
    let mut articles = Vec::with_capacity(entries.len().min(FEED_ENTRIES_MAX));
    for entry in entries.iter().take(FEED_ENTRIES_MAX) {
        articles.push(NewArticle {
            feed_id,
            guid: entry.guid().to_string(),
            title: entry.title.clone(),
            link: (!entry.link.is_empty()).then(|| entry.link.clone()),
            author: entry.author.clone(),
            summary: entry.summary.clone(),
            content: entry.content.clone(),
            image_url: entry.image_url.clone(),
            published_at: entry.published_at(),
        });
    }
    articles
}
