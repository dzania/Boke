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

// Aliases for service layer
impl Feed {
    pub fn site_url(&self) -> Option<&str> {
        if self.link.is_empty() {
            None
        } else {
            Some(&self.link)
        }
    }

    pub fn last_build_date(&self) -> Option<DateTime<Utc>> {
        self.last_updated
    }

    pub fn items(&self) -> &[FeedEntry] {
        &self.entries
    }
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

// Aliases for service layer
impl FeedEntry {
    pub fn guid(&self) -> &str {
        &self.id
    }

    pub fn published_at(&self) -> Option<DateTime<Utc>> {
        self.published
    }
}
