use crate::AppState;
use crate::error::ApiError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use axum_extra::extract::Multipart;
use boke_core::models::FeedWithMeta;
use boke_core::parse_opml;
use serde::{Deserialize, Serialize};

// Feed handlers

pub async fn get_feeds(State(state): State<AppState>) -> Result<Json<Vec<FeedWithMeta>>, ApiError> {
    let feeds = state.feed_service.get_feeds().await?;
    Ok(Json(feeds))
}

#[derive(Deserialize)]
pub struct AddFeedRequest {
    url: String,
}

pub async fn add_feed(
    State(state): State<AppState>,
    Json(req): Json<AddFeedRequest>,
) -> Result<Json<FeedWithMeta>, ApiError> {
    let feed = state.feed_service.add_feed(&req.url).await?;
    Ok(Json(feed))
}

pub async fn remove_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.feed_service.remove_feed(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct RefreshResult {
    feed_id: i64,
    new_articles: i64,
}

pub async fn refresh_feed(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<RefreshResult>, ApiError> {
    let result = state.feed_service.refresh_feed(id).await?;
    Ok(Json(RefreshResult {
        feed_id: result.feed_id,
        new_articles: result.new_articles,
    }))
}

pub async fn refresh_all_feeds(
    State(state): State<AppState>,
) -> Result<Json<Vec<RefreshResult>>, ApiError> {
    let results = state.feed_service.refresh_all_feeds().await?;
    Ok(Json(
        results
            .into_iter()
            .map(|r| RefreshResult {
                feed_id: r.feed_id,
                new_articles: r.new_articles,
            })
            .collect(),
    ))
}

#[derive(Serialize)]
pub struct ImportResult {
    pub added: i32,
    pub skipped: i32,
    pub errors: Vec<String>,
}

pub async fn import_opml(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, ApiError> {
    // Extract the file content from the multipart form
    let mut file_content: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?;
            file_content = Some(
                String::from_utf8(bytes.to_vec())
                    .map_err(|e| ApiError::BadRequest(format!("Invalid UTF-8 in file: {}", e)))?,
            );
            break;
        }
    }

    let content =
        file_content.ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    // Parse the OPML content
    let urls = parse_opml(&content)
        .map_err(|e| ApiError::BadRequest(format!("Failed to parse OPML: {}", e)))?;

    // Get existing feeds to check for duplicates
    let existing_feeds = state.feed_service.get_feeds().await?;
    let existing_urls: std::collections::HashSet<_> = existing_feeds
        .iter()
        .map(|f| f.feed_url.to_lowercase())
        .collect();

    let mut added = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    // Add each feed
    for url in urls {
        // Check if already subscribed (case-insensitive)
        if existing_urls.contains(&url.to_lowercase()) {
            skipped += 1;
            continue;
        }

        match state.feed_service.add_feed(&url).await {
            Ok(_) => added += 1,
            Err(e) => {
                errors.push(format!("{}: {}", url, e));
            }
        }
    }

    Ok(Json(ImportResult {
        added,
        skipped,
        errors,
    }))
}
