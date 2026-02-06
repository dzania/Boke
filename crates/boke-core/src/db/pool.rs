use crate::db::{Database, DbError, DbResult, InsertResult};
use crate::models::{Article, ArticleQuery, Feed, FeedWithMeta, Folder, NewArticle, NewFeed};
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

#[async_trait]
impl Database for DatabasePool {
    async fn insert_feed(&self, feed: &NewFeed) -> DbResult<i64> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.insert_feed(feed).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.insert_feed(feed).await,
        }
    }

    async fn get_feed(&self, id: i64) -> DbResult<Option<Feed>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_feed(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_feed(id).await,
        }
    }

    async fn get_feeds(&self) -> DbResult<Vec<FeedWithMeta>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_feeds().await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_feeds().await,
        }
    }

    async fn delete_feed(&self, id: i64) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.delete_feed(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.delete_feed(id).await,
        }
    }

    async fn update_feed_favicon(&self, id: i64, favicon_url: &str) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.update_feed_favicon(id, favicon_url).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.update_feed_favicon(id, favicon_url).await,
        }
    }

    async fn update_feed_last_fetched(&self, id: i64) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.update_feed_last_fetched(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.update_feed_last_fetched(id).await,
        }
    }

    async fn get_feed_url(&self, id: i64) -> DbResult<Option<String>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_feed_url(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_feed_url(id).await,
        }
    }

    async fn insert_article(&self, article: &NewArticle) -> DbResult<InsertResult> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.insert_article(article).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.insert_article(article).await,
        }
    }

    async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_article(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_article(id).await,
        }
    }

    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<Article>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_articles(query).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_articles(query).await,
        }
    }

    async fn toggle_read(&self, id: i64) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.toggle_read(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.toggle_read(id).await,
        }
    }

    async fn mark_all_read(&self, feed_id: Option<i64>) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.mark_all_read(feed_id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.mark_all_read(feed_id).await,
        }
    }

    async fn mark_all_unread(&self, feed_id: Option<i64>) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.mark_all_unread(feed_id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.mark_all_unread(feed_id).await,
        }
    }

    async fn toggle_favorite(&self, id: i64) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.toggle_favorite(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.toggle_favorite(id).await,
        }
    }

    async fn get_favorites_count(&self) -> DbResult<i64> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_favorites_count().await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_favorites_count().await,
        }
    }

    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<Article>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.search_articles(query, limit).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.search_articles(query, limit).await,
        }
    }

    async fn update_article_content(&self, id: i64, content: &str) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.update_article_content(id, content).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.update_article_content(id, content).await,
        }
    }

    async fn get_article_link(&self, id: i64) -> DbResult<Option<String>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_article_link(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_article_link(id).await,
        }
    }

    async fn get_folders(&self) -> DbResult<Vec<Folder>> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.get_folders().await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.get_folders().await,
        }
    }

    async fn create_folder(&self, name: &str) -> DbResult<Folder> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.create_folder(name).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.create_folder(name).await,
        }
    }

    async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.rename_folder(id, name).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.rename_folder(id, name).await,
        }
    }

    async fn delete_folder(&self, id: i64) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.delete_folder(id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.delete_folder(id).await,
        }
    }

    async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()> {
        match self {
            #[cfg(feature = "sqlite")]
            Self::Sqlite(db) => db.move_feed_to_folder(feed_id, folder_id).await,
            #[cfg(feature = "postgres")]
            Self::Postgres(db) => db.move_feed_to_folder(feed_id, folder_id).await,
        }
    }
}
