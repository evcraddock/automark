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
