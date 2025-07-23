
use crate::site_type::SiteType;
use crate::error::RssGenError;
use serde::Serialize;

#[derive(Serialize)]
pub struct YoutubeRss {
    pub feed: String,
    pub name: String,
}



pub fn generate(url: &str, site_type: &SiteType) -> Result<String, RssGenError> {
    match site_type {
        SiteType::YouTube => youtube_rss_url(url),
        SiteType::Substack => substack_rss(url),
        SiteType::Telegram => telegram_rss(url),
        SiteType::Odysee => odysee_rss(url),
        SiteType::Bitchute => bitchute_rss(url),
        SiteType::Rumble => rumble_rss(url),
        SiteType::Blog => blog_rss_discover(url),
        SiteType::Unknown => Err(RssGenError::UnknownSiteType(url.to_string())),
    }
}

fn blog_rss_discover(url: &str) -> Result<String, RssGenError> {
    // Try common feed URL patterns first
    let patterns = [
        "/feed", "/feed/", "/rss", "/rss.xml", "/atom.xml", "/index.xml"
    ];
    let base = url.trim_end_matches('/');
    for pat in &patterns {
        let candidate = if base.ends_with(pat) {
            base.to_string()
        } else {
            format!("{}{}", base, pat)
        };
        if let Ok(resp) = reqwest::blocking::get(&candidate) {
            if resp.status().is_success() {
                let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
                if content_type.contains("xml") || content_type.contains("rss") || content_type.contains("atom") {
                    return Ok(candidate);
                }
                // Fallback: check for <rss or <feed in body
                if let Ok(body) = resp.text() {
                    if body.contains("<rss") || body.contains("<feed") {
                        return Ok(candidate);
                    }
                }
            }
        }
    }
    // Fallback: HTML parsing for <link rel="alternate" ...>
    let resp = reqwest::blocking::get(url).map_err(|_| RssGenError::InvalidUrl(url.to_string()))?;
    let body = resp.text().map_err(|_| RssGenError::RssNotFound(url.to_string()))?;
    // Try RSS first
    if let Some(feed_url) = extract_feed_link(&body, "application/rss+xml") {
        return Ok(feed_url);
    }
    // Try Atom
    if let Some(feed_url) = extract_feed_link(&body, "application/atom+xml") {
        return Ok(feed_url);
    }
    // Generalized: scan all <a> and <link> tags for hrefs that look like feeds
    let re = regex::Regex::new(r#"(?:<a|<link)[^>]+href=["']([^"'>]+)["'][^>]*>"#).ok();
    if let Some(re) = &re {
        for cap in re.captures_iter(&body) {
            if let Some(href) = cap.get(1) {
                let href = href.as_str();
                let href_lc = href.to_ascii_lowercase();
                if href_lc.contains("rss") || href_lc.contains("atom") || href_lc.ends_with(".xml") {
                    // Make absolute if needed
                    let feed_url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with('/') {
                        // Absolute path
                        let base = url.trim_end_matches('/');
                        format!("{}{}", base, href)
                    } else {
                        // Relative path
                        let base = url.trim_end_matches('/');
                        format!("{}/{}", base, href)
                    };
                    // Optionally, check if it is reachable
                    if let Ok(resp) = reqwest::blocking::get(&feed_url) {
                        if resp.status().is_success() {
                            return Ok(feed_url);
                        }
                    }
                }
            }
        }
    }
    // Fallback: return original URL or error
    Err(RssGenError::RssNotFound(url.to_string()))
}

fn extract_feed_link(body: &str, feed_type: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r#"<link[^>]+rel=["']alternate["'][^>]+type=["']{}["'][^>]+href=["']([^"']+)["']"#, feed_type)).ok()?;
    if let Some(caps) = re.captures(body) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    None
}


fn youtube_rss_url(url: &str) -> Result<String, RssGenError> {
    let (feed, _name) = youtube_rss_blocking(url)?;
    Ok(feed)
}

fn youtube_rss_blocking(url: &str) -> Result<(String, String), RssGenError> {
    // If /channel/CHANNEL_ID, use directly
    if let Some(id) = url.split("/channel/").nth(1) {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", id);
        let name = get_youtube_channel_name(url);
        return Ok((feed, name));
    }
    // For all other YouTube URLs, fetch the page and try to extract RSS link or channelId
    let resp = reqwest::blocking::get(url).map_err(|_| RssGenError::InvalidUrl(url.to_string()))?;
    let body = resp.text().map_err(|_| RssGenError::RssNotFound(url.to_string()))?;
    // 1. Try to find direct RSS link
    if let Some(feed_url) = extract_youtube_rss_link(&body) {
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed_url, name));
    }
    // 2. Try meta tag for channelId
    if let Some(channel_id) = extract_meta_content(&body, "itemprop=\"channelId\"") {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed, name));
    }
    // 3. Fallback: regex search for channelId
    if let Some(channel_id) = extract_channel_id_regex(&body) {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed, name));
    }
    Err(RssGenError::RssNotFound(url.to_string()))
}

