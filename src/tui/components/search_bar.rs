use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the search input bar component
pub fn render_search_bar(f: &mut Frame, area: Rect, query: &str, is_active: bool) {
    let style = if is_active {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = if is_active {
        "Search (Enter to apply, Esc to cancel)"
    } else {
        "Search"
    };

    let search_input = Paragraph::new(query)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(search_input, area);
}

/// Render the add bookmark input component
pub fn render_add_input(f: &mut Frame, area: Rect, input: &str, cursor_pos: usize) {
    let mut display_text = input.to_string();
    
    // Add cursor indicator if within bounds
    if cursor_pos <= input.len() {
        display_text.insert(cursor_pos, '|');
    }

    let input_widget = Paragraph::new(display_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Enter URL (Enter to add, Esc to cancel)"));

    f.render_widget(input_widget, area);
}

/// Render a confirmation dialog
pub fn render_confirmation_dialog(f: &mut Frame, area: Rect, title: &str, message: &str) {
    let confirmation = Paragraph::new(message)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(confirmation, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, layout::Rect, Terminal};

    #[test]
    fn test_search_bar_rendering() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 40, 3);
            render_search_bar(f, area, "test query", true);
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }

    #[test]
    fn test_add_input_with_cursor() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 40, 3);
            render_add_input(f, area, "https://example.com", 5);
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }

    #[test]
    fn test_confirmation_dialog() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        
        terminal.draw(|f| {
            let area = Rect::new(0, 0, 40, 5);
            render_confirmation_dialog(f, area, "Confirm", "Are you sure?");
        }).unwrap();

        // Test passes if no panic occurs during rendering
    }
}