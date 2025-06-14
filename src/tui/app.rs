use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult, BookmarkFilters};
use super::components::*;
use super::handlers::*;

/// Different view modes for the TUI application
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Main bookmark list view
    List,
    /// Detailed view of selected bookmark
    Detail,
    /// Search input mode
    Search,
    /// Add new bookmark mode
    Add,
    /// Delete confirmation mode
    Delete,
}

/// Message types for user feedback
#[derive(Debug, Clone, PartialEq)]
pub enum TuiMessage {
    Success(String),
    Error(String),
    Info(String),
}

impl TuiMessage {
    pub fn color(&self) -> Color {
        match self {
            TuiMessage::Success(_) => Color::Green,
            TuiMessage::Error(_) => Color::Red,
            TuiMessage::Info(_) => Color::Blue,
        }
    }

    pub fn content(&self) -> &str {
        match self {
            TuiMessage::Success(msg) | TuiMessage::Error(msg) | TuiMessage::Info(msg) => msg,
        }
    }
}

/// Main TUI application state
pub struct TuiApp {
    /// Current view mode
    pub mode: ViewMode,
    /// List of bookmarks
    pub bookmarks: Vec<Bookmark>,
    /// List state for navigation
    pub list_state: ListState,
    /// Currently selected bookmark index
    pub selected_index: Option<usize>,
    /// Search query input
    pub search_query: String,
    /// Current bookmark filters
    pub filters: Option<BookmarkFilters>,
    /// Message to display to user
    pub message: Option<TuiMessage>,
    /// Time when message was set
    pub message_time: Option<Instant>,
    /// Whether to quit the application
    pub should_quit: bool,
    /// Input buffer for add/edit operations
    pub input_buffer: String,
    /// Cursor position in input buffer
    pub cursor_position: usize,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new(repository: &dyn BookmarkRepository) -> BookmarkResult<Self> {
        let bookmarks = repository.find_all(None).await?;
        let mut list_state = ListState::default();
        if !bookmarks.is_empty() {
            list_state.select(Some(0));
        }

        let has_bookmarks = !bookmarks.is_empty();
        
        Ok(Self {
            mode: ViewMode::List,
            bookmarks,
            list_state,
            selected_index: if has_bookmarks { Some(0) } else { None },
            search_query: String::new(),
            filters: None,
            message: None,
            message_time: None,
            should_quit: false,
            input_buffer: String::new(),
            cursor_position: 0,
        })
    }

    /// Get the currently selected bookmark
    pub fn selected_bookmark(&self) -> Option<&Bookmark> {
        self.selected_index
            .and_then(|index| self.bookmarks.get(index))
    }

    /// Navigate up in the bookmark list
    pub fn navigate_up(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = if current == 0 {
            self.bookmarks.len() - 1
        } else {
            current - 1
        };
        
        self.selected_index = Some(new_index);
        self.list_state.select(Some(new_index));
    }

    /// Navigate down in the bookmark list
    pub fn navigate_down(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new_index = if current >= self.bookmarks.len() - 1 {
            0
        } else {
            current + 1
        };
        
        self.selected_index = Some(new_index);
        self.list_state.select(Some(new_index));
    }

    /// Set a message with current timestamp
    pub fn set_message(&mut self, message: TuiMessage) {
        self.message = Some(message);
        self.message_time = Some(Instant::now());
    }

    /// Clear message if timeout has passed
    pub fn update_message(&mut self) {
        if let Some(time) = self.message_time {
            if time.elapsed() > Duration::from_secs(3) {
                self.message = None;
                self.message_time = None;
            }
        }
    }

    /// Refresh bookmarks from repository
    pub async fn refresh_bookmarks(&mut self, repository: &dyn BookmarkRepository) -> BookmarkResult<()> {
        self.bookmarks = repository.find_all(self.filters.clone()).await?;
        
        // Update selection if needed
        if self.bookmarks.is_empty() {
            self.selected_index = None;
            self.list_state.select(None);
        } else if self.selected_index.is_none() || self.selected_index.unwrap() >= self.bookmarks.len() {
            self.selected_index = Some(0);
            self.list_state.select(Some(0));
        }
        
        Ok(())
    }

    /// Apply search filters
    pub async fn apply_search(&mut self, repository: &dyn BookmarkRepository) -> BookmarkResult<()> {
        if self.search_query.trim().is_empty() {
            self.filters = None;
        } else {
            self.filters = Some(BookmarkFilters {
                text_query: Some(self.search_query.clone()),
                ..Default::default()
            });
        }
        
        self.refresh_bookmarks(repository).await?;
        self.set_message(TuiMessage::Info(format!("Found {} bookmarks", self.bookmarks.len())));
        Ok(())
    }

    /// Clear search and show all bookmarks
    pub async fn clear_search(&mut self, repository: &dyn BookmarkRepository) -> BookmarkResult<()> {
        self.search_query.clear();
        self.filters = None;
        self.refresh_bookmarks(repository).await?;
        self.set_message(TuiMessage::Info("Search cleared".to_string()));
        Ok(())
    }

