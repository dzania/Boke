use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub title: String,
    pub folder_id: Option<i64>,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub favicon_url: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub last_build_date: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedWithMeta {
    pub id: i64,
    pub title: String,
    pub folder_id: Option<i64>,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub favicon_url: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub last_build_date: Option<DateTime<Utc>>,
    pub unread_count: i64,
}

#[derive(Debug, Clone)]
pub struct NewFeed {
    pub title: String,
    pub folder_id: Option<i64>,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub favicon_url: Option<String>,
    pub last_build_date: Option<DateTime<Utc>>,
}
