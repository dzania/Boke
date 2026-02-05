use quick_xml::events::Event;
use quick_xml::Reader;

use super::date::parse_date;
use super::error::FeedError;
use super::model::{Feed, FeedEntry};

/// Parse an RSS 1.0 (RDF) feed from XML bytes.
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
    let mut current_ns_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_string();
                let full = std::str::from_utf8(e.name().as_ref()).unwrap_or("").to_string();

                match local.as_str() {
                    "channel" => in_channel = true,
                    "item" => {
                        in_item = true;
                        in_channel = false;
                        let mut about = String::new();
                        for attr in e.attributes().flatten() {
                            let key_name = attr.key.local_name();
                            let key = std::str::from_utf8(key_name.as_ref()).unwrap_or("");
                            if key == "about" {
                                about = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                        current_entry = Some(FeedEntry {
                            id: about,
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
                    apply_text(&mut feed, &current_tag, &current_ns_tag, &text, in_item, in_channel, &mut current_entry);
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.is_empty() {
                    apply_text(&mut feed, &current_tag, &current_ns_tag, &text, in_item, in_channel, &mut current_entry);
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
    in_channel: bool,
    current_entry: &mut Option<FeedEntry>,
) {
    if in_item {
        if let Some(ref mut entry) = current_entry {
            match tag {
                "title" => entry.title = text.to_string(),
                "link" => entry.link = text.to_string(),
                "description" => entry.summary = Some(text.to_string()),
                "encoded" if ns_tag.contains("content") => {
                    entry.content = Some(text.to_string());
                }
                "creator" if ns_tag.contains("dc") => {
                    entry.author = Some(text.to_string());
                }
                "date" if ns_tag.contains("dc") => {
                    entry.published = parse_date(text);
                }
                "subject" if ns_tag.contains("dc") => {
                    entry.categories.push(text.to_string());
                }
                _ => {}
            }
        }
    } else if in_channel {
        match tag {
            "title" => feed.title = text.to_string(),
            "link" => feed.link = text.to_string(),
            "description" => feed.description = Some(text.to_string()),
            "language" if ns_tag.contains("dc") => feed.language = Some(text.to_string()),
            "date" if ns_tag.contains("dc") => feed.last_updated = parse_date(text),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rdf:RDF
  xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  xmlns="http://purl.org/rss/1.0/"
  xmlns:dc="http://purl.org/dc/elements/1.1/"
>
  <channel>
    <title>RDF Test Blog</title>
    <link>https://example.com</link>
    <description>An RDF test blog</description>
  </channel>
  <item rdf:about="https://example.com/rdf-post-1">
    <title>RDF Post One</title>
    <link>https://example.com/rdf-post-1</link>
    <dc:date>2024-01-15T10:30:00Z</dc:date>
    <dc:creator>RDF Author</dc:creator>
    <description>RDF summary</description>
  </item>
</rdf:RDF>"#;

        let feed = parse(xml.as_bytes(), "https://example.com/rss1").unwrap();
        assert_eq!(feed.title, "RDF Test Blog");
        assert_eq!(feed.link, "https://example.com");
        assert_eq!(feed.description.as_deref(), Some("An RDF test blog"));
        assert_eq!(feed.entries.len(), 1);

        let entry = &feed.entries[0];
        assert_eq!(entry.title, "RDF Post One");
        assert_eq!(entry.id, "https://example.com/rdf-post-1");
        assert_eq!(entry.link, "https://example.com/rdf-post-1");
        assert_eq!(entry.author.as_deref(), Some("RDF Author"));
        assert_eq!(entry.summary.as_deref(), Some("RDF summary"));
        assert!(entry.published.is_some());
    }
}
