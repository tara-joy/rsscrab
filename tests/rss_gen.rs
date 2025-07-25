use rsscrab::rss_gen::{substack_rss, telegram_rss};

#[tokio::test]
async fn test_substack_rss_valid() {
    let url = "https://example.substack.com";
    let result = substack_rss(url).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://example.substack.com/feed");
}

#[tokio::test]
async fn test_substack_rss_invalid() {
    let url = "not_a_url";
    let result = substack_rss(url).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_telegram_rss_valid() {
    let url = "https://t.me/mychannel";
    let result = telegram_rss(url).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://rsshub.app/telegram/channel/mychannel");
}

#[tokio::test]
async fn test_telegram_rss_invalid() {
    let url = "https://t.me/";
    let result = telegram_rss(url).await;
    assert!(result.is_err());
}