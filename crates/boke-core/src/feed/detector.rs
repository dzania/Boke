use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FeedFormat {
    Rss2,
    Rss1,
    Atom,
}

/// Detect the feed format by examining the root XML element.
pub fn detect_format(xml: &[u8]) -> Option<FeedFormat> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                return match name {
                    "rss" => Some(FeedFormat::Rss2),
                    "RDF" => Some(FeedFormat::Rss1),
                    "feed" => Some(FeedFormat::Atom),
                    _ => None,
                };
            }
            Ok(Event::Decl(_)) | Ok(Event::Comment(_)) | Ok(Event::PI(_)) => {
                // Skip XML declaration, comments, processing instructions
                continue;
            }
            Ok(Event::Eof) => return None,
            Err(_) => return None,
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rss2() {
        let xml = br#"<?xml version="1.0"?><rss version="2.0"><channel></channel></rss>"#;
        assert_eq!(detect_format(xml), Some(FeedFormat::Rss2));
    }

    #[test]
    fn test_detect_atom() {
        let xml = br#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom"></feed>"#;
        assert_eq!(detect_format(xml), Some(FeedFormat::Atom));
    }

    #[test]
    fn test_detect_rss1() {
        let xml = br#"<?xml version="1.0"?><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"></rdf:RDF>"#;
        assert_eq!(detect_format(xml), Some(FeedFormat::Rss1));
    }

    #[test]
    fn test_detect_unknown() {
        let xml = br#"<html><body>Hello</body></html>"#;
        assert_eq!(detect_format(xml), None);
    }
}
