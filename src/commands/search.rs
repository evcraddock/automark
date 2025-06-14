use crate::commands::{CommandHandler, OutputFormat, output};
use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult, BookmarkError, BookmarkFilters, ReadingStatus, SortBy, SortDirection};
use clap::Args;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Command-line arguments for search command
#[derive(Args, Debug, Clone)]
pub struct SearchArgs {
    /// Text query to search for in title, URL, author, and notes
    pub query: Option<String>,
    
    /// Filter by tags (comma-separated for multiple tags, uses AND logic)
    #[arg(long, value_delimiter = ',')]
    pub tags: Option<Vec<String>>,
    
    /// Filter by reading status
    #[arg(long, value_enum)]
    pub status: Option<ReadingStatus>,
    
    /// Filter by priority range (format: "min-max" or "exact")
    #[arg(long)]
    pub priority: Option<String>,
    
    /// Filter bookmarks created since this date (MM-DD-YYYY format)
    #[arg(long)]
    pub since: Option<String>,
    
    /// Filter bookmarks created until this date (MM-DD-YYYY format)
    #[arg(long)]
    pub until: Option<String>,
    
    /// Filter by publish date since (MM-DD-YYYY format)
    #[arg(long)]
    pub published_since: Option<String>,
    
    /// Filter by publish date until (MM-DD-YYYY format)
    #[arg(long)]
    pub published_until: Option<String>,
    
    /// Sort results by field
    #[arg(long, value_enum)]
    pub sort_by: Option<SortBy>,
    
    /// Sort direction
    #[arg(long, value_enum, default_value = "descending")]
    pub sort_order: SortDirection,
}

/// JSON response for search command
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
    pub results: Vec<Bookmark>,
    pub total_count: usize,
    pub query_summary: QuerySummary,
}

/// Summary of the search query and filters applied
#[derive(Serialize, Deserialize, Debug)]
pub struct QuerySummary {
    pub text_query: Option<String>,
    pub tags_filter: Option<Vec<String>>,
    pub status_filter: Option<ReadingStatus>,
    pub priority_filter: Option<String>,
    pub date_range: Option<String>,
    pub sort_info: Option<String>,
}

pub struct SearchCommand {
    args: SearchArgs,
}

impl SearchCommand {
    pub fn new(args: SearchArgs) -> Self {
        Self { args }
    }
    
    /// Parse priority range from string format
    fn parse_priority_range(&self, priority_str: &str) -> BookmarkResult<(u8, u8)> {
        if let Some(dash_pos) = priority_str.find('-') {
            // Range format: "min-max"
            let min_str = &priority_str[..dash_pos];
            let max_str = &priority_str[dash_pos + 1..];
            
            let min: u8 = min_str.parse()
                .map_err(|_| BookmarkError::InvalidId(format!("Invalid priority minimum: {}", min_str)))?;
            let max: u8 = max_str.parse()
                .map_err(|_| BookmarkError::InvalidId(format!("Invalid priority maximum: {}", max_str)))?;
                
            if !(1..=5).contains(&min) || !(1..=5).contains(&max) || min > max {
                return Err(BookmarkError::InvalidId(
                    format!("Priority range must be 1-5 with min <= max, got {}-{}", min, max)
                ));
            }
            
            Ok((min, max))
        } else {
            // Single value format
            let priority: u8 = priority_str.parse()
                .map_err(|_| BookmarkError::InvalidId(format!("Invalid priority: {}", priority_str)))?;
                
            if !(1..=5).contains(&priority) {
                return Err(BookmarkError::InvalidId(
                    format!("Priority must be between 1 and 5, got {}", priority)
                ));
            }
            
            Ok((priority, priority))
        }
    }
    
