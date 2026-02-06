use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::str::FromStr;

use super::{Database, DbResult, InsertResult};
use crate::models::{Article, ArticleQuery, Feed, FeedWithMeta, Folder, NewArticle, NewFeed};

#[derive(Clone)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(database_url: &str) -> DbResult<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        // Initialize schema
        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &SqlitePool) -> DbResult<()> {
        sqlx::query(SCHEMA).execute(pool).await?;
        Ok(())
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS folders (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    folder_id       INTEGER REFERENCES folders(id) ON DELETE SET NULL,
    feed_url        TEXT NOT NULL UNIQUE,
    site_url        TEXT,
    description     TEXT,
    language        TEXT,
    favicon_url     TEXT,
    last_fetched_at DATETIME,
    last_build_date DATETIME,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS articles (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id      INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid         TEXT NOT NULL,
    title        TEXT NOT NULL,
    link         TEXT,
    author       TEXT,
    summary      TEXT,
    content      TEXT,
    image_url    TEXT,
    published_at DATETIME,
    is_read      INTEGER DEFAULT 0,
    is_favorite  INTEGER DEFAULT 0,
    created_at   DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(feed_id, guid)
);

CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id);
CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_unread ON articles(feed_id, is_read);
CREATE INDEX IF NOT EXISTS idx_articles_favorite ON articles(is_favorite) WHERE is_favorite = 1;

CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    title,
    content,
    content=articles,
    content_rowid=id
);

CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content)
    VALUES ('delete', old.id, old.title, old.content);
END;

CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content)
    VALUES ('delete', old.id, old.title, old.content);
    INSERT INTO articles_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