fn extract_youtube_rss_link(body: &str) -> Option<String> {
    // Look for: <link rel="alternate" type="application/rss+xml" href="...">
    let re = regex::Regex::new(r#"<link[^>]+type=["']application/rss\+xml["'][^>]+href=["']([^"']+)["']"#).ok()?;
    if let Some(caps) = re.captures(body) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    None
}

fn extract_channel_id_regex(body: &str) -> Option<String> {
    // Use regex to find channelId in the HTML
    let re = regex::Regex::new(r#"channelId"\s*:\s*"([A-Za-z0-9_-]{24})"|itemprop=\"channelId\" content=\"([A-Za-z0-9_-]{24})\"|<meta itemprop=\"channelId\" content=\"([A-Za-z0-9_-]{24})\""#).ok()?;
    if let Some(caps) = re.captures(body) {
        for i in 1..=3 {
            if let Some(m) = caps.get(i) {
                return Some(m.as_str().to_string());
            }
        }
    }
    None
}

fn extract_meta_content(body: &str, needle: &str) -> Option<String> {
    // Simple string search for meta tag content (single line)
    for line in body.lines() {
        if let Some(idx) = line.find(needle) {
            // Look for content="..."
            if let Some(content_idx) = line[idx..].find("content=\"") {
                let start = idx + content_idx + 9;
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start+end].to_string());
                }
            }
        }
    }
    None
}

fn get_youtube_channel_name(url: &str) -> String {
    if let Ok(resp) = reqwest::blocking::get(url) {
        if let Ok(body) = resp.text() {
            if let Some(name) = extract_meta_content(&body, "property=\"og:title\"") {
                return name;
            }
        }
    }
    "Unknown Channel".to_string()
}
// Add regex crate to dependencies in Cargo.toml:
// regex = "1.10"

fn substack_rss(url: &str) -> Result<String, RssGenError> {
    // Example: https://example.substack.com
    if let Some(domain) = url.split("//").nth(1) {
        let domain = domain.trim_end_matches('/');
        Ok(format!("https://{}/feed", domain))
    } else {
        Err(RssGenError::InvalidUrl(url.to_string()))
    }
}

fn telegram_rss(url: &str) -> Result<String, RssGenError> {
    // Example: https://t.me/channel
    if let Some(channel) = url.split("/t.me/").nth(1) {
        Ok(format!("https://rsshub.app/telegram/channel/{}", channel))
    } else {
        Err(RssGenError::InvalidUrl(url.to_string()))
    }
}

fn odysee_rss(_url: &str) -> Result<String, RssGenError> {
    // Placeholder: Odysee does not have official RSS, may require RSSHub or similar
    Err(RssGenError::RssNotFound("Odysee RSS not implemented".to_string()))
}

fn bitchute_rss(url: &str) -> Result<String, RssGenError> {
    // Example: https://www.bitchute.com/channel/biocharisma/
    // RSS:    https://api.bitchute.com/feeds/rss/channel/biocharisma
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    if let Some((i, _)) = parts.iter().enumerate().rev().find(|&(_, &s)| s == "channel") {
        if let Some(channel_name) = parts.get(i + 1) {
            let feed = format!("https://api.bitchute.com/feeds/rss/channel/{}", channel_name);
            return Ok(feed);
        }
    }
    Err(RssGenError::InvalidUrl(url.to_string()))
}

fn rumble_rss(_url: &str) -> Result<String, RssGenError> {
    // Placeholder: Rumble does not have official RSS, may require RSSHub or similar
    Err(RssGenError::RssNotFound("Rumble RSS not implemented".to_string()))
}