    /// Add character to input buffer
    pub fn add_char_to_input(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Remove character from input buffer
    pub fn remove_char_from_input(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Move cursor left in input buffer
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right in input buffer
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Clear input buffer
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }
}

/// Run the TUI application
pub async fn run_tui(repository: &mut dyn BookmarkRepository) -> BookmarkResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = TuiApp::new(repository).await?;

    // Run app loop
    let result = run_app(&mut terminal, &mut app, repository).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    loop {
        // Update message timeout
        app.update_message();

        // Draw UI
        terminal.draw(|f| ui(f, app))?;

        // Handle events
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(key, app, repository).await?;
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Draw the user interface
fn ui(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Draw header
    draw_header(f, chunks[0], app);

    // Draw main content based on mode
    match app.mode {
        ViewMode::List => draw_bookmark_list(f, chunks[1], app),
        ViewMode::Detail => draw_bookmark_detail(f, chunks[1], app),
        ViewMode::Search => draw_search_input(f, chunks[1], app),
        ViewMode::Add => draw_add_input(f, chunks[1], app),
        ViewMode::Delete => draw_delete_confirmation(f, chunks[1], app),
    }

    // Draw status bar
    draw_status_bar(f, chunks[2], app);
}

/// Draw the header with title and mode
fn draw_header(f: &mut Frame, area: Rect, app: &TuiApp) {
    let title = match app.mode {
        ViewMode::List => "Automark - Bookmark Manager",
        ViewMode::Detail => "Bookmark Details",
        ViewMode::Search => "Search Bookmarks",
        ViewMode::Add => "Add New Bookmark",
        ViewMode::Delete => "Delete Bookmark",
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

/// Draw the bookmark list
fn draw_bookmark_list(f: &mut Frame, area: Rect, app: &mut TuiApp) {
    render_bookmark_list(f, area, &app.bookmarks, &mut app.list_state, false);
}

/// Draw bookmark detail view
fn draw_bookmark_detail(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(bookmark) = app.selected_bookmark() {
        render_bookmark_detail(f, area, bookmark);
    } else {
        render_no_bookmark_selected(f, area);
    }
}

/// Draw search input
fn draw_search_input(f: &mut Frame, area: Rect, app: &TuiApp) {
    render_search_bar(f, area, &app.search_query, true);
}

/// Draw add bookmark input
fn draw_add_input(f: &mut Frame, area: Rect, app: &TuiApp) {
    render_add_input(f, area, &app.input_buffer, app.cursor_position);
}

/// Draw delete confirmation
fn draw_delete_confirmation(f: &mut Frame, area: Rect, app: &TuiApp) {
    if let Some(bookmark) = app.selected_bookmark() {
        let message = format!("Delete bookmark '{}'?\n\nPress 'y' to confirm, any other key to cancel", bookmark.title);
        render_confirmation_dialog(f, area, "Confirm Delete", &message);
    }
}

/// Draw status bar with messages and key hints
fn draw_status_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
    render_status_bar(f, area, &app.mode, app.message.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::types::Bookmark;

    #[tokio::test]
    async fn test_tui_app_creation() {
        let repo = MockBookmarkRepository::new();
        let app = TuiApp::new(&repo).await.unwrap();
        
        assert_eq!(app.mode, ViewMode::List);
        assert!(app.bookmarks.is_empty());
        assert_eq!(app.selected_index, None);
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn test_navigation() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example1.com", "Example 1").unwrap();
        let bookmark2 = Bookmark::new("https://example2.com", "Example 2").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let mut app = TuiApp::new(&repo).await.unwrap();
        
        assert_eq!(app.selected_index, Some(0));
        
        app.navigate_down();
        assert_eq!(app.selected_index, Some(1));
        
        app.navigate_down(); // Should wrap to 0
        assert_eq!(app.selected_index, Some(0));
        
        app.navigate_up(); // Should wrap to last item
        assert_eq!(app.selected_index, Some(1));
    }

    #[test]
    fn test_message_system() {
        let mut app = TuiApp {
            mode: ViewMode::List,
            bookmarks: vec![],
            list_state: ListState::default(),
            selected_index: None,
            search_query: String::new(),
            filters: None,
            message: None,
            message_time: None,
            should_quit: false,
            input_buffer: String::new(),
            cursor_position: 0,
        };

        app.set_message(TuiMessage::Success("Test message".to_string()));
        assert!(app.message.is_some());
        assert!(app.message_time.is_some());
        
        if let Some(TuiMessage::Success(msg)) = &app.message {
            assert_eq!(msg, "Test message");
        } else {
            panic!("Expected success message");
        }
    }

    #[test]
    fn test_input_buffer() {
        let mut app = TuiApp {
            mode: ViewMode::Add,
            bookmarks: vec![],
            list_state: ListState::default(),
            selected_index: None,
            search_query: String::new(),
            filters: None,
            message: None,
            message_time: None,
            should_quit: false,
            input_buffer: String::new(),
            cursor_position: 0,
        };

        app.add_char_to_input('h');
        app.add_char_to_input('i');
        assert_eq!(app.input_buffer, "hi");
        assert_eq!(app.cursor_position, 2);

        app.remove_char_from_input();
        assert_eq!(app.input_buffer, "h");
        assert_eq!(app.cursor_position, 1);

        app.clear_input();
        assert_eq!(app.input_buffer, "");
        assert_eq!(app.cursor_position, 0);
    }
}