// Error handling for RSS Crab
// import fmt module from std library
use std::fmt;

/// Custom error type for RSS feed generation
#[derive(Debug)]
pub enum RssGenError {
    /// IO error (e.g., file or network issues)
    Io(std::io::Error),
    /// The provided URL is invalid
    InvalidUrl(String),
    /// The site type is not recognized
    UnknownSiteType(String),
    /// No RSS feed found for the given URL
    RssNotFound(String),
}

// Display implementation for user-friendly error messages
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

// Allow automatic conversion from std::io::Error to RssGenError
impl From<std::io::Error> for RssGenError {
    fn from(e: std::io::Error) -> Self {
        RssGenError::Io(e)
    }
}
