# Task 9: Implement TUI Foundation and Core Components

**GitHub Issue**: [#9](https://github.com/evcraddock/automark/issues/9)

## Objective
Create the terminal user interface foundation using ratatui with core app state management and basic components.

## Requirements

1. **Create TUI app structure** in `src/tui/app.rs`:
   - TuiApp struct with application state
   - ViewMode enum for different screens
   - Current bookmark selection and navigation state
   - Search query state and filter state
   - Message system for user feedback

2. **Implement app state management**:
   - Load bookmarks from repository on startup
   - Handle bookmark list navigation (up/down)
   - Manage view transitions between screens
   - Track selected bookmark and current mode

3. **Create core components** in `src/tui/components/`:
   - bookmark_list.rs: Main bookmark listing component
   - bookmark_detail.rs: Detailed bookmark view
   - search_bar.rs: Search input component
   - status_bar.rs: Status and message display

4. **Implement event handling** in `src/tui/handlers/`:
   - Keyboard event processing
   - Navigation between components
   - Mode switching (list, detail, search)
   - Quit and escape handling

5. **Define TUI key bindings**:
   - j/k for navigation up/down
   - Enter for selection/details
   - / for search mode
   - Esc for back/cancel
   - q for quit
   - a for add bookmark
   - d for delete bookmark
   - e for edit bookmark

6. **Implement message system**:
   - TuiMessage enum (Success, Error, Info)
   - Temporary message display with timeout
   - Status bar integration for messages
   - Non-blocking user experience

7. **Add TUI command integration**:
   - Add "tui" subcommand to CLI
   - Launch TUI mode from command line
   - Initialize TUI with repository access
   - Handle TUI startup and shutdown

8. **Write comprehensive tests** using TDD approach:
   - Test app state transitions
   - Test navigation and selection
   - Test keyboard event handling
   - Test message system functionality
   - Test component rendering logic

## Success Criteria
- TUI launches and displays bookmark list
- Navigation works with keyboard controls
- Components render correctly with ratatui
- Event handling is responsive and accurate
- Message system provides user feedback