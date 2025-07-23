use std::fmt;

#[derive(Debug)]
pub enum RssGenError {
    Io(std::io::Error),
    InvalidUrl(String),
    UnknownSiteType(String),
    RssNotFound(String),
}

impl fmt::Display for RssGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RssGenError::Io(e) => write!(f, "IO error: {}", e),
            RssGenError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            RssGenError::UnknownSiteType(url) => write!(f, "Unknown site type: {}", url),
            RssGenError::RssNotFound(url) => write!(f, "RSS feed not found for: {}", url),
        }
    }
}

impl From<std::io::Error> for RssGenError {
    fn from(e: std::io::Error) -> Self {
        RssGenError::Io(e)
    }
}
