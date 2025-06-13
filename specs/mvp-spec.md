# MVP Specification - Automark

## Overview
Minimal viable product for a bookmarking application that can add URLs and list them using Automerge for local storage.

## Core MVP Features

### 1. Basic Bookmark Management
- **Add URL**: Accept a URL and title (manual entry only)
- **List Bookmarks**: Display all saved bookmarks in simple list format
- **Delete Bookmark**: Remove a bookmark by ID

### 2. Data Model (Simplified)
**Bookmark Structure (MVP):**
- `id`: Unique identifier (UUID)
- `url`: Required URL string (with basic validation)
- `title`: Required title string
- `bookmarked_date`: Auto-generated timestamp

**No MVP Features:**
- No metadata extraction
- No tags
- No author field
- No notes
- No reading status
- No priority rating
- No search functionality
- No filtering
- No sorting

### 3. Application Interface
**CLI Only (No TUI for MVP):**
- `automark add <URL> <TITLE>` - Add bookmark
- `automark list` - List all bookmarks
- `automark delete <ID>` - Delete bookmark by ID

### 4. Storage
- **Automerge**: Single document storing all bookmarks
- **Local Storage**: `~/.local/share/automark/bookmarks.automerge`
- **No Configuration**: Use hardcoded default paths
- **No Server Sync**: Local-only for MVP

### 5. Technical Implementation

**Required Modules:**
- `types/bookmark.rs` - Basic Bookmark struct
- `traits/repository.rs` - BookmarkRepository trait (minimal methods)
- `adapters/automerge_repo.rs` - Automerge implementation
- `commands/add.rs` - Add command handler
- `commands/list.rs` - List command handler
- `commands/delete.rs` - Delete command handler
- `main.rs` - CLI bootstrap with clap

**Repository Interface (MVP):**
- `create(bookmark)` - Add new bookmark
- `find_all()` - Get all bookmarks
- `delete(id)` - Remove bookmark

**Dependencies (Minimal):**
- `automerge` - CRDT storage
- `clap` - CLI parsing
- `uuid` - ID generation
- `chrono` - Timestamps
- `serde` - Serialization
- `url` - Basic URL validation

## What's Excluded from MVP
- Metadata extraction (no HTTP requests)
- TUI interface
- Configuration management
- Search/filtering/sorting
- Tags and complex metadata
- Import/export functionality
- Error recovery mechanisms
- Comprehensive validation

## Success Criteria
MVP is complete when:
1. Can add a bookmark with URL and title
2. Can list all bookmarks showing ID, URL, title, and date
3. Can delete a bookmark by ID
4. Data persists between application runs using Automerge
5. Basic URL validation prevents malformed URLs
6. All code follows TDD approach with passing tests

This represents the absolute minimum functionality to demonstrate the core Automerge integration and basic bookmark operations.