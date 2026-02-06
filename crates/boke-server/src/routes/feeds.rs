use crate::error::ApiError;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use boke_core::models::FeedWithMeta;
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
