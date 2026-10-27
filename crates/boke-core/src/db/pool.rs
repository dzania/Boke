use crate::db::{Database, DbError, DbResult};
use crate::models::{
    Article, ArticleListItem, ArticleQuery, ArticleScope, Feed, FeedWithMeta, Folder, NewArticle,
    NewFeed,
};
use async_trait::async_trait;

#[cfg(feature = "sqlite")]
use super::sqlite::SqliteDatabase;

#[cfg(feature = "postgres")]
use super::postgres::PostgresDatabase;

#[derive(Clone)]
pub enum DatabasePool {
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteDatabase),
    #[cfg(feature = "postgres")]
    Postgres(PostgresDatabase),
}

impl DatabasePool {
    pub async fn from_url(database_url: &str) -> DbResult<Self> {
        if database_url.starts_with("sqlite:") || database_url.ends_with(".db") {
            #[cfg(feature = "sqlite")]
            {
                let db = SqliteDatabase::new(database_url).await?;
                return Ok(Self::Sqlite(db));
            }
            #[cfg(not(feature = "sqlite"))]
            {
                return Err(DbError::InvalidUrl(
                    "SQLite support not compiled in".to_string(),
                ));
            }
        } else if database_url.starts_with("postgres://")
            || database_url.starts_with("postgresql://")
        {
            #[cfg(feature = "postgres")]
            {
                let db = PostgresDatabase::new(database_url).await?;
                return Ok(Self::Postgres(db));
            }
            #[cfg(not(feature = "postgres"))]
            {
                return Err(DbError::InvalidUrl(
                    "PostgreSQL support not compiled in".to_string(),
                ));
            }
        }

        Err(DbError::InvalidUrl(format!(
            "Unknown database URL scheme: {}",
            database_url
        )))
    }
}

/// Forwards a call to whichever backend this pool wraps.
macro_rules! dispatch {
    ($self:ident, $db:ident => $call:expr) => {
        match $self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite($db) => $call,
            #[cfg(feature = "postgres")]
            Self::Postgres($db) => $call,
        }
    };
}

#[async_trait]
impl Database for DatabasePool {
    async fn insert_feed(&self, feed: &NewFeed) -> DbResult<i64> {
        dispatch!(self, db => db.insert_feed(feed).await)
    }

    async fn get_feed(&self, id: i64) -> DbResult<Option<Feed>> {
        dispatch!(self, db => db.get_feed(id).await)
    }

    async fn get_feeds(&self) -> DbResult<Vec<FeedWithMeta>> {
        dispatch!(self, db => db.get_feeds().await)
    }

    async fn delete_feed(&self, id: i64) -> DbResult<()> {
        dispatch!(self, db => db.delete_feed(id).await)
    }

    async fn update_feed_favicon(&self, id: i64, favicon_url: &str) -> DbResult<()> {
        dispatch!(self, db => db.update_feed_favicon(id, favicon_url).await)
    }

    async fn update_feed_last_fetched(&self, id: i64) -> DbResult<()> {
        dispatch!(self, db => db.update_feed_last_fetched(id).await)
    }

    async fn get_feed_url(&self, id: i64) -> DbResult<Option<String>> {
        dispatch!(self, db => db.get_feed_url(id).await)
    }

    async fn insert_articles(&self, articles: &[NewArticle]) -> DbResult<u64> {
        dispatch!(self, db => db.insert_articles(articles).await)
    }

    async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        dispatch!(self, db => db.get_article(id).await)
    }

    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<ArticleListItem>> {
        dispatch!(self, db => db.get_articles(query).await)
    }

    async fn set_read(&self, id: i64, is_read: bool) -> DbResult<()> {
        dispatch!(self, db => db.set_read(id, is_read).await)
    }

    async fn set_favorite(&self, id: i64, is_favorite: bool) -> DbResult<()> {
        dispatch!(self, db => db.set_favorite(id, is_favorite).await)
    }

    async fn mark_all_read(&self, scope: ArticleScope) -> DbResult<()> {
        dispatch!(self, db => db.mark_all_read(scope).await)
    }

    async fn mark_all_unread(&self, scope: ArticleScope) -> DbResult<()> {
        dispatch!(self, db => db.mark_all_unread(scope).await)
    }

    async fn get_favorites_count(&self) -> DbResult<i64> {
        dispatch!(self, db => db.get_favorites_count().await)
    }

    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<ArticleListItem>> {
        dispatch!(self, db => db.search_articles(query, limit).await)
    }

    async fn update_article_content(&self, id: i64, content: &str) -> DbResult<()> {
        dispatch!(self, db => db.update_article_content(id, content).await)
    }

    async fn get_folders(&self) -> DbResult<Vec<Folder>> {
        dispatch!(self, db => db.get_folders().await)
    }

    async fn create_folder(&self, name: &str) -> DbResult<Folder> {
        dispatch!(self, db => db.create_folder(name).await)
    }

    async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()> {
        dispatch!(self, db => db.rename_folder(id, name).await)
    }

    async fn delete_folder(&self, id: i64) -> DbResult<()> {
        dispatch!(self, db => db.delete_folder(id).await)
    }

    async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()> {
        dispatch!(self, db => db.move_feed_to_folder(feed_id, folder_id).await)
    }
}
