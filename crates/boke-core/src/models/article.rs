use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A fully loaded article, including its stored HTML body.
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
    pub feed_site_url: Option<String>,
}

/// Lightweight article projection used by list and search views.
///
/// It deliberately omits `summary` and `content`: those can be hundreds of kilobytes per row and
/// list views never render them. The reader loads the body separately by id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleListItem {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub link: Option<String>,
    pub author: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub created_at: Option<DateTime<Utc>>,
    // Joined fields
    pub feed_title: Option<String>,
    pub feed_favicon_url: Option<String>,
    pub feed_site_url: Option<String>,
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

/// The set of articles a bulk read-state change applies to.
///
/// `feed_id: None` means every feed. `favorites_only` narrows the set to favourites, so that
/// "mark all read" in the Favourites view does not touch the rest of the database.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ArticleScope {
    pub feed_id: Option<i64>,
    #[serde(default)]
    pub favorites_only: bool,
}
