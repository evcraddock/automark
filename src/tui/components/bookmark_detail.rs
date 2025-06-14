use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::types::Bookmark;

/// Render the bookmark detail component
pub fn render_bookmark_detail(f: &mut Frame, area: Rect, bookmark: &Bookmark) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // URL
            Constraint::Length(3), // Metadata
            Constraint::Min(0),    // Notes/Description
        ])
        .split(area);

    // Title
    let title = Paragraph::new(bookmark.title.as_str())
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Title"))
        .wrap(Wrap { trim: true });
    f.render_widget(title, chunks[0]);

    // URL
    let url = Paragraph::new(bookmark.url.as_str())
        .style(Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED))
        .block(Block::default().borders(Borders::ALL).title("URL"))
        .wrap(Wrap { trim: true });
    f.render_widget(url, chunks[1]);

    // Metadata (date, tags, reading status, priority)
    let mut metadata_lines = vec![];
    
    // Date added
    metadata_lines.push(Line::from(vec![
        Span::styled("Added: ", Style::default().fg(Color::Gray)),
        Span::styled(
            bookmark.bookmarked_date.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            Style::default().fg(Color::Green),
        ),
    ]));

    // Tags
    if !bookmark.tags.is_empty() {
        metadata_lines.push(Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(Color::Gray)),
            Span::styled(
                bookmark.tags.join(", "),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }

    // Reading status and priority
    let status_text = format!("{:?}", bookmark.reading_status);
    let priority_text = if let Some(rating) = bookmark.priority_rating {
        format!("Priority: {}/5", rating)
    } else {
        "No priority set".to_string()
    };
    
    metadata_lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(status_text, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(priority_text, Style::default().fg(Color::Magenta)),
    ]));

    let metadata = Paragraph::new(metadata_lines)
        .block(Block::default().borders(Borders::ALL).title("Metadata"));
    f.render_widget(metadata, chunks[2]);

    // Notes and additional info
    let mut content_lines = vec![];
    
    // Author if available
    if let Some(ref author) = bookmark.author {
        content_lines.push(Line::from(vec![
            Span::styled("Author: ", Style::default().fg(Color::Gray)),
            Span::styled(author.clone(), Style::default().fg(Color::Green)),
        ]));
    }

    // Publish date if available
    if let Some(ref publish_date) = bookmark.publish_date {
        content_lines.push(Line::from(vec![
            Span::styled("Published: ", Style::default().fg(Color::Gray)),
            Span::styled(
                publish_date.format("%Y-%m-%d").to_string(),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    // Notes
    if !bookmark.notes.is_empty() {
        if !content_lines.is_empty() {
            content_lines.push(Line::from(""));
        }
        content_lines.push(Line::from(
            Span::styled("Notes:", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
        ));
        
        for (i, note) in bookmark.notes.iter().enumerate() {
            content_lines.push(Line::from(vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                Span::raw(note.content.clone()),
            ]));
            content_lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    format!("Added: {}", note.created_at.format("%Y-%m-%d %H:%M")),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    if content_lines.is_empty() {
        content_lines.push(Line::from(
            Span::styled("No additional information available", Style::default().fg(Color::DarkGray))
        ));
    }

    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(content, chunks[3]);
}

/// Render a simple "no bookmark selected" message
pub fn render_no_bookmark_selected(f: &mut Frame, area: Rect) {
    let message = Paragraph::new("No bookmark selected\n\nPress Esc to go back to the list")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(message, area);
}

#[cfg(test)]
mod tests {
    use crate::types::{Bookmark, ReadingStatus};

    #[test]
    fn test_bookmark_with_all_fields() {
        let mut bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        bookmark.tags = vec!["test".to_string(), "example".to_string()];
        bookmark.reading_status = ReadingStatus::Reading;
        bookmark.priority_rating = Some(4);
        bookmark.author = Some("Test Author".to_string());
        
        // Test that the bookmark has all the expected fields
        assert_eq!(bookmark.title, "Test Bookmark");
        assert_eq!(bookmark.url, "https://example.com");
        assert_eq!(bookmark.tags.len(), 2);
        assert_eq!(bookmark.reading_status, ReadingStatus::Reading);
        assert_eq!(bookmark.priority_rating, Some(4));
        assert_eq!(bookmark.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_bookmark_with_notes() {
        let mut bookmark = Bookmark::new("https://example.com", "Test Bookmark").unwrap();
        let note_id = bookmark.add_note("This is a test note");
        
        assert_eq!(bookmark.notes.len(), 1);
        assert_eq!(bookmark.notes[0].content, "This is a test note");
        assert_eq!(bookmark.notes[0].id, note_id);
    }
}