    /// Parse date string to DateTime<Utc>
    fn parse_date(&self, date_str: &str) -> BookmarkResult<DateTime<Utc>> {
        use chrono::NaiveDate;
        
        // Parse MM-DD-YYYY format
        NaiveDate::parse_from_str(date_str, "%m-%d-%Y")
            .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc())
            .map_err(|_| BookmarkError::InvalidId(
                format!("Invalid date format '{}'. Use MM-DD-YYYY format (e.g., 01-15-2023)", date_str)
            ))
    }
    
    /// Build BookmarkFilters from command arguments
    fn build_filters(&self) -> BookmarkResult<BookmarkFilters> {
        let priority_range = if let Some(ref priority_str) = self.args.priority {
            Some(self.parse_priority_range(priority_str)?)
        } else {
            None
        };
        
        let bookmarked_since = if let Some(ref since_str) = self.args.since {
            Some(self.parse_date(since_str)?)
        } else {
            None
        };
        
        let bookmarked_until = if let Some(ref until_str) = self.args.until {
            Some(self.parse_date(until_str)?)
        } else {
            None
        };
        
        let published_since = if let Some(ref since_str) = self.args.published_since {
            Some(self.parse_date(since_str)?)
        } else {
            None
        };
        
        let published_until = if let Some(ref until_str) = self.args.published_until {
            Some(self.parse_date(until_str)?)
        } else {
            None
        };
        
        Ok(BookmarkFilters {
            text_query: self.args.query.clone(),
            tags: self.args.tags.clone(),
            reading_status: self.args.status.clone(),
            priority_range,
            bookmarked_since,
            bookmarked_until,
            published_since,
            published_until,
            sort_by: self.args.sort_by.clone(),
            sort_order: Some(self.args.sort_order.clone()),
        })
    }
    
    /// Generate query summary for JSON output
    fn generate_query_summary(&self) -> QuerySummary {
        let date_range = match (&self.args.since, &self.args.until) {
            (Some(since), Some(until)) => Some(format!("{} to {}", since, until)),
            (Some(since), None) => Some(format!("since {}", since)),
            (None, Some(until)) => Some(format!("until {}", until)),
            (None, None) => None,
        };
        
        let sort_info = match (&self.args.sort_by, &self.args.sort_order) {
            (Some(sort_by), sort_order) => Some(format!("{:?} {:?}", sort_by, sort_order)),
            (None, _) => None,
        };
        
        QuerySummary {
            text_query: self.args.query.clone(),
            tags_filter: self.args.tags.clone(),
            status_filter: self.args.status.clone(),
            priority_filter: self.args.priority.clone(),
            date_range,
            sort_info,
        }
    }
    
    /// Format search results for human output
    fn format_human_output(&self, bookmarks: &[Bookmark]) -> String {
        if bookmarks.is_empty() {
            return "No bookmarks found matching your search criteria.".to_string();
        }
        
        let mut output = format!("Found {} bookmark(s):\n\n", bookmarks.len());
        
        for (i, bookmark) in bookmarks.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\n   URL: {}\n   ID: {}\n   Status: {:?}",
                i + 1,
                bookmark.title,
                bookmark.url,
                &bookmark.id[..8],
                bookmark.reading_status
            ));
            
            if let Some(priority) = bookmark.priority_rating {
                output.push_str(&format!(" | Priority: {}", priority));
            }
            
            if !bookmark.tags.is_empty() {
                output.push_str(&format!(" | Tags: {}", bookmark.tags.join(", ")));
            }
            
            if let Some(ref author) = bookmark.author {
                output.push_str(&format!("\n   Author: {}", author));
            }
            
            if !bookmark.notes.is_empty() {
                output.push_str(&format!("\n   Notes: {} note(s)", bookmark.notes.len()));
            }
            
            output.push_str("\n\n");
        }
        
        output
    }
}

#[async_trait::async_trait]
impl CommandHandler for SearchCommand {
    async fn execute(&self, repository: &mut dyn BookmarkRepository, format: OutputFormat) -> BookmarkResult<()> {
        let filters = self.build_filters()?;
        let bookmarks = repository.find_all(Some(filters)).await?;
        
        match format {
            OutputFormat::Json => {
                let response = SearchResponse {
                    total_count: bookmarks.len(),
                    query_summary: self.generate_query_summary(),
                    results: bookmarks,
                };
                output::print_response(format, response)?;
            }
            OutputFormat::Human => {
                let formatted_output = self.format_human_output(&bookmarks);
                print!("{}", formatted_output);
            }
        }
        
        Ok(())
    }
}

