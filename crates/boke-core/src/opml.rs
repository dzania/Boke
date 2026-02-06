//! OPML parsing for feed import/export.

use quick_xml::Reader;
use quick_xml::events::Event;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpmlError {
    #[error("Failed to parse OPML: {0}")]
    ParseError(String),

    #[error("No feeds found in OPML file")]
    NoFeeds,
}

/// Parse an OPML file and extract feed URLs.
///
/// Returns a list of feed URLs found in the OPML file.
pub fn parse_opml(xml: &str) -> Result<Vec<String>, OpmlError> {
    let mut reader = Reader::from_str(xml);
    let mut urls = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e))
                if e.name().as_ref() == b"outline" =>
            {
                let mut xml_url = None;
                for attr in e.attributes().flatten() {
                    if (attr.key.as_ref() == b"xmlUrl" || attr.key.as_ref() == b"xmlurl")
                        && let Ok(val) = attr.unescape_value()
                    {
                        let url = val.to_string();
                        if !url.is_empty() {
                            xml_url = Some(url);
                        }
                    }
                }
                if let Some(url) = xml_url {
                    urls.push(url);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpmlError::ParseError(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    if urls.is_empty() {
        return Err(OpmlError::NoFeeds);
    }

    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_opml() {
        let opml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <body>
    <outline text="Tech" title="Tech">
      <outline type="rss" text="Hacker News" xmlUrl="https://news.ycombinator.com/rss"/>
      <outline type="rss" text="Lobsters" xmlUrl="https://lobste.rs/rss"/>
    </outline>
  </body>
</opml>"#;

        let urls = parse_opml(opml).unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://news.ycombinator.com/rss".to_string()));
        assert!(urls.contains(&"https://lobste.rs/rss".to_string()));
    }

    #[test]
    fn parse_flat_opml() {
        let opml = r#"<?xml version="1.0"?>
<opml version="1.0">
  <body>
    <outline xmlUrl="https://example.com/feed.xml"/>
  </body>
</opml>"#;

        let urls = parse_opml(opml).unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/feed.xml");
    }

    #[test]
    fn empty_opml_returns_error() {
        let opml = r#"<?xml version="1.0"?>
<opml version="1.0">
  <body>
  </body>
</opml>"#;

        let result = parse_opml(opml);
        assert!(matches!(result, Err(OpmlError::NoFeeds)));
    }
}
