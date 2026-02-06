use std::fmt;

#[derive(Debug)]
pub enum FeedError {
    Xml(quick_xml::Error),
    UnknownFormat,
    MissingField(&'static str),
    Http(reqwest::Error),
    Discovery(String),
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeedError::Xml(e) => write!(f, "XML parsing error: {e}"),
            FeedError::UnknownFormat => write!(f, "Unknown or unsupported feed format"),
            FeedError::MissingField(field) => write!(f, "Missing required field: {field}"),
            FeedError::Http(e) => write!(f, "HTTP error: {e}"),
            FeedError::Discovery(msg) => write!(f, "Feed discovery failed: {msg}"),
        }
    }
}

impl std::error::Error for FeedError {}

impl From<quick_xml::Error> for FeedError {
    fn from(e: quick_xml::Error) -> Self {
        FeedError::Xml(e)
    }
}

impl From<reqwest::Error> for FeedError {
    fn from(e: reqwest::Error) -> Self {
        FeedError::Http(e)
    }
}
