#[derive(Debug, PartialEq, Eq)]
pub enum SiteType {
    YouTube,
    Substack,
    Blog,
    Telegram,
    Odysee,
    Bitchute,
    Rumble,
    Unknown,
}

pub fn detect(url: &str) -> SiteType {
    let url = url.to_lowercase();
    if url.contains("youtube.com") || url.contains("youtu.be") {
        SiteType::YouTube
    } else if url.contains("substack.com") {
        SiteType::Substack
    } else if url.contains("t.me") || url.contains("telegram.me") {
        SiteType::Telegram
    } else if url.contains("odysee.com") {
        SiteType::Odysee
    } else if url.contains("bitchute.com") {
        SiteType::Bitchute
    } else if url.contains("rumble.com") {
        SiteType::Rumble
    } else if url.contains("http") {
        SiteType::Blog
    } else {
        SiteType::Unknown
    }
}
