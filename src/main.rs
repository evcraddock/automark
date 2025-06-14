mod types;
mod traits;
mod adapters;
mod commands;
mod tui;

use std::process;
use clap::Parser;
use commands::{Cli, Commands, OutputFormat, handle_add_command, handle_list_command, handle_delete_command, handle_search_command, handle_sync_command, handle_tui_command, auto_sync, output};
use adapters::{AutomergeBookmarkRepository, FileStorageManager};
use types::{BookmarkError, ConfigError};

fn handle_config_error(error: ConfigError, format: OutputFormat) -> ! {
    match format {
        OutputFormat::Json => {
            eprintln!("{{\"success\": false, \"error\": {{\"code\": \"CONFIG_ERROR\", \"message\": \"{}\"}}}}", error);
        }
        OutputFormat::Human => {
            eprintln!("Configuration error: {}", error);
        }
    }
    process::exit(1);
}

fn handle_bookmark_error(error: BookmarkError, format: OutputFormat) -> ! {
    output::print_error(format, &error);
    let exit_code = match error {
        BookmarkError::InvalidUrl(_) => 2,
        BookmarkError::NotFound(_) => 3,
        BookmarkError::EmptyTitle => 2,
        BookmarkError::InvalidId(_) => 3,
        BookmarkError::MetadataExtraction(_) => 4,
        BookmarkError::SyncError(_) => 5,
        BookmarkError::TerminalError(_) => 6,
    };
    process::exit(exit_code);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let format = OutputFormat::from(cli.output);
    
    // Load configuration
    let config = match FileStorageManager::load_config() {
        Ok(config) => config,
        Err(e) => handle_config_error(e, format),
    };
    
    // Ensure data directory exists
    let _data_dir = match FileStorageManager::ensure_data_directory(&config) {
        Ok(dir) => dir,
        Err(e) => handle_config_error(e, format),
    };
    
    // Get bookmark file path
    let data_file_path = match FileStorageManager::get_bookmark_file_path(&config) {
        Ok(path) => path,
        Err(e) => handle_config_error(e, format),
    };
    
    // Initialize repository
    let mut repository = match AutomergeBookmarkRepository::new(data_file_path) {
        Ok(repo) => repo,
        Err(e) => {
            match format {
                OutputFormat::Json => {
                    output::print_error(format, &e);
                }
                OutputFormat::Human => {
                    eprintln!("Failed to initialize bookmark repository: {}", e);
                }
            }
            process::exit(1);
        }
    };
    
    // Execute commands
    let result = match &cli.command {
        Commands::Add(args) => {
            let result = handle_add_command(args.clone(), &mut repository, &config, format).await;
            if result.is_ok() {
                auto_sync::auto_sync_if_enabled(&mut repository, &config, format).await?;
            }
            result
        }
        Commands::List => {
            handle_list_command(&mut repository, format).await
        }
        Commands::Delete(args) => {
            let result = handle_delete_command(args.clone(), &mut repository, format).await;
            if result.is_ok() {
                auto_sync::auto_sync_if_enabled(&mut repository, &config, format).await?;
            }
            result
        }
        Commands::Search(args) => {
            handle_search_command(args.clone(), &mut repository, format).await
        }
        Commands::Sync(args) => {
            handle_sync_command(args, &mut repository, &config, format).await
        }
        Commands::Tui(args) => {
            handle_tui_command(args.clone(), &mut repository, format).await
        }
    };
    
    if let Err(error) = result {
        handle_bookmark_error(error, format);
    }
    
    Ok(())
}