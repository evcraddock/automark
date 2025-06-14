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
        ViewMode::List => "↑/↓ or j/k: navigate | Enter: open URL | e: details | /: search | a: add | d: delete | q: quit",
        ViewMode::Detail => "Esc: back to list | q: quit",
        ViewMode::Search => "Type to search | Enter: apply search | Esc: cancel",
        ViewMode::Add => "Type URL | Enter: add bookmark | Esc: cancel",
        ViewMode::Delete => "y: confirm delete | any other key: cancel",
    }
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

}