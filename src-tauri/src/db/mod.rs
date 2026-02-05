use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
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

CREATE TABLE IF NOT EXISTS tags (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS feed_tags (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (feed_id, tag_id)
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

pub async fn init(db_path: &std::path::Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::raw_sql(SCHEMA).execute(&pool).await?;

    Ok(pool)
}
