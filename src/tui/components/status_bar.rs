use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{TuiMessage, ViewMode};

/// Render the status bar component with messages and key hints
pub fn render_status_bar(f: &mut Frame, area: Rect, mode: &ViewMode, message: Option<&TuiMessage>) {
    let mut spans = vec![];

    // Show message if present
    if let Some(msg) = message {
        spans.push(Span::styled(
            msg.content(),
            Style::default().fg(msg.color()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" | "));
    }

    // Show key hints based on mode
    let hints = get_key_hints(mode);
    spans.push(Span::styled(hints, Style::default().fg(Color::Gray)));

    let status = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
}

/// Get key hints for the current mode
fn get_key_hints(mode: &ViewMode) -> &'static str {
    match mode {
        ViewMode::List => "↑/↓ or j/k: navigate | Enter: details | /: search | a: add | d: delete | q: quit",
        ViewMode::Detail => "Esc: back to list | q: quit",
        ViewMode::Search => "Type to search | Enter: apply search | Esc: cancel",
        ViewMode::Add => "Type URL | Enter: add bookmark | Esc: cancel",
        ViewMode::Delete => "y: confirm delete | any other key: cancel",
    }
}

/// Render a simple status message
pub fn render_status_message(f: &mut Frame, area: Rect, message: &str, color: Color) {
    let status = Paragraph::new(message)
        .style(Style::default().fg(color))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
}

/// Render bookmark count and filter info
pub fn render_bookmark_info(f: &mut Frame, area: Rect, total_count: usize, filtered_count: Option<usize>) {
    let info_text = if let Some(filtered) = filtered_count {
        format!("Showing {} of {} bookmarks", filtered, total_count)
    } else {
        format!("{} bookmarks", total_count)
    };

    let info = Paragraph::new(info_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Info"));
    f.render_widget(info, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{TuiMessage, ViewMode};
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    #[test]
    fn test_status_bar_list_mode() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 3);
            render_status_bar(f, area, &ViewMode::List, None);
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }

    #[test]
    fn test_status_bar_with_message() {
        let backend = TestBackend::new(80, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        let message = TuiMessage::Success("Test message".to_string());
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 80, 3);
            render_status_bar(f, area, &ViewMode::List, Some(&message));
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }

    #[test]
    fn test_key_hints_for_all_modes() {
        assert!(!get_key_hints(&ViewMode::List).is_empty());
        assert!(!get_key_hints(&ViewMode::Detail).is_empty());
        assert!(!get_key_hints(&ViewMode::Search).is_empty());
        assert!(!get_key_hints(&ViewMode::Add).is_empty());
        assert!(!get_key_hints(&ViewMode::Delete).is_empty());
    }

    #[test]
    fn test_bookmark_info_rendering() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 40, 3);
            render_bookmark_info(f, area, 100, Some(25));
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }

    #[test]
    fn test_status_message_rendering() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 40, 3);
            render_status_message(f, area, "Loading...", Color::Yellow);
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }
}