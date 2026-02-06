use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub published_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub created_at: Option<DateTime<Utc>>,
    // Joined fields
    pub feed_title: Option<String>,
    pub feed_favicon_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewArticle {
    pub feed_id: i64,
    pub guid: String,
    pub title: String,
    pub link: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct ArticleQuery {
    pub feed_id: Option<i64>,
    pub offset: i64,
    pub limit: i64,
    pub unread_only: bool,
    pub favorites_only: bool,
}
