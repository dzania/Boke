use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

use crate::commands::articles::resolve_relative_urls;
use crate::feed;
use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, Serialize)]
pub struct FeedWithMeta {
    pub id: i64,
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub favicon_url: Option<String>,
    pub folder_id: Option<i64>,
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
        let base = if entry.link.is_empty() {
            &feed_url
        } else {
            &entry.link
        };
        let content = entry
            .content
            .as_deref()
            .or(entry.summary.as_deref())
            .map(|c| resolve_relative_urls(c, base));
        let summary = entry
            .summary
            .as_deref()
            .map(|s| resolve_relative_urls(s, base));
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(feed_id)
        .bind(&entry.id)
        .bind(&entry.title)
        .bind(&entry.link)
        .bind(&entry.author)
        .bind(&summary)
        .bind(&content)
        .bind(&entry.image_url)
        .bind(&published)
        .execute(pool.inner())
        .await;
    }

    // Fetch favicon
    if let Some(ref site) = site_url {
        if let Some(icon) = fetch_favicon(site).await {
            let _ = sqlx::query("UPDATE feeds SET favicon_url = ? WHERE id = ?")
                .bind(&icon)
                .bind(feed_id)
                .execute(pool.inner())
                .await;
        }
    }

    // Return the feed with metadata
    get_feed_by_id(feed_id, pool.inner()).await
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub added: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_opml(
    path: String,
    pool: State<'_, SqlitePool>,
) -> Result<ImportResult, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let feed_urls = parse_opml(&content)?;

    let mut result = ImportResult {
        added: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .map_err(|e| e.to_string())?;

    for feed_url in feed_urls {
        // Check if already subscribed
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM feeds WHERE feed_url = ?")
            .bind(&feed_url)
            .fetch_optional(pool.inner())
            .await
            .map_err(|e| e.to_string())?;

        if exists.is_some() {
            result.skipped += 1;
            continue;
        }

        // Fetch and parse feed
        let body = match client.get(&feed_url).send().await {
            Ok(resp) => match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    result.errors.push(format!("{feed_url}: {e}"));
                    continue;
                }
            },
            Err(e) => {
                result.errors.push(format!("{feed_url}: {e}"));
                continue;
            }
        };

        let parsed = match feed::parse(&body, &feed_url) {
            Ok(p) => p,
            Err(e) => {
                result.errors.push(format!("{feed_url}: {e}"));
                continue;
            }
        };

        let site_url = if parsed.link.is_empty() {
            None
        } else {
            Some(parsed.link.clone())
        };
        let last_build = parsed.last_updated.map(|d| d.to_rfc3339());

        let feed_id = match sqlx::query_scalar::<_, i64>(
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
        {
            Ok(id) => id,
            Err(e) => {
                result.errors.push(format!("{feed_url}: {e}"));
                continue;
            }
        };

        // Insert articles
        for entry in &parsed.entries {
            let published = entry.published.map(|d| d.to_rfc3339());
            let base = if entry.link.is_empty() {
                &feed_url
            } else {
                &entry.link
            };
            let content = entry
                .content
                .as_deref()
                .or(entry.summary.as_deref())
                .map(|c| resolve_relative_urls(c, base));
            let summary = entry
                .summary
                .as_deref()
                .map(|s| resolve_relative_urls(s, base));
            let _ = sqlx::query(
                "INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(feed_id)
            .bind(&entry.id)
            .bind(&entry.title)
            .bind(&entry.link)
            .bind(&entry.author)
            .bind(&summary)
            .bind(&content)
            .bind(&entry.image_url)
            .bind(&published)
            .execute(pool.inner())
            .await;
        }

        // Fetch favicon
        if let Some(ref site) = site_url {
            if let Some(icon) = fetch_favicon(site).await {
                let _ = sqlx::query("UPDATE feeds SET favicon_url = ? WHERE id = ?")
                    .bind(&icon)
                    .bind(feed_id)
                    .execute(pool.inner())
                    .await;
            }
        }

        result.added += 1;
    }

    Ok(result)
}

