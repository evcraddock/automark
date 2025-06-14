use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::types::Bookmark;

/// Render the bookmark list component
pub fn render_bookmark_list(
    f: &mut Frame,
    area: Rect,
    bookmarks: &[Bookmark],
    list_state: &mut ListState,
    show_details: bool,
) {
    let items: Vec<ListItem> = bookmarks
        .iter()
        .enumerate()
        .map(|(i, bookmark)| {
            let content = if show_details {
                format!(
                    "{:<3} {} - {}\n    Added: {} | Tags: {}",
                    i + 1,
                    bookmark.title,
                    bookmark.url,
                    bookmark.bookmarked_date.format("%Y-%m-%d"),
                    if bookmark.tags.is_empty() {
                        "none".to_string()
                    } else {
                        bookmark.tags.join(", ")
                    }
                )
            } else {
                format!("{:<3} {} - {}", i + 1, bookmark.title, bookmark.url)
            };
            ListItem::new(content)
        })
        .collect();

    let title = format!("Bookmarks ({})", bookmarks.len());
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

/// Get the display text for a bookmark in the list
pub fn format_bookmark_item(bookmark: &Bookmark, index: usize, compact: bool) -> String {
    if compact {
        format!("{:<3} {}", index + 1, bookmark.title)
    } else {
        let mut text = format!("{:<3} {}\n", index + 1, bookmark.title);
        text.push_str(&format!("    {}\n", bookmark.url));
        
        if !bookmark.tags.is_empty() {
            text.push_str(&format!("    Tags: {}\n", bookmark.tags.join(", ")));
        }
        
        text.push_str(&format!(
            "    Added: {}",
            bookmark.bookmarked_date.format("%Y-%m-%d %H:%M")
        ));
        
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Bookmark;

    #[test]
    fn test_format_bookmark_item_compact() {
        let bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        let formatted = format_bookmark_item(&bookmark, 0, true);
        assert!(formatted.contains("1   Test Bookmark"));
        assert!(!formatted.contains("https://example.com"));
    }

    #[test]
    fn test_format_bookmark_item_detailed() {
        let bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        let formatted = format_bookmark_item(&bookmark, 0, false);
        assert!(formatted.contains("1   Test Bookmark"));
        assert!(formatted.contains("https://example.com"));
        assert!(formatted.contains("Added:"));
    }
}