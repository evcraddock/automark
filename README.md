# Automark

A powerful, local-first CLI bookmarking application built with Rust and Automerge.

## What it does

Automark is a feature-rich command-line tool for managing bookmarks locally with advanced search and filtering capabilities. It uses [Automerge](https://automerge.org/) for conflict-free replicated data storage, making it easy to sync bookmarks across devices in the future.

### Features

#### Core Functionality
- **Add bookmarks**: Save URLs with titles (auto-extracted or manual)
- **List bookmarks**: View all bookmarks with rich formatting and metadata
- **Delete bookmarks**: Remove bookmarks by full or partial ID
- **Search bookmarks**: Powerful search with advanced filtering and sorting

#### Metadata & Content
- **Automatic metadata extraction**: Fetches page titles, authors, and publish dates
- **Manual title override**: Specify custom titles or use `--no-fetch` flag
- **Rich bookmark data**: URLs, titles, authors, dates, tags, notes, reading status, and priority ratings

#### Search & Filtering
- **Text search**: Search across titles, URLs, authors, and notes
- **Tag filtering**: Filter by multiple tags with AND logic
- **Status filtering**: Filter by reading status (unread, reading, completed)
- **Priority filtering**: Filter by priority ratings (1-5) with range support
- **Date filtering**: Filter by bookmarked date and publish date ranges
- **Flexible sorting**: Sort by date, title, or priority with ascending/descending order

#### Configuration & Storage
- **TOML configuration**: User-friendly configuration with automatic setup
- **Cross-platform**: Proper config directories on Linux, macOS, and Windows
- **Configurable data directory**: Customize where bookmarks are stored
- **Automatic directory creation**: Sets up required directories with proper permissions

#### Output Formats
- **Human-readable**: Clean, formatted output for terminal use
- **JSON output**: Structured data perfect for scripting and integration

## Usage

### Basic Commands

```bash
# Add a bookmark with automatic title extraction
automark add "https://example.com"

# Add a bookmark with manual title
automark add "https://example.com" "My Custom Title"

# Add without fetching metadata (faster)
automark add "https://example.com" "Title" --no-fetch

# List all bookmarks
automark list

# Delete a bookmark (using full or partial ID)
automark delete abc12345
```

### Advanced Search

```bash
# Search by text (searches titles, URLs, authors, notes)
automark search "rust programming"

# Filter by tags (multiple tags use AND logic)
automark search --tags rust,web,tutorial

# Filter by reading status
automark search --status reading

# Filter by priority range
automark search --priority 4-5

# Filter by date range (MM-DD-YYYY format)
automark search --since 01-01-2024 --until 12-31-2024

# Combine multiple filters with sorting
automark search "web development" --tags javascript --status unread --sort-by priority --sort-order descending

# Get results in JSON format
automark search rust --tags programming -o json
```

### Output Formats

```bash
# Human-readable output (default)
automark list

# JSON output for scripting
automark list -o json
automark search rust -o json
automark add "https://example.com" "Title" -o json
```

## Configuration

Automark automatically creates a configuration file on first run. The config file location varies by platform:

- **Linux/Unix**: `~/.config/automark/config.toml`
- **macOS**: `~/Library/Application Support/automark/config.toml`  
- **Windows**: `%APPDATA%\automark\config.toml`

### Default Configuration

```toml
# Automark Configuration File
# This file contains configuration settings for the automark bookmark manager.

[storage]
# Directory where bookmark data is stored
# Use ~ for home directory, which will be expanded automatically
data_dir = "~/.local/share/automark"
```

### Customizing Storage Location

Edit the config file to change where bookmarks are stored:

```toml
[storage]
data_dir = "/custom/path/to/bookmarks"
# or
data_dir = "~/Documents/my-bookmarks"
```

The application will automatically create the directory if it doesn't exist and validate permissions.

## Installation

### From Source

```bash
git clone https://github.com/evcraddock/automark.git
cd automark
cargo install --path .
```

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git (for cloning the repository)

## Roadmap

This is an MVP implementation. For the full feature roadmap and specifications, see:

- [MVP Specification](specs/mvp-spec.md) - Current implementation scope
- [Full Application Specification](specs/bookmarking-application-spec.md) - Complete vision
- [Technical Architecture](specs/technical-architecture.md) - System design

Future features will include metadata extraction, tagging, search, TUI interface, and cross-device synchronization.

## Development

Built with:
- **Rust** - Systems programming language
- **Automerge** - Conflict-free replicated data types
- **Clap** - Command line argument parsing
- **Tokio** - Async runtime

Run tests:
```bash
cargo test
```

Build:
```bash
cargo build --release
```