fn parse_opml(xml: &str) -> Result<Vec<String>, String> {
    let mut reader = Reader::from_str(xml);
    let mut urls = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.name().as_ref() == b"outline" =>
            {
                let mut xml_url = None;
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"xmlUrl" || attr.key.as_ref() == b"xmlurl" {
                        if let Ok(val) = attr.unescape_value() {
                            let url = val.to_string();
                            if !url.is_empty() {
                                xml_url = Some(url);
                            }
                        }
                    }
                }
                if let Some(url) = xml_url {
                    urls.push(url);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Failed to parse OPML: {e}")),
            _ => {}
        }
        buf.clear();
    }

    if urls.is_empty() {
        return Err("No feeds found in OPML file".to_string());
    }

    Ok(urls)
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
    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>, String, String)>(
        "SELECT f.id, f.title, f.feed_url, f.site_url, f.description, f.language, f.favicon_url, f.folder_id, f.last_fetched_at, f.last_build_date, f.created_at, f.updated_at
         FROM feeds f ORDER BY f.title COLLATE NOCASE"
    )
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let mut feeds = Vec::new();
    for row in rows {
        let unread: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM articles WHERE feed_id = ? AND is_read = 0")
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
            folder_id: row.7,
            last_fetched_at: row.8,
            last_build_date: row.9,
            created_at: row.10,
            updated_at: row.11,
            unread_count: unread,
        });
    }

    Ok(feeds)
}

#[tauri::command]
pub async fn refresh_feed(
    feed_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<RefreshResult, String> {
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
pub async fn refresh_all_feeds(
    pool: State<'_, SqlitePool>,
    app: AppHandle,
) -> Result<Vec<RefreshResult>, String> {
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, feed_url FROM feeds")
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
        let base = if entry.link.is_empty() {
            feed_url
        } else {
            &entry.link
        };
        let content = entry
            .content
            .as_deref()
            .or(entry.summary.as_deref())
            .map(|c| resolve_relative_urls(c, base));
        let summary = entry
            .summary
            .as_deref()
            .map(|s| resolve_relative_urls(s, base));
        let result = sqlx::query(
            "INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(feed_id)
        .bind(&entry.id)
        .bind(&entry.title)
        .bind(&entry.link)
        .bind(&entry.author)
        .bind(&summary)
        .bind(&content)
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

async fn fetch_favicon(site_url: &str) -> Option<String> {
    let parsed = url::Url::parse(site_url).ok()?;
    let origin = format!("{}://{}", parsed.scheme(), parsed.host_str()?);
    let favicon_url = format!("{}/favicon.ico", origin);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .ok()?;

    // Try /favicon.ico first
    let resp = client.head(&favicon_url).send().await.ok()?;
    if resp.status().is_success() {
        return Some(favicon_url);
    }

    // Try parsing HTML for <link rel="icon"> or <link rel="shortcut icon">
    let html = client.get(&origin).send().await.ok()?.text().await.ok()?;
    let doc = scraper::Html::parse_document(&html);
    let selector = scraper::Selector::parse(r#"link[rel~="icon"]"#).ok()?;

    for el in doc.select(&selector) {
        if let Some(href) = el.value().attr("href") {
            let resolved = parsed.join(href).ok()?;
            return Some(resolved.to_string());
        }
    }

    None
}

async fn get_feed_by_id(feed_id: i64, pool: &SqlitePool) -> Result<FeedWithMeta, String> {
    let row = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>, String, String)>(
        "SELECT id, title, feed_url, site_url, description, language, favicon_url, folder_id, last_fetched_at, last_build_date, created_at, updated_at
         FROM feeds WHERE id = ?"
    )
    .bind(feed_id)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let unread: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM articles WHERE feed_id = ? AND is_read = 0")
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
        folder_id: row.7,
        last_fetched_at: row.8,
        last_build_date: row.9,
        created_at: row.10,
        updated_at: row.11,
        unread_count: unread,
    })
}
