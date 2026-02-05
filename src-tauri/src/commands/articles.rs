use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use tauri::State;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub created_at: String,
    pub feed_title: Option<String>,
}

const ARTICLE_COLUMNS: &str = "\
    a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content, \
    a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at, \
    f.title AS feed_title";

#[tauri::command]
pub async fn get_articles(
    feed_id: Option<i64>,
    offset: i64,
    limit: i64,
    unread_only: bool,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Article>, String> {
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a JOIN feeds f ON a.feed_id = f.id WHERE 1=1"
    ));

    if let Some(fid) = feed_id {
        qb.push(" AND a.feed_id = ").push_bind(fid);
    }
    if unread_only {
        qb.push(" AND a.is_read = 0");
    }

    qb.push(" ORDER BY a.published_at DESC NULLS LAST, a.created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    qb.build_query_as::<Article>()
        .fetch_all(pool.inner())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_article(article_id: i64, pool: State<'_, SqlitePool>) -> Result<Article, String> {
    sqlx::query_as::<_, Article>(&format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a JOIN feeds f ON a.feed_id = f.id WHERE a.id = ?"
    ))
    .bind(article_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_read(article_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("UPDATE articles SET is_read = CASE WHEN is_read = 0 THEN 1 ELSE 0 END WHERE id = ?")
        .bind(article_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn mark_all_read(feed_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("UPDATE articles SET is_read = 1 WHERE feed_id = ? AND is_read = 0")
        .bind(feed_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_favorite(article_id: i64, pool: State<'_, SqlitePool>) -> Result<(), String> {
    sqlx::query("UPDATE articles SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END WHERE id = ?")
        .bind(article_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_articles(query: String, limit: i64, pool: State<'_, SqlitePool>) -> Result<Vec<Article>, String> {
    sqlx::query_as::<_, Article>(&format!(
        "SELECT {ARTICLE_COLUMNS} FROM articles a \
         JOIN articles_fts fts ON a.id = fts.rowid \
         JOIN feeds f ON a.feed_id = f.id \
         WHERE articles_fts MATCH ? \
         ORDER BY bm25(articles_fts) \
         LIMIT ?"
    ))
    .bind(&query)
    .bind(limit)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())
}
