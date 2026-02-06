//! Feed-related Tauri commands.

use boke_core::{
    parse_opml, DatabasePool, FeedService, FeedWithMeta, OpmlError, RefreshResult,
};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn add_feed(
    url: String,
    svc: State<'_, FeedService<DatabasePool>>,
) -> Result<FeedWithMeta, String> {
    svc.add_feed(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_feed(
    feed_id: i64,
    svc: State<'_, FeedService<DatabasePool>>,
) -> Result<(), String> {
    svc.remove_feed(feed_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_feeds(
    svc: State<'_, FeedService<DatabasePool>>,
) -> Result<Vec<FeedWithMeta>, String> {
    svc.get_feeds().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_feed(
    feed_id: i64,
    svc: State<'_, FeedService<DatabasePool>>,
) -> Result<RefreshResult, String> {
    svc.refresh_feed(feed_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_all_feeds(
    svc: State<'_, FeedService<DatabasePool>>,
    app: AppHandle,
) -> Result<Vec<RefreshResult>, String> {
    let results = svc.refresh_all_feeds().await.map_err(|e| e.to_string())?;

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

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportResult {
    pub added: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_opml(
    path: String,
    svc: State<'_, FeedService<DatabasePool>>,
) -> Result<ImportResult, String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let feed_urls = parse_opml(&content).map_err(|e| match e {
        OpmlError::ParseError(msg) => msg,
        OpmlError::NoFeeds => "No feeds found in OPML file".to_string(),
    })?;

    let mut result = ImportResult {
        added: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    // Get existing feeds to check for duplicates
    let existing_feeds = svc.get_feeds().await.map_err(|e| e.to_string())?;
    let existing_urls: std::collections::HashSet<_> =
        existing_feeds.iter().map(|f| f.feed_url.as_str()).collect();

    for feed_url in feed_urls {
        if existing_urls.contains(feed_url.as_str()) {
            result.skipped += 1;
            continue;
        }

        match svc.add_feed(&feed_url).await {
            Ok(_) => result.added += 1,
            Err(e) => result.errors.push(format!("{feed_url}: {e}")),
        }
    }

    Ok(result)
}
