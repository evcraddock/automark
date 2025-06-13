# Automark

A local-first CLI bookmarking application built with Rust and Automerge.

## What it does

Automark is a simple command-line tool for managing bookmarks locally. It uses [Automerge](https://automerge.org/) for conflict-free replicated data storage, making it easy to sync bookmarks across devices in the future.

### Current MVP Features

- **Add bookmarks**: Save URLs with custom titles
- **List bookmarks**: View all saved bookmarks with IDs, URLs, titles, and dates  
- **Delete bookmarks**: Remove bookmarks by ID
- **Local storage**: Data persists in `~/.local/share/automark/bookmarks.automerge`
- **URL validation**: Basic validation prevents malformed URLs

## Usage

```bash
# Add a bookmark
automark add "https://example.com" "Example Website"

# List all bookmarks
automark list

# Delete a bookmark (using partial ID)
automark delete abc123
```

## Installation

```bash
cargo install --path .
```

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