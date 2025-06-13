mod types;
mod traits;
mod adapters;
mod commands;

use std::path::PathBuf;
use std::process;
use clap::Parser;
use commands::{Cli, Commands, handle_add_command, handle_list_command, handle_delete_command};
use adapters::AutomergeBookmarkRepository;
use types::BookmarkError;

fn get_data_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = dirs::data_local_dir()
        .ok_or("Could not determine local data directory")?
        .join("automark");
    
    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;
    
    Ok(data_dir.join("bookmarks.automerge"))
}

fn handle_bookmark_error(error: BookmarkError) -> ! {
    eprintln!("Error: {}", error);
    let exit_code = match error {
        BookmarkError::InvalidUrl(_) => 2,
        BookmarkError::NotFound(_) => 3,
        BookmarkError::EmptyTitle => 2,
        BookmarkError::InvalidId(_) => 3,
    };
    process::exit(exit_code);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize repository
    let data_file_path = get_data_file_path()?;
    let mut repository = match AutomergeBookmarkRepository::new(data_file_path) {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to initialize bookmark repository: {}", e);
            process::exit(1);
        }
    };
    
    // Execute commands
    let result = match cli.command {
        Commands::Add(args) => {
            handle_add_command(args, &mut repository).await
        }
        Commands::List => {
            handle_list_command(&mut repository).await
        }
        Commands::Delete(args) => {
            handle_delete_command(args, &mut repository).await
        }
    };
    
    if let Err(error) = result {
        handle_bookmark_error(error);
    }
    
    Ok(())
}