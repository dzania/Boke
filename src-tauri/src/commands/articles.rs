//! Article-related Tauri commands.

use boke_core::{Article, ArticleQuery, ArticleService, DatabasePool};
use tauri::State;

#[tauri::command]
pub async fn get_articles(
    feed_id: Option<i64>,
    offset: i64,
    limit: i64,
    unread_only: bool,
    favorites_only: bool,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<Vec<Article>, String> {
    let query = ArticleQuery {
        feed_id,
        offset,
        limit,
        unread_only,
        favorites_only,
    };
    svc.get_articles(query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article(
    article_id: i64,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<Article, String> {
    svc.get_article(article_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Article not found".to_string())
}

#[tauri::command]
pub async fn toggle_read(
    article_id: i64,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<(), String> {
    svc.toggle_read(article_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_all_read(
    feed_id: Option<i64>,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<(), String> {
    svc.mark_all_read(feed_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_all_unread(
    feed_id: Option<i64>,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<(), String> {
    svc.mark_all_unread(feed_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_favorite(
    article_id: i64,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<(), String> {
    svc.toggle_favorite(article_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites_count(
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<i64, String> {
    svc.get_favorites_count().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_articles(
    query: String,
    limit: i64,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<Vec<Article>, String> {
    svc.search_articles(&query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_article_content(
    article_id: i64,
    svc: State<'_, ArticleService<DatabasePool>>,
) -> Result<String, String> {
    svc.fetch_article_content(article_id)
        .await
        .map_err(|e| e.to_string())
}
