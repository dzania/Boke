mod pool;

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
mod postgres;

pub use pool::DatabasePool;

use async_trait::async_trait;
use thiserror::Error;

use crate::models::{Article, ArticleQuery, Feed, FeedWithMeta, Folder, NewArticle, NewFeed};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid database URL: {0}")]
    InvalidUrl(String),
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertResult {
    Inserted(i64),
    Ignored,
}

#[async_trait]
pub trait Database: Send + Sync + Clone + 'static {
    // Feed operations
    async fn insert_feed(&self, feed: &NewFeed) -> DbResult<i64>;
    async fn get_feed(&self, id: i64) -> DbResult<Option<Feed>>;
    async fn get_feeds(&self) -> DbResult<Vec<FeedWithMeta>>;
    async fn delete_feed(&self, id: i64) -> DbResult<()>;
    async fn update_feed_favicon(&self, id: i64, favicon_url: &str) -> DbResult<()>;
    async fn update_feed_last_fetched(&self, id: i64) -> DbResult<()>;
    async fn get_feed_url(&self, id: i64) -> DbResult<Option<String>>;

    // Article operations
    async fn insert_article(&self, article: &NewArticle) -> DbResult<InsertResult>;
    async fn get_article(&self, id: i64) -> DbResult<Option<Article>>;
    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<Article>>;
    async fn toggle_read(&self, id: i64) -> DbResult<()>;
    async fn mark_all_read(&self, feed_id: Option<i64>) -> DbResult<()>;
    async fn mark_all_unread(&self, feed_id: Option<i64>) -> DbResult<()>;
    async fn toggle_favorite(&self, id: i64) -> DbResult<()>;
    async fn get_favorites_count(&self) -> DbResult<i64>;
    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<Article>>;
    async fn update_article_content(&self, id: i64, content: &str) -> DbResult<()>;
    async fn get_article_link(&self, id: i64) -> DbResult<Option<String>>;

    // Folder operations
    async fn get_folders(&self) -> DbResult<Vec<Folder>>;
    async fn create_folder(&self, name: &str) -> DbResult<Folder>;
    async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()>;
    async fn delete_folder(&self, id: i64) -> DbResult<()>;
    async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()>;
}
