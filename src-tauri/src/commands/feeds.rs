use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

use crate::feed;

#[derive(Debug, Clone, Serialize)]
pub struct FeedWithMeta {
    pub id: i64,
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub favicon_url: Option<String>,
    pub last_fetched_at: Option<String>,
    pub last_build_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshResult {
    pub feed_id: i64,
    pub new_articles: i64,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn add_feed(url: String, pool: State<'_, SqlitePool>) -> Result<FeedWithMeta, String> {
    // Discover the feed URL
    let discovered = feed::discovery::discover(&url)
        .await
        .map_err(|e| e.to_string())?;

    let feed_url = discovered
        .first()
        .ok_or("No feed found at that URL")?
        .url
        .clone();

    // Fetch the feed XML
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let body = client
        .get(&feed_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    // Parse the feed
    let parsed = feed::parse(&body, &feed_url).map_err(|e| e.to_string())?;

    // Insert feed into DB
    let site_url = if parsed.link.is_empty() {
        None
    } else {
        Some(parsed.link.clone())
    };
    let last_build = parsed.last_updated.map(|d| d.to_rfc3339());

    let feed_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO feeds (title, feed_url, site_url, description, language, last_fetched_at, last_build_date)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
         RETURNING id",
    )
    .bind(&parsed.title)
    .bind(&feed_url)
    .bind(&site_url)
    .bind(&parsed.description)
    .bind(&parsed.language)
    .bind(&last_build)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            "This feed is already subscribed".to_string()
        } else {
            e.to_string()
        }
    })?;

    // Insert articles
    for entry in &parsed.entries {
        let published = entry.published.map(|d| d.to_rfc3339());
        let content = entry.content.as_deref().or(entry.summary.as_deref());
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(feed_id)
        .bind(&entry.id)
        .bind(&entry.title)
        .bind(&entry.link)
        .bind(&entry.author)
        .bind(&entry.summary)
        .bind(content)
        .bind(&entry.image_url)
        .bind(&published)
        .execute(pool.inner())
        .await;
    }

    // Return the feed with metadata
    get_feed_by_id(feed_id, pool.inner()).await
}

#[tauri::command]
pub async fn remove_feed(feed_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("DELETE FROM feeds WHERE id = ?")
        .bind(feed_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_feeds(pool: State<'_, SqlitePool>) -> Result<Vec<FeedWithMeta>, String> {
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String)>(
        "SELECT f.id, f.title, f.feed_url, f.site_url, f.description, f.language, f.favicon_url, f.last_fetched_at, f.last_build_date, f.created_at, f.updated_at
         FROM feeds f ORDER BY f.title COLLATE NOCASE"
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut feeds = Vec::new();
    for row in rows {
        let unread: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM articles WHERE feed_id = ? AND is_read = 0")
            .bind(row.0)
            .fetch_one(pool.inner())
            .await
            .unwrap_or(0);

        feeds.push(FeedWithMeta {
            id: row.0,
            title: row.1,
            feed_url: row.2,
            site_url: row.3,
            description: row.4,
            language: row.5,
            favicon_url: row.6,
            last_fetched_at: row.7,
            last_build_date: row.8,
            created_at: row.9,
            updated_at: row.10,
            unread_count: unread,
        });
    }

    Ok(feeds)
}

#[tauri::command]
pub async fn refresh_feed(feed_id: i64, pool: State<'_, SqlitePool>) -> Result<RefreshResult, String> {
    let feed_url: String = sqlx::query_scalar("SELECT feed_url FROM feeds WHERE id = ?")
        .bind(feed_id)
        .fetch_one(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    match do_refresh(feed_id, &feed_url, pool.inner()).await {
        Ok(count) => Ok(RefreshResult {
            feed_id,
            new_articles: count,
            error: None,
        }),
        Err(e) => Ok(RefreshResult {
            feed_id,
            new_articles: 0,
            error: Some(e),
        }),
    }
}

#[tauri::command]
pub async fn refresh_all_feeds(pool: State<'_, SqlitePool>, app: AppHandle) -> Result<Vec<RefreshResult>, String> {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, feed_url FROM feeds")
            .fetch_all(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

    let pool_ref = pool.inner().clone();
    let mut set = tokio::task::JoinSet::new();

    for (id, url) in rows {
        let pool = pool_ref.clone();
        set.spawn(async move {
            match do_refresh(id, &url, &pool).await {
                Ok(count) => RefreshResult {
                    feed_id: id,
                    new_articles: count,
                    error: None,
                },
                Err(e) => RefreshResult {
                    feed_id: id,
                    new_articles: 0,
                    error: Some(e),
                },
            }
        });
    }

    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(r) = res {
            results.push(r);
        }
    }

    // Emit event for notifications
    let total_new: i64 = results.iter().map(|r| r.new_articles).sum();
    let feeds_with_new = results.iter().filter(|r| r.new_articles > 0).count();
    if total_new > 0 {
        let _ = app.emit(
            "new-articles",
            serde_json::json!({ "total": total_new, "feeds": feeds_with_new }),
        );
    }

    Ok(results)
}

async fn do_refresh(feed_id: i64, feed_url: &str, pool: &SqlitePool) -> Result<i64, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    let body = client
        .get(feed_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let parsed = feed::parse(&body, feed_url).map_err(|e| e.to_string())?;

    let mut new_count: i64 = 0;
    for entry in &parsed.entries {
        let published = entry.published.map(|d| d.to_rfc3339());
        let content = entry.content.as_deref().or(entry.summary.as_deref());
        let result = sqlx::query(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(feed_id)
        .bind(&entry.id)
        .bind(&entry.title)
        .bind(&entry.link)
        .bind(&entry.author)
        .bind(&entry.summary)
        .bind(content)
        .bind(&entry.image_url)
        .bind(&published)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        if result.rows_affected() > 0 {
            new_count += 1;
        }
    }

    // Update last_fetched_at
    sqlx::query("UPDATE feeds SET last_fetched_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(feed_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_count)
}

async fn get_feed_by_id(feed_id: i64, pool: &SqlitePool) -> Result<FeedWithMeta, String> {
    let row = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String)>(
        "SELECT id, title, feed_url, site_url, description, language, favicon_url, last_fetched_at, last_build_date, created_at, updated_at
         FROM feeds WHERE id = ?"
    )
    .bind(feed_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let unread: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM articles WHERE feed_id = ? AND is_read = 0")
        .bind(feed_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Ok(FeedWithMeta {
        id: row.0,
        title: row.1,
        feed_url: row.2,
        site_url: row.3,
        description: row.4,
        language: row.5,
        favicon_url: row.6,
        last_fetched_at: row.7,
        last_build_date: row.8,
        created_at: row.9,
        updated_at: row.10,
        unread_count: unread,
    })
}
