use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::str::FromStr;

use super::{Database, DbResult};
use crate::models::{
    Article, ArticleListItem, ArticleQuery, ArticleScope, Feed, FeedWithMeta, Folder, NewArticle,
    NewFeed,
};

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
CREATE INDEX IF NOT EXISTS idx_articles_feed_published ON articles(feed_id, published_at DESC);

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

-- Only re-index when indexed columns change. Older databases had an unconditional trigger that
-- rewrote the FTS rows on every read/favourite toggle, so drop and recreate it.
DROP TRIGGER IF EXISTS articles_au;
CREATE TRIGGER articles_au AFTER UPDATE OF title, content ON articles BEGIN
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
                f.created_at, f.updated_at,
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

    async fn insert_articles(&self, articles: &[NewArticle]) -> DbResult<u64> {
        let mut tx = self.pool.begin().await?;
        let mut inserted_count: u64 = 0;
        for article in articles {
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
            .execute(&mut *tx)
            .await?;
            inserted_count += result.rows_affected();
        }
        tx.commit().await?;
        assert!(inserted_count <= articles.len() as u64);
        Ok(inserted_count)
    }

    async fn get_article(&self, id: i64) -> DbResult<Option<Article>> {
        let article = sqlx::query_as::<_, ArticleRow>(
            r#"
            SELECT
                a.id, a.feed_id, a.guid, a.title, a.link, a.author, a.summary, a.content,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url,
                f.site_url as feed_site_url
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

    async fn get_articles(&self, query: &ArticleQuery) -> DbResult<Vec<ArticleListItem>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT
                a.id, a.feed_id, a.title, a.link, a.author,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url,
                f.site_url as feed_site_url
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

        // `a.id` makes the order total, so offset pages never overlap or skip rows on ties.
        qb.push(" ORDER BY a.published_at DESC NULLS LAST, a.created_at DESC, a.id DESC");
        qb.push(" LIMIT ");
        qb.push_bind(query.limit);
        qb.push(" OFFSET ");
        qb.push_bind(query.offset);

        let articles = qb
            .build_query_as::<ArticleListRow>()
            .fetch_all(&self.pool)
            .await?;

        Ok(articles.into_iter().map(|a| a.into()).collect())
    }

    async fn set_read(&self, id: i64, is_read: bool) -> DbResult<()> {
        sqlx::query("UPDATE articles SET is_read = ? WHERE id = ? AND is_read != ?")
            .bind(is_read)
            .bind(id)
            .bind(is_read)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_favorite(&self, id: i64, is_favorite: bool) -> DbResult<()> {
        sqlx::query("UPDATE articles SET is_favorite = ? WHERE id = ? AND is_favorite != ?")
            .bind(is_favorite)
            .bind(id)
            .bind(is_favorite)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_all_read(&self, scope: ArticleScope) -> DbResult<()> {
        set_read_in_scope(&self.pool, scope, true).await
    }

    async fn mark_all_unread(&self, scope: ArticleScope) -> DbResult<()> {
        set_read_in_scope(&self.pool, scope, false).await
    }

    async fn get_favorites_count(&self) -> DbResult<i64> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM articles WHERE is_favorite = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }

    async fn search_articles(&self, query: &str, limit: i64) -> DbResult<Vec<ArticleListItem>> {
        let Some(match_expression) = fts5_match_expression(query) else {
            return Ok(Vec::new());
        };
        let articles = sqlx::query_as::<_, ArticleListRow>(
            r#"
            SELECT
                a.id, a.feed_id, a.title, a.link, a.author,
                a.image_url, a.published_at, a.is_read, a.is_favorite, a.created_at,
                f.title as feed_title, f.favicon_url as feed_favicon_url,
                f.site_url as feed_site_url
            FROM articles a
            JOIN feeds f ON a.feed_id = f.id
            JOIN articles_fts fts ON a.id = fts.rowid
            WHERE articles_fts MATCH ?
            ORDER BY bm25(articles_fts)
            LIMIT ?
            "#,
        )
        .bind(match_expression)
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

/// Sets `is_read` on every article in `scope` whose flag differs. The `is_read != ?` guard keeps
/// the write set to rows that actually change.
async fn set_read_in_scope(pool: &SqlitePool, scope: ArticleScope, is_read: bool) -> DbResult<()> {
    let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE articles SET is_read = ");
    qb.push_bind(is_read);
    qb.push(" WHERE is_read != ");
    qb.push_bind(is_read);
    if let Some(feed_id) = scope.feed_id {
        qb.push(" AND feed_id = ");
        qb.push_bind(feed_id);
    }
    if scope.favorites_only {
        qb.push(" AND is_favorite = 1");
    }
    qb.build().execute(pool).await?;
    Ok(())
}

/// Maximum number of search terms passed to FTS5. Longer queries are truncated.
const SEARCH_TERMS_MAX: usize = 16;

/// Turns free-form user input into a safe FTS5 `MATCH` expression.
///
/// Each whitespace-separated term becomes a quoted string with a prefix wildcard, so that FTS5
/// operators and punctuation in user input (`c++`, `foo-bar`, a stray `"`) are matched literally
/// instead of being parsed as query syntax. Returns `None` when the input has no terms.
fn fts5_match_expression(query: &str) -> Option<String> {
    let mut expression = String::with_capacity(query.len() + 16);
    for term in query.split_whitespace().take(SEARCH_TERMS_MAX) {
        if !expression.is_empty() {
            expression.push(' ');
        }
        expression.push('"');
        for character in term.chars() {
            if character == '"' {
                expression.push('"');
            }
            expression.push(character);
        }
        expression.push_str("\"*");
    }
    if expression.is_empty() {
        None
    } else {
        Some(expression)
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
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
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
            created_at: row.created_at,
            updated_at: row.updated_at,
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
    feed_site_url: Option<String>,
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
            feed_site_url: row.feed_site_url,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ArticleListRow {
    id: i64,
    feed_id: i64,
    title: String,
    link: Option<String>,
    author: Option<String>,
    image_url: Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    is_read: i32,
    is_favorite: i32,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    feed_title: Option<String>,
    feed_favicon_url: Option<String>,
    feed_site_url: Option<String>,
}

impl From<ArticleListRow> for ArticleListItem {
    fn from(row: ArticleListRow) -> Self {
        ArticleListItem {
            id: row.id,
            feed_id: row.feed_id,
            title: row.title,
            link: row.link,
            author: row.author,
            image_url: row.image_url,
            published_at: row.published_at,
            is_read: row.is_read != 0,
            is_favorite: row.is_favorite != 0,
            created_at: row.created_at,
            feed_title: row.feed_title,
            feed_favicon_url: row.feed_favicon_url,
            feed_site_url: row.feed_site_url,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Each test gets its own database file. An in-memory URL would give every pooled connection
    /// a separate empty database.
    async fn test_db() -> SqliteDatabase {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "boke-test-{}-{}.db",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        for suffix in ["", "-wal", "-shm"] {
            let mut stale = path.clone().into_os_string();
            stale.push(suffix);
            // A leftover file from an earlier run is the only expected failure; NotFound is fine.
            let _ = std::fs::remove_file(stale);
        }
        SqliteDatabase::new(&format!("sqlite://{}", path.display()))
            .await
            .expect("test database opens")
    }

    async fn insert_feed(db: &SqliteDatabase, url: &str) -> i64 {
        db.insert_feed(&NewFeed {
            title: url.to_string(),
            folder_id: None,
            feed_url: url.to_string(),
            site_url: Some("https://example.com/".to_string()),
            description: None,
            language: None,
            favicon_url: None,
            last_build_date: None,
        })
        .await
        .expect("feed inserts")
    }

    fn article(feed_id: i64, guid: &str, title: &str) -> NewArticle {
        NewArticle {
            feed_id,
            guid: guid.to_string(),
            title: title.to_string(),
            link: Some(format!("https://example.com/{guid}")),
            author: None,
            summary: None,
            content: Some(format!("<p>{title} body</p>")),
            image_url: None,
            published_at: None,
        }
    }

    fn list_query(feed_id: Option<i64>, offset: i64, limit: i64) -> ArticleQuery {
        ArticleQuery {
            feed_id,
            offset,
            limit,
            unread_only: false,
            favorites_only: false,
        }
    }

    #[tokio::test]
    async fn insert_articles_counts_only_new_rows() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        let batch = [article(feed_id, "a", "A"), article(feed_id, "b", "B")];

        assert_eq!(db.insert_articles(&batch).await.expect("insert"), 2);
        assert_eq!(db.insert_articles(&batch).await.expect("insert"), 0);
        assert_eq!(db.insert_articles(&[]).await.expect("insert"), 0);
    }

    #[tokio::test]
    async fn list_items_carry_feed_site_url() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        db.insert_articles(&[article(feed_id, "a", "A")])
            .await
            .expect("insert");

        let items = db
            .get_articles(&list_query(None, 0, 10))
            .await
            .expect("list");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].feed_site_url.as_deref(),
            Some("https://example.com/")
        );
    }

    #[tokio::test]
    async fn pages_do_not_overlap() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        let batch: Vec<_> = (0..7)
            .map(|index| article(feed_id, &index.to_string(), "Same"))
            .collect();
        db.insert_articles(&batch).await.expect("insert");

        let mut seen = Vec::new();
        for offset in [0, 3, 6] {
            let page = db
                .get_articles(&list_query(Some(feed_id), offset, 3))
                .await
                .expect("page");
            seen.extend(page.into_iter().map(|item| item.id));
        }
        let mut unique = seen.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(seen.len(), 7);
        assert_eq!(unique.len(), 7);
    }

    #[tokio::test]
    async fn set_read_and_favorite_are_idempotent() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        db.insert_articles(&[article(feed_id, "a", "A")])
            .await
            .expect("insert");
        let id = db
            .get_articles(&list_query(None, 0, 1))
            .await
            .expect("list")[0]
            .id;

        for _ in 0..2 {
            db.set_read(id, true).await.expect("set read");
            db.set_favorite(id, true).await.expect("set favorite");
        }
        let article = db.get_article(id).await.expect("get").expect("exists");
        assert!(article.is_read);
        assert!(article.is_favorite);

        db.set_read(id, false).await.expect("set unread");
        let article = db.get_article(id).await.expect("get").expect("exists");
        assert!(!article.is_read);
    }

    #[tokio::test]
    async fn search_survives_read_toggles() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        db.insert_articles(&[article(feed_id, "a", "Rustacean news")])
            .await
            .expect("insert");
        let id = db
            .get_articles(&list_query(None, 0, 1))
            .await
            .expect("list")[0]
            .id;

        db.set_read(id, true).await.expect("read");
        db.set_favorite(id, true).await.expect("favorite");
        db.mark_all_unread(ArticleScope::default())
            .await
            .expect("mark unread");

        let hits = db.search_articles("rustac", 10).await.expect("search");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, id);
    }

    #[tokio::test]
    async fn search_after_content_update_uses_new_content() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        db.insert_articles(&[article(feed_id, "a", "Title")])
            .await
            .expect("insert");
        let id = db
            .get_articles(&list_query(None, 0, 1))
            .await
            .expect("list")[0]
            .id;

        db.update_article_content(id, "<p>zeppelin</p>")
            .await
            .expect("update");
        assert_eq!(
            db.search_articles("zeppelin", 10)
                .await
                .expect("search")
                .len(),
            1
        );
        assert!(
            db.search_articles("body", 10)
                .await
                .expect("search")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn search_accepts_fts_syntax_characters() {
        let db = test_db().await;
        let feed_id = insert_feed(&db, "https://example.com/feed").await;
        db.insert_articles(&[article(feed_id, "a", "Modern c++ tips")])
            .await
            .expect("insert");

        for query in ["c++", "\"", "foo-bar", "AND", "title:x", "(", "*"] {
            db.search_articles(query, 10)
                .await
                .unwrap_or_else(|error| panic!("query {query:?} failed: {error}"));
        }
        assert!(
            db.search_articles("   ", 10)
                .await
                .expect("empty")
                .is_empty()
        );
        assert_eq!(
            db.search_articles("modern tips", 10)
                .await
                .expect("search")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn mark_all_read_respects_scope() {
        let db = test_db().await;
        let feed_a = insert_feed(&db, "https://a.example.com/feed").await;
        let feed_b = insert_feed(&db, "https://b.example.com/feed").await;
        db.insert_articles(&[article(feed_a, "1", "A1"), article(feed_a, "2", "A2")])
            .await
            .expect("insert");
        db.insert_articles(&[article(feed_b, "1", "B1")])
            .await
            .expect("insert");
        let favourite_id = db
            .get_articles(&list_query(Some(feed_a), 0, 1))
            .await
            .expect("list")[0]
            .id;
        db.set_favorite(favourite_id, true).await.expect("favorite");

        db.mark_all_read(ArticleScope {
            feed_id: None,
            favorites_only: true,
        })
        .await
        .expect("mark favourites read");
        let unread = |items: Vec<ArticleListItem>| items.iter().filter(|i| !i.is_read).count();
        assert_eq!(
            unread(
                db.get_articles(&list_query(None, 0, 10))
                    .await
                    .expect("list")
            ),
            2
        );

        db.mark_all_read(ArticleScope {
            feed_id: Some(feed_b),
            favorites_only: false,
        })
        .await
        .expect("mark feed b read");
        assert_eq!(
            unread(
                db.get_articles(&list_query(None, 0, 10))
                    .await
                    .expect("list")
            ),
            1
        );
        assert_eq!(
            unread(
                db.get_articles(&list_query(Some(feed_a), 0, 10))
                    .await
                    .expect("list")
            ),
            1
        );
    }

    #[test]
    fn fts5_match_expression_quotes_terms() {
        assert_eq!(fts5_match_expression(""), None);
        assert_eq!(fts5_match_expression(" \t"), None);
        assert_eq!(fts5_match_expression("c++").as_deref(), Some("\"c++\"*"));
        assert_eq!(
            fts5_match_expression("say \"hi\"").as_deref(),
            Some("\"say\"* \"\"\"hi\"\"\"*")
        );
        let many = "x ".repeat(SEARCH_TERMS_MAX + 5);
        let expression = fts5_match_expression(&many).expect("terms");
        assert_eq!(expression.matches('*').count(), SEARCH_TERMS_MAX);
    }
}
