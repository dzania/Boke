pub mod atom;
pub mod date;
pub mod detector;
pub mod discovery;
pub mod error;
pub mod model;
pub mod rss1;
pub mod rss2;

use detector::FeedFormat;
use model::Feed;

pub use discovery::{discover, DiscoveredFeed};
pub use error::FeedError;
pub use model::{Feed as ParsedFeed, FeedEntry as ParsedFeedEntry};

pub struct FeedParser;

impl FeedParser {
    /// Parse XML string into a Feed, auto-detecting the format.
    pub fn parse(xml: &str, feed_url: &str) -> Result<ParsedFeed, anyhow::Error> {
        let xml_bytes = xml.as_bytes();
        let format = detector::detect_format(xml_bytes)
            .ok_or_else(|| anyhow::anyhow!("Unknown feed format"))?;

        let feed = match format {
            FeedFormat::Rss2 => rss2::parse(xml_bytes, feed_url),
            FeedFormat::Rss1 => rss1::parse(xml_bytes, feed_url),
            FeedFormat::Atom => atom::parse(xml_bytes, feed_url),
        }?;

        Ok(feed)
    }
}

/// Parse XML bytes into a Feed, auto-detecting the format.
pub fn parse(xml: &[u8], feed_url: &str) -> Result<Feed, FeedError> {
    let format = detector::detect_format(xml).ok_or(FeedError::UnknownFormat)?;

    match format {
        FeedFormat::Rss2 => rss2::parse(xml, feed_url),
        FeedFormat::Rss1 => rss1::parse(xml, feed_url),
        FeedFormat::Atom => atom::parse(xml, feed_url),
    }
}
