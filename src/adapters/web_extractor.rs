use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

use crate::traits::MetadataExtractor;
use crate::types::{ExtractedMetadata, ExtractorError};

pub struct WebExtractor {
    client: Client,
}

impl WebExtractor {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MetadataExtractor for WebExtractor {
    async fn extract_metadata(&self, url: &str, timeout: Duration) -> Result<ExtractedMetadata, ExtractorError> {
        // Validate URL
        let parsed_url = Url::parse(url)
            .map_err(|_| ExtractorError::InvalidUrl(url.to_string()))?;

        // Make HTTP request
        let response = self
            .client
            .get(parsed_url)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ExtractorError::Timeout
                } else {
                    ExtractorError::NetworkError(e.to_string())
                }
            })?;

        // Get HTML content
        let html_content = response
            .text()
            .await
            .map_err(|e| ExtractorError::NetworkError(e.to_string()))?;

        // Parse HTML
        let document = Html::parse_document(&html_content);

        // Extract metadata
        let title = extract_title(&document);
        let author = extract_author(&document);
        let publish_date = extract_publish_date(&document);

        Ok(ExtractedMetadata {
            title,
            author,
            publish_date,
        })
    }
}

fn extract_title(document: &Html) -> Option<String> {
    let title_selector = Selector::parse("title").ok()?;
    document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_author(document: &Html) -> Option<String> {
    // Try various meta tags for author
    let meta_selectors = vec![
        "meta[name='author']",
        "meta[property='article:author']",
        "meta[name='article:author']",
        "meta[property='author']",
    ];

    for selector_str in meta_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    let content = content.trim().to_string();
                    if !content.is_empty() {
                        return Some(content);
                    }
                }
            }
        }
    }

    None
}

fn extract_publish_date(document: &Html) -> Option<DateTime<Utc>> {
    // Try various meta tags for publish date
    let meta_selectors = vec![
        "meta[property='article:published_time']",
        "meta[name='article:published_time']",
        "meta[property='published_time']",
        "meta[name='published_time']",
        "meta[property='article:published']",
        "meta[name='publish_date']",
    ];

    for selector_str in meta_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    // Try to parse various date formats
                    if let Ok(date) = DateTime::parse_from_rfc3339(content) {
                        return Some(date.with_timezone(&Utc));
                    }
                    if let Ok(date) = DateTime::parse_from_rfc2822(content) {
                        return Some(date.with_timezone(&Utc));
                    }
                    // Try ISO 8601 without timezone
                    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(content, "%Y-%m-%dT%H:%M:%S") {
                        return Some(date.and_utc());
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use chrono::Datelike;

    #[test]
    fn test_extract_title() {
        let html = r#"
            <html>
                <head>
                    <title>Test Page Title</title>
                </head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document);
        assert_eq!(title, Some("Test Page Title".to_string()));
    }

    #[test]
    fn test_extract_title_with_whitespace() {
        let html = r#"
            <html>
                <head>
                    <title>   Whitespace Title   </title>
                </head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document);
        assert_eq!(title, Some("Whitespace Title".to_string()));
    }

    #[test]
    fn test_extract_title_missing() {
        let html = r#"
            <html>
                <head></head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let title = extract_title(&document);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_author_from_meta_name() {
        let html = r#"
            <html>
                <head>
                    <meta name="author" content="John Doe">
                </head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let author = extract_author(&document);
        assert_eq!(author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_author_from_article_meta() {
        let html = r#"
            <html>
                <head>
                    <meta property="article:author" content="Jane Smith">
                </head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let author = extract_author(&document);
        assert_eq!(author, Some("Jane Smith".to_string()));
    }

    #[test]
    fn test_extract_author_missing() {
        let html = r#"
            <html>
                <head></head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let author = extract_author(&document);
        assert_eq!(author, None);
    }

    #[test]
    fn test_extract_publish_date_rfc3339() {
        let html = r#"
            <html>
                <head>
                    <meta property="article:published_time" content="2023-12-25T10:30:00Z">
                </head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let date = extract_publish_date(&document);
        assert!(date.is_some());
        let date = date.unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 25);
    }

    #[test]
    fn test_extract_publish_date_missing() {
        let html = r#"
            <html>
                <head></head>
                <body></body>
            </html>
        "#;
        let document = Html::parse_document(html);
        let date = extract_publish_date(&document);
        assert_eq!(date, None);
    }

    #[tokio::test]
    async fn test_web_extractor_creation() {
        let extractor = WebExtractor::new();
        // Just test that it can be created
        assert!(std::mem::size_of_val(&extractor) > 0);
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let extractor = WebExtractor::new();
        let result = extractor.extract_metadata("not-a-url", Duration::from_secs(10)).await;
        assert!(matches!(result, Err(ExtractorError::InvalidUrl(_))));
    }

    // Integration test with mock server would go here in a real implementation
    // For now, we'll skip network tests to avoid external dependencies in unit tests
}