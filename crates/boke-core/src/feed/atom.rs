use quick_xml::events::Event;
use quick_xml::Reader;

use super::date::parse_date;
use super::error::FeedError;
use super::model::{Feed, FeedEntry};

/// Parse an Atom 1.0 feed from XML bytes.
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

    let mut in_entry = false;
    let mut in_author = false;
    let mut current_entry: Option<FeedEntry> = None;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_string();

                match local.as_str() {
                    "entry" if !in_entry => {
                        in_entry = true;
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
                    "author" => in_author = true,
                    "link" => {
                        // Atom links are in attributes: <link href="..." rel="alternate" />
                        extract_link(e, &mut feed, &mut current_entry, in_entry);
                    }
                    "category" => {
                        // Atom categories: <category term="..." />
                        if in_entry {
                            if let Some(ref mut entry) = current_entry {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"term" {
                                        let val =
                                            attr.unescape_value().unwrap_or_default().to_string();
                                        if !val.is_empty() {
                                            entry.categories.push(val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                current_tag = local;
            }
            Ok(Event::Empty(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_string();
                match local.as_str() {
                    "link" => {
                        extract_link(e, &mut feed, &mut current_entry, in_entry);
                    }
                    "category" => {
                        if in_entry {
                            if let Some(ref mut entry) = current_entry {
                                for attr in e.attributes().flatten() {
                                    if attr.key.as_ref() == b"term" {
                                        let val =
                                            attr.unescape_value().unwrap_or_default().to_string();
                                        if !val.is_empty() {
                                            entry.categories.push(val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let local = std::str::from_utf8(e.local_name().as_ref())
                    .unwrap_or("")
                    .to_string();
                match local.as_str() {
                    "entry" if in_entry => {
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
                        in_entry = false;
                    }
                    "author" => in_author = false,
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::CData(ref e)) => {
                let text = std::str::from_utf8(e.as_ref()).unwrap_or("").to_string();
                if !text.is_empty() {
                    apply_text(
                        &mut feed,
                        &current_tag,
                        &text,
                        in_entry,
                        in_author,
                        &mut current_entry,
                    );
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.is_empty() {
                    apply_text(
                        &mut feed,
                        &current_tag,
                        &text,
                        in_entry,
                        in_author,
                        &mut current_entry,
                    );
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

fn extract_link(
    e: &quick_xml::events::BytesStart<'_>,
    feed: &mut Feed,
    current_entry: &mut Option<FeedEntry>,
    in_entry: bool,
) {
    let mut href = String::new();
    let mut rel = String::from("alternate"); // default rel is alternate

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        let val = attr.unescape_value().unwrap_or_default().to_string();
        match key {
            "href" => href = val,
            "rel" => rel = val,
            _ => {}
        }
    }

    if !href.is_empty() && (rel == "alternate" || rel.is_empty()) {
        if in_entry {
            if let Some(ref mut entry) = current_entry {
                if entry.link.is_empty() {
                    entry.link = href;
                }
            }
        } else if feed.link.is_empty() {
            feed.link = href;
        }
    }
}

fn apply_text(
    feed: &mut Feed,
    tag: &str,
    text: &str,
    in_entry: bool,
    in_author: bool,
    current_entry: &mut Option<FeedEntry>,
) {
    if in_entry {
        if let Some(ref mut entry) = current_entry {
            if in_author && tag == "name" {
                entry.author = Some(text.to_string());
                return;
            }
            match tag {
                "title" => entry.title = text.to_string(),
                "id" => entry.id = text.to_string(),
                "content" => entry.content = Some(text.to_string()),
                "summary" => entry.summary = Some(text.to_string()),
                "published" => entry.published = parse_date(text),
                "updated" => entry.updated = parse_date(text),
                _ => {}
            }
        }
    } else {
        match tag {
            "title" => feed.title = text.to_string(),
            "subtitle" => feed.description = Some(text.to_string()),
            "updated" => feed.last_updated = parse_date(text),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Atom Blog</title>
  <link href="https://example.com" rel="alternate" />
  <link href="https://example.com/atom.xml" rel="self" />
  <subtitle>An Atom test blog</subtitle>
  <updated>2024-01-15T10:30:00Z</updated>
  <entry>
    <title>Atom Post One</title>
    <id>urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a</id>
    <link href="https://example.com/atom-post-1" rel="alternate" />
    <content type="html">&lt;p&gt;Full atom content&lt;/p&gt;</content>
    <summary>Short atom summary</summary>
    <published>2024-01-15T10:30:00Z</published>
    <updated>2024-01-15T12:00:00Z</updated>
    <author><name>Atom Author</name></author>
    <category term="Atom" />
    <category term="Test" />
  </entry>
</feed>"#;

        let feed = parse(xml.as_bytes(), "https://example.com/atom.xml").unwrap();
        assert_eq!(feed.title, "Test Atom Blog");
        assert_eq!(feed.link, "https://example.com");
        assert_eq!(feed.description.as_deref(), Some("An Atom test blog"));
        assert!(feed.last_updated.is_some());
        assert_eq!(feed.entries.len(), 1);

        let entry = &feed.entries[0];
        assert_eq!(entry.title, "Atom Post One");
        assert_eq!(entry.id, "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a");
        assert_eq!(entry.link, "https://example.com/atom-post-1");
        assert!(entry.content.is_some());
        assert_eq!(entry.summary.as_deref(), Some("Short atom summary"));
        assert_eq!(entry.author.as_deref(), Some("Atom Author"));
        assert!(entry.published.is_some());
        assert!(entry.updated.is_some());
        assert_eq!(entry.categories, vec!["Atom", "Test"]);
    }
}
