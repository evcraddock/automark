# Task 1: Initialize Rust Project

## Objective
Initialize a new Rust project for the automark CLI application with all required dependencies for the MVP.

## Requirements

1. **Create Cargo.toml** with project name "automark", version "0.1.0", and description

2. **Add dependencies**: automerge, clap (with derive features), uuid (with v4 and serde features), chrono (with serde features), serde (with derive features), url, tokio (with full features), anyhow

3. **Create module structure**: 
   - `src/main.rs` with a simple hello message
   - Empty modules: `types/`, `traits/`, `adapters/`, `commands/`
   - Each module should have a `mod.rs` file

4. **Verify setup**: Ensure `cargo check` and `cargo run` work

## Success Criteria
- Project compiles without errors
- Basic "Hello, automark!" output when run
- All module directories created with proper structure