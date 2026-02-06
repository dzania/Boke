use crate::error::ApiError;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use boke_core::models::ArticleQuery;
use serde::{Deserialize, Serialize};

// Article handlers

#[derive(Deserialize)]
pub struct GetArticlesQuery {
    feed_id: Option<i64>,
    offset: Option<i64>,
    limit: Option<i64>,
    unread_only: Option<bool>,
    favorites_only: Option<bool>,
}

pub async fn get_articles(
    State(state): State<AppState>,
    Query(params): Query<GetArticlesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let query = ArticleQuery {
        feed_id: params.feed_id,
        offset: params.offset.unwrap_or(0),
        limit: params.limit.unwrap_or(50),
        unread_only: params.unread_only.unwrap_or(false),
        favorites_only: params.favorites_only.unwrap_or(false),
    };
    let articles = state.article_service.get_articles(query).await?;
    Ok(Json(articles))
}

pub async fn get_article(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let article = state
        .article_service
        .get_article(id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(article))
}

pub async fn toggle_read(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.article_service.toggle_read(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_favorite(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.article_service.toggle_favorite(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn fetch_article_content(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ContentResponse>, ApiError> {
    let content = state.article_service.fetch_article_content(id).await?;
    Ok(Json(ContentResponse { content }))
}

#[derive(Serialize)]
pub struct ContentResponse {
    content: String,
}

#[derive(Deserialize)]
pub struct MarkAllRequest {
    feed_id: Option<i64>,
}

pub async fn mark_all_read(
    State(state): State<AppState>,
    Json(req): Json<MarkAllRequest>,
) -> Result<StatusCode, ApiError> {
    state.article_service.mark_all_read(req.feed_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn mark_all_unread(
    State(state): State<AppState>,
    Json(req): Json<MarkAllRequest>,
) -> Result<StatusCode, ApiError> {
    state.article_service.mark_all_unread(req.feed_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

pub async fn search_articles(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let articles = state
        .article_service
        .search_articles(&params.q, params.limit.unwrap_or(50))
        .await?;
    Ok(Json(articles))
}

pub async fn get_favorites_count(
    State(state): State<AppState>,
) -> Result<Json<CountResponse>, ApiError> {
    let count = state.article_service.get_favorites_count().await?;
    Ok(Json(CountResponse { count }))
}

#[derive(Serialize)]
pub struct CountResponse {
    count: i64,
}
