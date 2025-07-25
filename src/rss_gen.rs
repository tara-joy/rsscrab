use crate::site_type::SiteType;
use crate::error::RssGenError;
use serde::Serialize;

#[derive(Serialize)]
pub struct YoutubeRss {
    pub feed: String,
    pub name: String,
}

pub async fn generate(url: &str, site_type: &SiteType) -> Result<String, RssGenError> {
    match site_type {
        SiteType::YouTube => youtube_rss_url(url).await,
        SiteType::Substack => substack_rss(url).await,
        SiteType::Telegram => telegram_rss(url).await,
        SiteType::Odysee => odysee_rss(url).await,
        SiteType::Bitchute => bitchute_rss(url).await,
        SiteType::Rumble => rumble_rss(url).await,
        SiteType::Blog => blog_rss_discover(url).await,
        SiteType::Unknown => Err(RssGenError::UnknownSiteType(url.to_string())),
    }
}

async fn blog_rss_discover(url: &str) -> Result<String, RssGenError> {
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
        if let Ok(resp) = reqwest::get(&candidate).await {
            if resp.status().is_success() {
                let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("");
                if content_type.contains("xml") || content_type.contains("rss") || content_type.contains("atom") {
                    return Ok(candidate);
                }
                if let Ok(body) = resp.text().await {
                    if body.contains("<rss") || body.contains("<feed") {
                        return Ok(candidate);
                    }
                }
            }
        }
    }
    let resp = reqwest::get(url).await.map_err(|_| RssGenError::InvalidUrl(url.to_string()))?;
    let body = resp.text().await.map_err(|_| RssGenError::RssNotFound(url.to_string()))?;
    if let Some(feed_url) = extract_feed_link(&body, "application/rss+xml") {
        return Ok(feed_url);
    }
    if let Some(feed_url) = extract_feed_link(&body, "application/atom+xml") {
        return Ok(feed_url);
    }
    let re = regex::Regex::new(r#"(?:<a|<link)[^>]+href=["']([^"'>]+)["'][^>]*>"#).ok();
    if let Some(re) = &re {
        for cap in re.captures_iter(&body) {
            if let Some(href) = cap.get(1) {
                let href = href.as_str();
                let href_lc = href.to_ascii_lowercase();
                if href_lc.contains("rss") || href_lc.contains("atom") || href_lc.ends_with(".xml") {
                    let feed_url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with('/') {
                        let base = url.trim_end_matches('/');
                        format!("{}{}", base, href)
                    } else {
                        let base = url.trim_end_matches('/');
                        format!("{}/{}", base, href)
                    };
                    if let Ok(resp) = reqwest::get(&feed_url).await {
                        if resp.status().is_success() {
                            return Ok(feed_url);
                        }
                    }
                }
            }
        }
    }
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

async fn youtube_rss_url(url: &str) -> Result<String, RssGenError> {
    let (feed, _name) = youtube_rss_blocking(url).await?;
    Ok(feed)
}

async fn youtube_rss_blocking(url: &str) -> Result<(String, String), RssGenError> {
    if let Some(id) = url.split("/channel/").nth(1) {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", id);
        let name = get_youtube_channel_name(url).await;
        return Ok((feed, name));
    }
    let resp = reqwest::get(url).await.map_err(|_| RssGenError::InvalidUrl(url.to_string()))?;
    let body = resp.text().await.map_err(|_| RssGenError::RssNotFound(url.to_string()))?;
    if let Some(feed_url) = extract_youtube_rss_link(&body) {
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed_url, name));
    }
    if let Some(channel_id) = extract_meta_content(&body, "itemprop=\"channelId\"") {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed, name));
    }
    if let Some(channel_id) = extract_channel_id_regex(&body) {
        let feed = format!("https://www.youtube.com/feeds/videos.xml?channel_id={}", channel_id);
        let name = extract_meta_content(&body, "property=\"og:title\"").unwrap_or_else(|| "Unknown Channel".to_string());
        return Ok((feed, name));
    }
    Err(RssGenError::RssNotFound(url.to_string()))
}

fn extract_youtube_rss_link(body: &str) -> Option<String> {
    let re = regex::Regex::new(r#"<link[^>]+type=["']application/rss\+xml["'][^>]+href=["']([^"']+)["']"#).ok()?;
    if let Some(caps) = re.captures(body) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    None
}

fn extract_channel_id_regex(body: &str) -> Option<String> {
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
    for line in body.lines() {
        if let Some(idx) = line.find(needle) {
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

async fn get_youtube_channel_name(url: &str) -> String {
    if let Ok(resp) = reqwest::get(url).await {
        if let Ok(body) = resp.text().await {
            if let Some(name) = extract_meta_content(&body, "property=\"og:title\"") {
                return name;
            }
        }
    }
    "Unknown Channel".to_string()
}

pub async fn substack_rss(url: &str) -> Result<String, RssGenError> {
    if let Some(domain) = url.split("//").nth(1) {
        let domain = domain.trim_end_matches('/');
        Ok(format!("https://{}/feed", domain))
    } else {
        Err(RssGenError::InvalidUrl(url.to_string()))
    }
}

pub async fn telegram_rss(url: &str) -> Result<String, RssGenError> {
    if let Some(channel) = url.split("/t.me/").nth(1) {
        if !channel.is_empty() {
            Ok(format!("https://rsshub.app/telegram/channel/{}", channel))
        } else {
            Err(RssGenError::InvalidUrl(url.to_string()))
        }
    } else {
        Err(RssGenError::InvalidUrl(url.to_string()))
    }
}

async fn odysee_rss(_url: &str) -> Result<String, RssGenError> {
    Err(RssGenError::RssNotFound("Odysee RSS not implemented".to_string()))
}

async fn bitchute_rss(url: &str) -> Result<String, RssGenError> {
    let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
    if let Some((i, _)) = parts.iter().enumerate().rev().find(|&(_, &s)| s == "channel") {
        if let Some(channel_name) = parts.get(i + 1) {
            let feed = format!("https://api.bitchute.com/feeds/rss/channel/{}", channel_name);
            return Ok(feed);
        }
    }
    Err(RssGenError::InvalidUrl(url.to_string()))
}

async fn rumble_rss(_url: &str) -> Result<String, RssGenError> {
    Err(RssGenError::RssNotFound("Rumble RSS not implemented".to_string()))
}
