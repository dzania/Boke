use quick_xml::events::Event;
use quick_xml::Reader;

use super::date::parse_date;
use super::error::FeedError;
use super::model::{Feed, FeedEntry};

/// Parse an RSS 2.0 feed from XML bytes.
pub fn parse(xml: &[u8], feed_url: &str) -> Result<Feed, FeedError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut feed = Feed {
        title: String::new(),
        link: String::new(),
        feed_url: feed_url.to_string(),
        description: None,
        language: None,
        last_updated: None,
        entries: Vec::new(),
    };

    let mut in_channel = false;
    let mut in_item = false;
    let mut current_entry: Option<FeedEntry> = None;
    let mut current_tag = String::new();
    // Track namespaced tags like content:encoded, dc:creator
    let mut current_ns_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_string();
                let full = std::str::from_utf8(e.name().as_ref()).unwrap_or("").to_string();

                match local.as_str() {
                    "channel" => in_channel = true,
                    "item" if in_channel => {
                        in_item = true;
                        current_entry = Some(FeedEntry {
                            id: String::new(),
                            title: String::new(),
                            link: String::new(),
                            content: None,
                            summary: None,
                            author: None,
                            published: None,
                            updated: None,
                            categories: Vec::new(),
                            image_url: None,
                        });
                    }
                    "enclosure" if in_item => {
                        if let Some(ref mut entry) = current_entry {
                            // Extract image from enclosure
                            let mut is_image = false;
                            let mut url = String::new();
                            for attr in e.attributes().flatten() {
                                let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                                let val = attr.unescape_value().unwrap_or_default().to_string();
                                match key {
                                    "url" => url = val,
                                    "type" if val.starts_with("image/") => is_image = true,
                                    _ => {}
                                }
                            }
                            if is_image && !url.is_empty() {
                                entry.image_url = Some(url);
                            }
                        }
                    }
                    _ => {}
                }

                current_tag = local;
                current_ns_tag = full;
            }
            Ok(Event::End(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_string();
                match local.as_str() {
                    "channel" => in_channel = false,
                    "item" if in_item => {
                        if let Some(mut entry) = current_entry.take() {
                            // Generate id from link if guid is missing
                            if entry.id.is_empty() {
                                entry.id = if !entry.link.is_empty() {
                                    entry.link.clone()
                                } else {
                                    format!("{}-{}", feed_url, feed.entries.len())
                                };
                            }
                            feed.entries.push(entry);
                        }
                        in_item = false;
                    }
                    _ => {}
                }
                current_tag.clear();
                current_ns_tag.clear();
            }
            Ok(Event::CData(ref e)) => {
                let text = std::str::from_utf8(e.as_ref()).unwrap_or("").to_string();
                if !text.is_empty() {
                    apply_text(&mut feed, &current_tag, &current_ns_tag, &text, in_item, &mut current_entry);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.is_empty() {
                    apply_text(&mut feed, &current_tag, &current_ns_tag, &text, in_item, &mut current_entry);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(FeedError::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    if feed.title.is_empty() {
        return Err(FeedError::MissingField("title"));
    }

    Ok(feed)
}

fn apply_text(
    feed: &mut Feed,
    tag: &str,
    ns_tag: &str,
    text: &str,
    in_item: bool,
    current_entry: &mut Option<FeedEntry>,
) {
    if in_item {
        if let Some(ref mut entry) = current_entry {
            match tag {
                "title" => entry.title = text.to_string(),
                "link" => entry.link = text.to_string(),
                "guid" => entry.id = text.to_string(),
                "description" => entry.summary = Some(text.to_string()),
                "encoded" if ns_tag.contains("content") => {
                    entry.content = Some(text.to_string());
                }
                "creator" if ns_tag.contains("dc") => {
                    entry.author = Some(text.to_string());
                }
                "author" => entry.author = Some(text.to_string()),
                "pubDate" => entry.published = parse_date(text),
                "category" => entry.categories.push(text.to_string()),
                _ => {}
            }
        }
    } else {
        // Channel-level metadata
        match tag {
            "title" => feed.title = text.to_string(),
            "link" => feed.link = text.to_string(),
            "description" => feed.description = Some(text.to_string()),
            "language" => feed.language = Some(text.to_string()),
            "lastBuildDate" | "pubDate" => {
                if feed.last_updated.is_none() {
                    feed.last_updated = parse_date(text);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss2() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:content="http://purl.org/rss/1.0/modules/content/" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Test Blog</title>
    <link>https://example.com</link>
    <description>A test blog</description>
    <language>en</language>
    <lastBuildDate>Mon, 15 Jan 2024 10:30:00 +0000</lastBuildDate>
    <item>
      <title>First Post</title>
      <link>https://example.com/post-1</link>
      <guid>https://example.com/post-1</guid>
      <description>Short summary</description>
      <content:encoded><![CDATA[<p>Full content here</p>]]></content:encoded>
      <dc:creator>Author Name</dc:creator>
      <pubDate>Mon, 15 Jan 2024 10:30:00 +0000</pubDate>
      <category>Tech</category>
      <category>Rust</category>
    </item>
    <item>
      <title>Second Post</title>
      <link>https://example.com/post-2</link>
      <description>Another post</description>
      <pubDate>Sun, 14 Jan 2024 08:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

        let feed = parse(xml.as_bytes(), "https://example.com/feed").unwrap();
        assert_eq!(feed.title, "Test Blog");
        assert_eq!(feed.link, "https://example.com");
        assert_eq!(feed.description.as_deref(), Some("A test blog"));
        assert_eq!(feed.language.as_deref(), Some("en"));
        assert!(feed.last_updated.is_some());
        assert_eq!(feed.entries.len(), 2);

        let first = &feed.entries[0];
        assert_eq!(first.title, "First Post");
        assert_eq!(first.link, "https://example.com/post-1");
        assert_eq!(first.id, "https://example.com/post-1");
        assert_eq!(first.summary.as_deref(), Some("Short summary"));
        assert_eq!(first.content.as_deref(), Some("<p>Full content here</p>"));
        assert_eq!(first.author.as_deref(), Some("Author Name"));
        assert!(first.published.is_some());
        assert_eq!(first.categories, vec!["Tech", "Rust"]);

        // Second entry has no guid — should use link as id
        let second = &feed.entries[1];
        assert_eq!(second.id, "https://example.com/post-2");
    }
}
