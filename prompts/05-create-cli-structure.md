# Task 5: Create CLI Command Structure

## Objective
Set up the CLI argument parsing and command structure using clap for the MVP commands.

## Requirements

1. **Create CLI structures** in `src/commands/mod.rs`:
   - Main Cli struct with clap Parser derive
   - Commands enum with Add, List, Delete variants
   - AddArgs struct (url and title fields)
   - DeleteArgs struct (id field)
   - Add helpful descriptions and help text

2. **Create CommandHandler trait**:
   - Async trait with execute method
   - Takes mutable repository reference
   - Returns BookmarkResult<()>

3. **Create placeholder command modules**:
   - Empty files: add.rs, list.rs, delete.rs
   - Export modules from commands/mod.rs

4. **Update main.rs** with basic CLI integration:
   - Parse CLI arguments
   - Match on commands and print placeholder messages
   - Use tokio main for async support
   - Include all module declarations

5. **Write tests** for:
   - CLI argument parsing
   - Command creation
   - Help and version output
   - Error handling for missing arguments

## Success Criteria
- All three commands parse correctly from command line
- Help and version flags work
- Placeholder messages print for each command
- Tests pass for CLI parsing
- Foundation is ready for command implementations