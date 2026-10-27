mod pool;

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
mod postgres;

pub use pool::DatabasePool;

use async_trait::async_trait;
use thiserror::Error;

use crate::models::{
    Article, ArticleListItem, ArticleQuery, ArticleScope, Feed, FeedWithMeta, Folder, NewArticle,
    NewFeed,
};

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
    /// Inserts `articles` in one transaction, skipping rows whose `(feed_id, guid)` already
    /// exists. Returns the number of rows actually inserted.
    async fn insert_articles(&self, articles: &[NewArticle]) -> DbResult<u64>;
    async fn get_article(&self, id: i64) -> DbResult<Option<Article>>;
    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<ArticleListItem>>;
    async fn set_read(&self, id: i64, is_read: bool) -> DbResult<()>;
    async fn set_favorite(&self, id: i64, is_favorite: bool) -> DbResult<()>;
    async fn mark_all_read(&self, scope: ArticleScope) -> DbResult<()>;
    async fn mark_all_unread(&self, scope: ArticleScope) -> DbResult<()>;
    async fn get_favorites_count(&self) -> DbResult<i64>;
    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<ArticleListItem>>;
    async fn update_article_content(&self, id: i64, content: &str) -> DbResult<()>;

    // Folder operations
    async fn get_folders(&self) -> DbResult<Vec<Folder>>;
    async fn create_folder(&self, name: &str) -> DbResult<Folder>;
    async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()>;
    async fn delete_folder(&self, id: i64) -> DbResult<()>;
    async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()>;
}
