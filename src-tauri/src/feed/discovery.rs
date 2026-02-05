use scraper::{Html, Selector};
use url::Url;

use super::error::FeedError;

#[derive(Debug, Clone)]
pub struct DiscoveredFeed {
    pub url: String,
}

const COMMON_FEED_PATHS: &[&str] = &[
    "/feed",
    "/rss",
    "/rss.xml",
    "/feed.xml",
    "/atom.xml",
    "/index.xml",
    "/blog/feed",
    "/blog/rss",
    "/?feed=rss2",
];

/// Discover feed URLs from a given URL.
/// If the URL itself is a feed, returns it directly.
/// If it's HTML, looks for <link rel="alternate"> tags and tries common paths.
pub async fn discover(url: &str) -> Result<Vec<DiscoveredFeed>, FeedError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Boke RSS Reader/0.1")
        .build()
        .map_err(FeedError::Http)?;

    let response = client.get(url).send().await.map_err(FeedError::Http)?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let body = response.text().await.map_err(FeedError::Http)?;

    // If the URL is already a feed (XML/RSS/Atom content type)
    if is_feed_content_type(&content_type) || looks_like_feed(&body) {
        return Ok(vec![DiscoveredFeed {
            url: url.to_string(),
        }]);
    }

    // Parse as HTML and look for feed links
    let base_url = Url::parse(url).map_err(|e| FeedError::Discovery(e.to_string()))?;

    // Scope the HTML parsing so `document` (not Send) is dropped before any await
    let mut feeds = {
        let document = Html::parse_document(&body);
        let selector = Selector::parse(r#"link[rel="alternate"]"#).unwrap();
        let mut found = Vec::new();

        for element in document.select(&selector) {
            let link_type = element.value().attr("type").unwrap_or("");
            let href = element.value().attr("href").unwrap_or("");

            if (link_type.contains("rss") || link_type.contains("atom") || link_type.contains("feed"))
                && !href.is_empty()
            {
                let resolved = resolve_url(&base_url, href);
                found.push(DiscoveredFeed {
                    url: resolved,
                });
            }
        }
        found
    };

    if !feeds.is_empty() {
        return Ok(feeds);
    }

    // Try common feed paths
    for path in COMMON_FEED_PATHS {
        let candidate = resolve_url(&base_url, path);
        if let Ok(resp) = client.head(&candidate).send().await {
            if resp.status().is_success() {
                let ct = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_lowercase();
                if is_feed_content_type(&ct) {
                    feeds.push(DiscoveredFeed {
                        url: candidate,
                    });
                    return Ok(feeds);
                }
            }
        }
    }

    if feeds.is_empty() {
        Err(FeedError::Discovery(format!(
            "No feed found at {url}. Try providing the direct feed URL."
        )))
    } else {
        Ok(feeds)
    }
}

fn is_feed_content_type(ct: &str) -> bool {
    ct.contains("xml")
        || ct.contains("rss")
        || ct.contains("atom")
        || ct.contains("feed")
}

fn looks_like_feed(body: &str) -> bool {
    let trimmed = body.trim_start();
    trimmed.starts_with("<?xml")
        || trimmed.starts_with("<rss")
        || trimmed.starts_with("<feed")
        || trimmed.starts_with("<rdf:RDF")
}

fn resolve_url(base: &Url, href: &str) -> String {
    base.join(href)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| href.to_string())
}
