use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub title: String,
    pub link: String,
    pub feed_url: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
    pub entries: Vec<FeedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    pub id: String,
    pub title: String,
    pub link: String,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub categories: Vec<String>,
    pub image_url: Option<String>,
}
