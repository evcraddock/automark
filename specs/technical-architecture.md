# Technical Architecture

## Core Technology Decisions

**CRDT Library**: Automerge for conflict-free local-first data synchronization

**Architecture Pattern**: Repository + Automerge Adapter Pattern
- Clean separation between business logic and CRDT implementation
- BookmarkRepository interface with AutomergeBookmarkRepository implementation
- Domain objects isolated from CRDT complexity

**Programming Language**: Rust
- Official automerge-rs library support
- Excellent CLI (clap) and TUI (ratatui) libraries
- Single binary deployment, cross-platform compatibility

**Application Type**: CLI/TUI (Command Line Interface with Text User Interface)
- Terminal-based application for developer-friendly usage
- TUI for interactive bookmark browsing and management
- CLI commands for scripting and automation

**Key Libraries**:
- `automerge` for CRDT-based local-first data synchronization
- `clap` for CLI argument parsing and command structure
- `ratatui` for terminal user interface components

## Project Structure

LLM-optimized modular architecture for independent development:

```
src/
├── main.rs                    # Minimal CLI bootstrap
├── types/
│   ├── mod.rs                 # Re-exports
│   ├── bookmark.rs            # Bookmark struct + validation
│   ├── tag.rs                 # Tag handling
│   └── config.rs              # Configuration types
├── traits/
│   ├── mod.rs                 # All trait definitions
│   ├── repository.rs          # BookmarkRepository trait
│   └── metadata_extractor.rs  # MetadataExtractor trait
├── adapters/
│   ├── automerge_repo.rs      # Automerge implementation (self-contained)
│   ├── web_extractor.rs       # HTTP metadata extraction
│   └── file_storage.rs        # File system operations
├── commands/
│   ├── add.rs                 # Each command is isolated
│   ├── list.rs
│   ├── delete.rs              # Delete bookmarks by ID
│   ├── search.rs
│   └── sync.rs
└── tui/
    ├── app.rs                 # TUI app state
    ├── components/            # Individual UI components
    └── handlers/              # Event handlers
```

**Benefits**: Each module can be developed independently with minimal context requirements.

## Data Models & Types

### Core Domain Types

**Bookmark Structure:**
- Contains unique ID, URL, title, optional author
- Has collection of tags (automatically normalized to lowercase)
- Includes optional publish date and auto-generated bookmarked date
- Contains collection of immutable Notes with timestamps
- Has reading status (Unread, Reading, Completed)
- Optional priority rating (1-5 scale)

**Note Structure:**
- Immutable notes with unique ID, content, and creation timestamp
- Notes can only be added or deleted, never modified

**Supporting Types:**
- ReadingStatus enum with three states
- SortOrder enum for different sorting options (PublishDate, BookmarkedDate, Title)
- BookmarkFilters struct for search/filter criteria
- BookmarkError enum for domain-specific error types
- BookmarkResult type alias for consistent error handling

**Validation Rules:**
- URL must be valid format
- Priority rating must be 1-5 if provided
- Tags automatically converted to lowercase
- Notes are immutable (add/delete only)

## Core Components/Modules

### Repository Layer

**BookmarkRepository Trait:**
- Async trait defining all bookmark operations
- CRUD operations: create, find_by_id, find_all, update, delete
- Specialized queries: search by text, find by tags
- Accepts optional filters for find_all operations
- Returns domain BookmarkResult types
- Implemented by AutomergeBookmarkRepository adapter

**Automerge-Specific Data Operations:**
- Collections (tags, notes) use Automerge sequences for CRDT semantics
- Bookmark updates merge field-by-field to preserve concurrent modifications
- Delete operations use tombstone markers to maintain CRDT consistency
- Complex nested updates decomposed into atomic CRDT operations
- All mutations produce new document states with causal ordering

### Metadata Extraction

**MetadataExtractor Trait:**
- Single async method to extract metadata from URLs
- Returns ExtractedMetadata struct with optional fields
- Handles timeout settings for slow websites
- Implemented by WebExtractor using HTTP requests and HTML parsing

**ExtractedMetadata Structure:**
- Contains optional title, author, and publish date
- All fields optional as not all websites provide complete metadata

### CLI Command Structure

**Command Handler Pattern:**
- Each command implemented as separate async handler function
- Handlers accept command-specific arguments struct
- Take repository and service dependencies as parameters
- Return BookmarkResult for consistent error handling
- Isolated in separate modules for independent development

### TUI Components

**TuiApp State Management:**
- Central state holding current bookmarks, selection, and view mode
- Manages search query and navigation state
- ViewMode enum defines different UI screens
- State transitions based on user input events

**Component Organization:**
- Separate component modules: bookmark_list, bookmark_detail, search_bar
- Each component renders specific UI section
- Components receive app state and render using ratatui widgets

## Interface Contracts

### CLI Command Arguments

**Command Structure:**
- Top-level Commands enum with variants for each operation
- Special Tui variant to launch interactive mode
- Each command has dedicated Args struct with clap derives