pub async fn handle_search_command(
    args: SearchArgs,
    repository: &mut dyn BookmarkRepository,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let command = SearchCommand::new(args);
    command.execute(repository, format).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::Bookmark;

    #[tokio::test]
    async fn test_search_with_text_query() {
        let mut repo = MockBookmarkRepository::new();
        
        let bookmark1 = Bookmark::new("https://rust-lang.org", "Rust Programming Language").unwrap();
        let bookmark2 = Bookmark::new("https://python.org", "Python Programming").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let args = SearchArgs {
            query: Some("rust".to_string()),
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        
        let result = handle_search_command(args, &mut repo, OutputFormat::Human).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_with_tags() {
        let mut repo = MockBookmarkRepository::new();
        
        let bookmark1 = Bookmark::new("https://example.com", "Example 1").unwrap()
            .with_tags(vec!["rust".to_string(), "programming".to_string()]);
        let bookmark2 = Bookmark::new("https://example2.com", "Example 2").unwrap()
            .with_tags(vec!["python".to_string(), "programming".to_string()]);
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let args = SearchArgs {
            query: None,
            tags: Some(vec!["rust".to_string()]),
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        
        let result = handle_search_command(args, &mut repo, OutputFormat::Human).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_with_status_filter() {
        let mut repo = MockBookmarkRepository::new();
        
        let mut bookmark1 = Bookmark::new("https://example.com", "Example 1").unwrap();
        bookmark1.reading_status = ReadingStatus::Completed;
        
        let bookmark2 = Bookmark::new("https://example2.com", "Example 2").unwrap();
        // bookmark2 has default ReadingStatus::Unread
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let args = SearchArgs {
            query: None,
            tags: None,
            status: Some(ReadingStatus::Completed),
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        
        let result = handle_search_command(args, &mut repo, OutputFormat::Human).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_priority_range() {
        let args = SearchArgs {
            query: None,
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        let command = SearchCommand::new(args);
        
        // Test range format
        assert_eq!(command.parse_priority_range("1-5").unwrap(), (1, 5));
        assert_eq!(command.parse_priority_range("3-4").unwrap(), (3, 4));
        
        // Test single value format
        assert_eq!(command.parse_priority_range("3").unwrap(), (3, 3));
        
        // Test invalid ranges
        assert!(command.parse_priority_range("0-5").is_err());
        assert!(command.parse_priority_range("1-6").is_err());
        assert!(command.parse_priority_range("5-1").is_err());
        assert!(command.parse_priority_range("abc").is_err());
        assert!(command.parse_priority_range("1-abc").is_err());
    }

    #[test]
    fn test_parse_date() {
        let args = SearchArgs {
            query: None,
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        let command = SearchCommand::new(args);
        
        // Test valid date formats
        assert!(command.parse_date("01-15-2023").is_ok());
        assert!(command.parse_date("12-31-2023").is_ok());
        assert!(command.parse_date("06-01-2024").is_ok());
        
        // Test invalid date formats
        assert!(command.parse_date("2023-01-01").is_err());
        assert!(command.parse_date("01/15/2023").is_err());
        assert!(command.parse_date("invalid-date").is_err());
        assert!(command.parse_date("").is_err());
        assert!(command.parse_date("13-01-2023").is_err()); // Invalid month
        assert!(command.parse_date("01-32-2023").is_err()); // Invalid day
    }

    #[test]
    fn test_build_filters() {
        let args = SearchArgs {
            query: Some("rust".to_string()),
            tags: Some(vec!["programming".to_string()]),
            status: Some(ReadingStatus::Unread),
            priority: Some("3-5".to_string()),
            since: Some("01-01-2023".to_string()),
            until: Some("12-31-2023".to_string()),
            published_since: None,
            published_until: None,
            sort_by: Some(SortBy::Title),
            sort_order: SortDirection::Ascending,
        };
        let command = SearchCommand::new(args);
        
        let filters = command.build_filters().unwrap();
        
        assert_eq!(filters.text_query, Some("rust".to_string()));
        assert_eq!(filters.tags, Some(vec!["programming".to_string()]));
        assert_eq!(filters.reading_status, Some(ReadingStatus::Unread));
        assert_eq!(filters.priority_range, Some((3, 5)));
        assert!(filters.bookmarked_since.is_some());
        assert!(filters.bookmarked_until.is_some());
        assert_eq!(filters.sort_by, Some(SortBy::Title));
        assert_eq!(filters.sort_order, Some(SortDirection::Ascending));
    }

    #[test]
    fn test_format_human_output_empty() {
        let args = SearchArgs {
            query: None,
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        let command = SearchCommand::new(args);
        
        let output = command.format_human_output(&[]);
        assert!(output.contains("No bookmarks found"));
    }

    #[test]
    fn test_format_human_output_with_results() {
        let args = SearchArgs {
            query: None,
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        let command = SearchCommand::new(args);
        
        let bookmark = Bookmark::new("https://example.com", "Example Title").unwrap();
        let output = command.format_human_output(&[bookmark]);
        
        assert!(output.contains("Found 1 bookmark"));
        assert!(output.contains("Example Title"));
        assert!(output.contains("https://example.com"));
    }

    #[test]
    fn test_generate_query_summary() {
        let args = SearchArgs {
            query: Some("rust".to_string()),
            tags: Some(vec!["programming".to_string()]),
            status: Some(ReadingStatus::Unread),
            priority: Some("3-5".to_string()),
            since: Some("01-01-2023".to_string()),
            until: Some("12-31-2023".to_string()),
            published_since: None,
            published_until: None,
            sort_by: Some(SortBy::Title),
            sort_order: SortDirection::Ascending,
        };
        let command = SearchCommand::new(args);
        
        let summary = command.generate_query_summary();
        
        assert_eq!(summary.text_query, Some("rust".to_string()));
        assert_eq!(summary.tags_filter, Some(vec!["programming".to_string()]));
        assert_eq!(summary.status_filter, Some(ReadingStatus::Unread));
        assert_eq!(summary.priority_filter, Some("3-5".to_string()));
        assert!(summary.date_range.is_some());
        assert!(summary.sort_info.is_some());
    }

    #[tokio::test]
    async fn test_search_command_json_output() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        repo.create(bookmark).await.unwrap();
        
        let args = SearchArgs {
            query: Some("example".to_string()),
            tags: None,
            status: None,
            priority: None,
            since: None,
            until: None,
            published_since: None,
            published_until: None,
            sort_by: None,
            sort_order: SortDirection::Descending,
        };
        
        let result = handle_search_command(args, &mut repo, OutputFormat::Json).await;
        assert!(result.is_ok());
    }
}