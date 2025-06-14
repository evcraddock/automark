use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::traits::BookmarkRepository;
use crate::types::{Bookmark, BookmarkResult};
use crate::tui::app::{TuiApp, ViewMode, TuiMessage};
use std::process::Command;

/// Handle keyboard events based on current application mode
pub async fn handle_key_event(
    key: KeyEvent,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    match app.mode {
        ViewMode::List => handle_list_mode_keys(key, app, repository).await,
        ViewMode::Detail => handle_detail_mode_keys(key, app),
        ViewMode::Search => handle_search_mode_keys(key, app, repository).await,
        ViewMode::Add => handle_add_mode_keys(key, app, repository).await,
        ViewMode::Delete => handle_delete_mode_keys(key, app, repository).await,
    }
}

/// Handle keys in list view mode
async fn handle_list_mode_keys(
    key: KeyEvent,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.navigate_down();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.navigate_up();
        }
        KeyCode::Enter => {
            if let Some(bookmark) = app.selected_bookmark() {
                // Open URL in default browser
                if let Err(e) = open_url(&bookmark.url) {
                    app.set_message(TuiMessage::Error(format!("Failed to open URL: {}", e)));
                } else {
                    app.set_message(TuiMessage::Success(format!("Opened: {}", bookmark.title)));
                }
            }
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if app.selected_bookmark().is_some() {
                app.mode = ViewMode::Detail;
            }
        }
        KeyCode::Char('/') => {
            app.mode = ViewMode::Search;
            app.search_query.clear();
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.mode = ViewMode::Add;
            app.clear_input();
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if app.selected_bookmark().is_some() {
                app.mode = ViewMode::Delete;
            } else {
                app.set_message(TuiMessage::Error("No bookmark selected".to_string()));
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.refresh_bookmarks(repository).await?;
            app.set_message(TuiMessage::Success("Bookmarks refreshed".to_string()));
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if app.filters.is_some() {
                app.clear_search(repository).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys in detail view mode
fn handle_detail_mode_keys(key: KeyEvent, app: &mut TuiApp) -> BookmarkResult<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Esc | KeyCode::Char('b') => {
            app.mode = ViewMode::List;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys in search mode
async fn handle_search_mode_keys(
    key: KeyEvent,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    match key.code {
        KeyCode::Enter => {
            app.apply_search(repository).await?;
            app.mode = ViewMode::List;
        }
        KeyCode::Esc => {
            app.mode = ViewMode::List;
            app.search_query.clear();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys in add bookmark mode
async fn handle_add_mode_keys(
    key: KeyEvent,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    match key.code {
        KeyCode::Enter => {
            let url = app.input_buffer.trim();
            if !url.is_empty() {
                match add_bookmark(url, repository).await {
                    Ok(bookmark) => {
                        app.refresh_bookmarks(repository).await?;
                        app.set_message(TuiMessage::Success(format!("Added bookmark: {}", bookmark.title)));
                        app.mode = ViewMode::List;
                        app.clear_input();
                    }
                    Err(e) => {
                        app.set_message(TuiMessage::Error(format!("Failed to add bookmark: {}", e)));
                    }
                }
            } else {
                app.set_message(TuiMessage::Error("URL cannot be empty".to_string()));
            }
        }
        KeyCode::Esc => {
            app.mode = ViewMode::List;
            app.clear_input();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Char(c) => {
            app.add_char_to_input(c);
        }
        KeyCode::Backspace => {
            app.remove_char_from_input();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        }
        _ => {}
    }
    Ok(())
}

/// Handle keys in delete confirmation mode
async fn handle_delete_mode_keys(
    key: KeyEvent,
    app: &mut TuiApp,
    repository: &mut dyn BookmarkRepository,
) -> BookmarkResult<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(bookmark) = app.selected_bookmark() {
                let bookmark_id = bookmark.id.clone();
                let title = bookmark.title.clone();
                
                match repository.delete(&bookmark_id).await {
                    Ok(_) => {
                        app.refresh_bookmarks(repository).await?;
                        app.set_message(TuiMessage::Success(format!("Deleted bookmark: {}", title)));
                    }
                    Err(e) => {
                        app.set_message(TuiMessage::Error(format!("Failed to delete bookmark: {}", e)));
                    }
                }
            }
            app.mode = ViewMode::List;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        _ => {
            // Any other key cancels the delete
            app.mode = ViewMode::List;
        }
    }
    Ok(())
}

/// Helper function to add a bookmark with basic title extraction
async fn add_bookmark(url: &str, repository: &mut dyn BookmarkRepository) -> BookmarkResult<Bookmark> {
    // Try to create bookmark with URL validation
    let mut bookmark = Bookmark::new(url, url)?;
    
    // For TUI, we'll use the URL as the title initially
    // In a real implementation, you might want to fetch the page title
    if let Some(domain) = extract_domain(url) {
        bookmark.title = format!("Bookmark from {}", domain);
    }
    
    repository.create(bookmark).await
}

/// Extract domain from URL for basic title generation
fn extract_domain(url: &str) -> Option<String> {
    url.split("://")
        .nth(1)?
        .split('/')
        .next()
        .map(|s| s.to_string())
}

/// Open URL in the default browser
fn open_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(format!("Opening URLs is not supported on this platform").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::repository::MockBookmarkRepository;
    use crate::tui::app::{TuiApp, ViewMode};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[tokio::test]
    async fn test_list_mode_navigation() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark1 = Bookmark::new("https://example1.com", "Example 1").unwrap();
        let bookmark2 = Bookmark::new("https://example2.com", "Example 2").unwrap();
        
        repo.create(bookmark1).await.unwrap();
        repo.create(bookmark2).await.unwrap();
        
        let mut app = TuiApp::new(&repo).await.unwrap();
        
        // Test navigation down
        let key = create_test_key_event(KeyCode::Down);
        handle_key_event(key, &mut app, &mut repo).await.unwrap();
        assert_eq!(app.selected_index, Some(1));
        
        // Test navigation up
        let key = create_test_key_event(KeyCode::Up);
        handle_key_event(key, &mut app, &mut repo).await.unwrap();
        assert_eq!(app.selected_index, Some(0));
    }

    #[tokio::test]
    async fn test_mode_transitions() {
        let mut repo = MockBookmarkRepository::new();
        let mut app = TuiApp::new(&repo).await.unwrap();
        
        assert_eq!(app.mode, ViewMode::List);
        
        // Test entering search mode
        let key = create_test_key_event(KeyCode::Char('/'));
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert_eq!(app.mode, ViewMode::Search);
        
        // Test escape from search mode
        let key = create_test_key_event(KeyCode::Esc);
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert_eq!(app.mode, ViewMode::List);
    }

    #[tokio::test]
    async fn test_quit_functionality() {
        let mut repo = MockBookmarkRepository::new();
        let mut app = TuiApp::new(&repo).await.unwrap();
        
        assert!(!app.should_quit);
        
        let key = create_test_key_event(KeyCode::Char('q'));
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert!(app.should_quit);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://www.google.com"), Some("www.google.com".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }

    #[tokio::test]
    async fn test_search_input() {
        let mut repo = MockBookmarkRepository::new();
        let mut app = TuiApp::new(&repo).await.unwrap();
        app.mode = ViewMode::Search;
        
        // Test character input
        let key = create_test_key_event(KeyCode::Char('t'));
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert_eq!(app.search_query, "t");
        
        // Test backspace
        let key = create_test_key_event(KeyCode::Backspace);
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert_eq!(app.search_query, "");
    }

    #[tokio::test]
    async fn test_enter_opens_url() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        repo.create(bookmark).await.unwrap();
        
        let mut app = TuiApp::new(&repo).await.unwrap();
        app.selected_index = Some(0);
        
        // Test Enter key opens URL (we can't actually test browser opening, but we can test the code path)
        let key = create_test_key_event(KeyCode::Enter);
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        // Should not crash and may have set a message
    }

    #[tokio::test]
    async fn test_e_opens_details() {
        let mut repo = MockBookmarkRepository::new();
        let bookmark = Bookmark::new("https://example.com", "Example").unwrap();
        repo.create(bookmark).await.unwrap();
        
        let mut app = TuiApp::new(&repo).await.unwrap();
        app.selected_index = Some(0);
        assert_eq!(app.mode, ViewMode::List);
        
        // Test 'e' key opens details
        let key = create_test_key_event(KeyCode::Char('e'));
        handle_key_event(key, &mut app, &mut repo as &mut dyn BookmarkRepository).await.unwrap();
        assert_eq!(app.mode, ViewMode::Detail);
    }

    #[test]
    fn test_open_url_function() {
        // Test that open_url function doesn't panic with valid URLs
        // We can't actually test browser opening in CI, but we can test the function exists
        let result = open_url("https://example.com");
        // The function should complete without panicking
        // Result may be Ok or Err depending on the environment, which is fine
        match result {
            Ok(_) => {}, // Success case
            Err(_) => {}, // Expected failure in CI environment
        }
    }
}