**Argument Patterns:**
- Required positional arguments (like URL for add command)
- Optional flags with short and long forms
- Vector arguments for multiple values (tags)
- Type-safe parsing using clap derives

**Output Format Options:**
- Global `--json` flag for machine-readable output
- Human-readable format by default (tables, formatted text)
- JSON output includes structured data with consistent schema
- Error responses also available in JSON format with error codes
- JSON schema versioning for API compatibility

### TUI Key Bindings
- `j/k` - Navigate up/down
- `Enter` - Select/open details
- `/` - Search mode
- `Esc` - Back/cancel
- `q` - Quit
- `a` - Add bookmark
- `d` - Delete bookmark
- `e` - Edit bookmark

### Error Handling

**Error Display Strategy:**
- BookmarkError implements Display trait for user-friendly messages
- Different error variants provide contextual information
- CLI displays errors directly to stderr (human format) or stdout (JSON format)
- TUI shows errors as temporary status messages at bottom of screen

**JSON Error Format:**
- Consistent error schema: `{"error": {"code": "ERROR_TYPE", "message": "description", "details": {...}}}`
- Standard error codes for programmatic handling
- Optional details object for additional context
- HTTP-style error codes where applicable

**TUI Message System:**
- TuiMessage enum for different message types (Success, Error, Info)
- Temporary display with automatic timeout
- Non-blocking user experience

## External Dependencies

### HTTP & Metadata Extraction
- `reqwest` for HTTP requests to fetch web pages
- `scraper` for HTML parsing and metadata extraction  
- `url` crate for URL validation and parsing
- Timeout settings for slow websites

### File System & Configuration
- `tokio::fs` for async file operations
- `config` crate for configuration management (supports multiple formats)
- `dirs` crate for cross-platform directory discovery
- `serde_json` for JSON serialization/deserialization
- `serde` with derive macros for JSON output formatting

### Data Storage Locations
- **Data**: `~/.local/share/automark/` (Automerge document storage)
- **Config**: `~/.config/automark/` (configuration files)
- **Format**: Automerge binary format for bookmark data

**Automerge Document Structure:**
- Root document contains map of bookmark IDs to bookmark objects
- Each bookmark stored as nested map with scalar fields (id, url, title, etc.)
- Tags collection stored as Automerge sequence (list CRDT)
- Notes collection stored as sequence of immutable note objects
- Timestamps use Automerge's built-in counter type for consistency
- All collections use appropriate CRDT types for conflict-free merging

**CRDT Conflict Resolution Strategy:**
- **Scalar conflicts**: Last-writer-wins using Automerge actor ordering
- **Collection conflicts**: Set union for tags, sequence ordering for notes
- **Structural conflicts**: Field-level resolution maintains referential integrity
- **Delete conflicts**: Tombstone markers prevent resurrection of deleted items
- **Concurrent updates**: Automerge automatically merges non-conflicting changes

## Local-First Synchronization Architecture

**CRDT-Based Synchronization:**
- Each device maintains its own Automerge document
- Changes are applied locally first (offline-first operation)
- Documents can be merged without central coordination
- Automatic conflict resolution through Automerge's CRDT semantics

**Peer Discovery & Sync Protocols:**
- Initial implementation: Manual sync via file sharing
- Future: Network-based peer discovery (mDNS, manual peer configuration)
- Sync command triggers document exchange and merge operations
- Bidirectional sync ensures all peers converge to same state

**Offline-First Operational Model:**
- All operations work without network connectivity
- Local changes are immediately persisted to Automerge document
- Sync operations are explicit and user-initiated
- Graceful degradation when peers are unavailable

**Document Versioning & History:**
- Automerge maintains complete operation history
- Each change creates new document state with causal ordering
- No data loss during concurrent modifications
- Version vectors track document evolution across peers

## Configuration & Settings

**Configuration Management:**
- Always require a valid `data_dir` setting
- If `~/.config/automark/config.toml` doesn't exist, create it with default values
- Default `data_dir` is `~/.local/share/automark`
- Expand `~` to actual home directory path when loading config
- Create the data directory if it doesn't exist
- Use the `config` crate to handle loading/parsing

**Configuration Structure:**
- Simple TOML format with `[storage]` section
- Only `data_dir` setting for initial version
- Extensible structure for future settings
- Sync peer configuration (addresses, discovery methods)

## Testing Strategy

**Test-Driven Development (TDD):**
- All code must be written using TDD approach
- Tests written first, then implementation
- All code must pass tests before being considered complete

**Unit Testing Focus:**
- Each module independently testable (leveraging modular structure)
- Mock implementations of `BookmarkRepository` and `MetadataExtractor` traits for testing
- Test validation rules in domain types (URL format, rating ranges, tag normalization)
- Test CLI argument parsing and command dispatch
- Test error handling and display formatting
- Test TUI state management and view transitions

**Testing Tools:**
- Standard Rust `#[cfg(test)]` and `#[test]` attributes
- `mockall` crate for trait mocking if needed
- Temporary directories for file system testing