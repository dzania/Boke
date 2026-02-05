pub mod atom;
pub mod date;
pub mod detector;
pub mod discovery;
pub mod error;
pub mod model;
pub mod rss1;
pub mod rss2;

use detector::FeedFormat;
use error::FeedError;
use model::Feed;

/// Parse XML bytes into a Feed, auto-detecting the format.
pub fn parse(xml: &[u8], feed_url: &str) -> Result<Feed, FeedError> {
    let format = detector::detect_format(xml).ok_or(FeedError::UnknownFormat)?;

    match format {
        FeedFormat::Rss2 => rss2::parse(xml, feed_url),
        FeedFormat::Rss1 => rss1::parse(xml, feed_url),
        FeedFormat::Atom => atom::parse(xml, feed_url),
    }
}