#[async_trait]
impl Database for SqliteDatabase {
    async fn insert_feed(&self, feed: &NewFeed) -> DbResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO feeds (title, folder_id, feed_url, site_url, description, language, favicon_url, last_build_date)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&feed.title)
        .bind(feed.folder_id)
        .bind(&feed.feed_url)
        .bind(&feed.site_url)
        .bind(&feed.description)
        .bind(&feed.language)
        .bind(&feed.favicon_url)
        .bind(feed.last_build_date)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn get_feed(&self, id: i64) -> DbResult<Option<Feed>> {
        let feed = sqlx::query_as::<_, FeedRow>(
            "SELECT id, title, folder_id, feed_url, site_url, description, language, favicon_url, last_fetched_at, last_build_date, created_at, updated_at FROM feeds WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(feed.map(|f| f.into()))
    }

    async fn get_feeds(&self) -> DbResult<Vec<FeedWithMeta>> {
        let feeds = sqlx::query_as::<_, FeedWithMetaRow>(
            r#"
            SELECT
                f.id, f.title, f.folder_id, f.feed_url, f.site_url, f.description,
                f.language, f.favicon_url, f.last_fetched_at, f.last_build_date,
                (SELECT COUNT(*) FROM articles a WHERE a.feed_id = f.id AND a.is_read = 0) as unread_count
            FROM feeds f
            ORDER BY f.title COLLATE NOCASE
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(feeds.into_iter().map(|f| f.into()).collect())
    }

    async fn delete_feed(&self, id: i64) -> DbResult<()> {
        sqlx::query("DELETE FROM feeds WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_feed_favicon(&self, id: i64, favicon_url: &str) -> DbResult<()> {
        sqlx::query(
            "UPDATE feeds SET favicon_url = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(favicon_url)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_feed_last_fetched(&self, id: i64) -> DbResult<()> {
        sqlx::query(
            "UPDATE feeds SET last_fetched_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_feed_url(&self, id: i64) -> DbResult<Option<String>> {
        let result = sqlx::query_scalar::<_, String>("SELECT feed_url FROM feeds WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    async fn insert_article(&self, article: &NewArticle) -> DbResult<InsertResult> {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO articles (feed_id, guid, title, link, author, summary, content, image_url, published_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(article.feed_id)
        .bind(&article.guid)
        .bind(&article.title)
        .bind(&article.link)
        .bind(&article.author)
        .bind(&article.summary)
        .bind(&article.content)
        .bind(&article.image_url)
        .bind(article.published_at)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(InsertResult::Inserted(result.last_insert_rowid()))
        } else {
            Ok(InsertResult::Ignored)
        }
    }

    async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        let article = sqlx::query_as::<_, ArticleRow>(
            r#"
            SELECT
                a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url
            FROM articles a
            JOIN feeds f ON a.feed_id = f.id
            WHERE a.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(article.map(|a| a.into()))
    }

    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<Article>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT
                a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url
            FROM articles a
            JOIN feeds f ON a.feed_id = f.id
            WHERE 1=1
            "#,
        );

        if let Some(feed_id) = query.feed_id {
            qb.push(" AND a.feed_id = ");
            qb.push_bind(feed_id);
        }

        if query.unread_only {
            qb.push(" AND a.is_read = 0");
        }

        if query.favorites_only {
            qb.push(" AND a.is_favorite = 1");
        }

        qb.push(" ORDER BY a.published_at DESC NULLS LAST, a.created_at DESC");
        qb.push(" LIMIT ");
        qb.push_bind(query.limit);
        qb.push(" OFFSET ");
        qb.push_bind(query.offset);

        let articles = qb
            .build_query_as::<ArticleRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(articles.into_iter().map(|a| a.into()).collect())
    }

    async fn toggle_read(&self, id: i64) -> DbResult<()> {
        sqlx::query(
            "UPDATE articles SET is_read = CASE WHEN is_read = 0 THEN 1 ELSE 0 END WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_all_read(&self, feed_id: Option<i64>) -> DbResult<()> {
        if let Some(fid) = feed_id {
            sqlx::query("UPDATE articles SET is_read = 1 WHERE feed_id = ?")
                .bind(fid)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE articles SET is_read = 1")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn mark_all_unread(&self, feed_id: Option<i64>) -> DbResult<()> {
        if let Some(fid) = feed_id {
            sqlx::query("UPDATE articles SET is_read = 0 WHERE feed_id = ?")
                .bind(fid)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE articles SET is_read = 0")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn toggle_favorite(&self, id: i64) -> DbResult<()> {
        sqlx::query(
            "UPDATE articles SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END WHERE id = ?",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_favorites_count(&self) -> DbResult<i64> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM articles WHERE is_favorite = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<Article>> {
        let articles = sqlx::query_as::<_, ArticleRow>(
            r#"
            SELECT
                a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url
            FROM articles a
            JOIN feeds f ON a.feed_id = f.id
            JOIN articles_fts fts ON a.id = fts.rowid
            WHERE articles_fts MATCH ?
            ORDER BY bm25(articles_fts)
            LIMIT ?
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(articles.into_iter().map(|a| a.into()).collect())
    }

    async fn update_article_content(&self, id: i64, content: &str) -> DbResult<()> {
        sqlx::query("UPDATE articles SET content = ? WHERE id = ?")
            .bind(content)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_article_link(&self, id: i64) -> DbResult<Option<String>> {
        let result = sqlx::query_scalar::<_, String>("SELECT link FROM articles WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    async fn get_folders(&self) -> DbResult<Vec<Folder>> {
        let folders = sqlx::query_as::<_, FolderRow>(
            r#"
            SELECT
                f.id, f.name,
                (SELECT COUNT(*) FROM feeds WHERE folder_id = f.id) as feed_count
            FROM folders f
            ORDER BY f.name COLLATE NOCASE
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(folders.into_iter().map(|f| f.into()).collect())
    }

    async fn create_folder(&self, name: &str) -> DbResult<Folder> {
        let result = sqlx::query("INSERT INTO folders (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(Folder {
            id: result.last_insert_rowid(),
            name: name.to_string(),
            feed_count: 0,
        })
    }

    async fn rename_folder(&self, id: i64, name: &str) -> DbResult<()> {
        sqlx::query("UPDATE folders SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_folder(&self, id: i64) -> DbResult<()> {
        // First set folder_id to NULL for all feeds in this folder
        sqlx::query("UPDATE feeds SET folder_id = NULL WHERE folder_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Then delete the folder
        sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn move_feed_to_folder(&self, feed_id: i64, folder_id: Option<i64>) -> DbResult<()> {
        sqlx::query("UPDATE feeds SET folder_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(folder_id)
            .bind(feed_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// Row types for SQLx
#[derive(sqlx::FromRow)]
struct FeedRow {
    id: i64,
    title: String,
    folder_id: Option<i64>,
    feed_url: String,
    site_url: Option<String>,
    description: Option<String>,
    language: Option<String>,
    favicon_url: Option<String>,
    last_fetched_at: Option<chrono::DateTime<chrono::Utc>>,
    last_build_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<FeedRow> for Feed {
    fn from(row: FeedRow) -> Self {
        Feed {
            id: row.id,
            title: row.title,
            folder_id: row.folder_id,
            feed_url: row.feed_url,
            site_url: row.site_url,
            description: row.description,
            language: row.language,
            favicon_url: row.favicon_url,
            last_fetched_at: row.last_fetched_at,
            last_build_date: row.last_build_date,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FeedWithMetaRow {
    id: i64,
    title: String,
    folder_id: Option<i64>,
    feed_url: String,
    site_url: Option<String>,
    description: Option<String>,
    language: Option<String>,
    favicon_url: Option<String>,
    last_fetched_at: Option<chrono::DateTime<chrono::Utc>>,
    last_build_date: Option<chrono::DateTime<chrono::Utc>>,
    unread_count: i64,
}

impl From<FeedWithMetaRow> for FeedWithMeta {
    fn from(row: FeedWithMetaRow) -> Self {
        FeedWithMeta {
            id: row.id,
            title: row.title,
            folder_id: row.folder_id,
            feed_url: row.feed_url,
            site_url: row.site_url,
            description: row.description,
            language: row.language,
            favicon_url: row.favicon_url,
            last_fetched_at: row.last_fetched_at,
            last_build_date: row.last_build_date,
            unread_count: row.unread_count,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ArticleRow {
    id: i64,
    feed_id: i64,
    guid: String,
    title: String,
    link: Option<String>,
    author: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    image_url: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    is_read: i32,
    is_favorite: i32,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    feed_title: Option<String>,
    feed_favicon_url: Option<String>,
}

impl From<ArticleRow> for Article {
    fn from(row: ArticleRow) -> Self {
        Article {
            id: row.id,
            feed_id: row.feed_id,
            guid: row.guid,
            title: row.title,
            link: row.link,
            author: row.author,
            summary: row.summary,
            content: row.content,
            image_url: row.image_url,
            published_at: row.published_at,
            is_read: row.is_read != 0,
            is_favorite: row.is_favorite != 0,
            created_at: row.created_at,
            feed_title: row.feed_title,
            feed_favicon_url: row.feed_favicon_url,
        }
    }
}

#[derive(sqlx::FromRow)]
struct FolderRow {
    id: i64,
    name: String,
    feed_count: i64,
}

impl From<FolderRow> for Folder {
    fn from(row: FolderRow) -> Self {
        Folder {
            id: row.id,
            name: row.name,
            feed_count: row.feed_count,
        }
    }
}
