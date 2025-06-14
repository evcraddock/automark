use clap::Args;
use serde::{Serialize, Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use uuid::Uuid;
use std::time::Duration;
use crate::traits::BookmarkRepository;
use crate::types::{BookmarkResult, BookmarkError, Config};
use super::{OutputFormat, output};

/// Arguments for the sync command
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// WebSocket sync server URL (overrides config)
    #[arg(long)]
    pub server: Option<String>,
    
    /// Document ID to sync (if not provided, syncs the main bookmark document)
    #[arg(long)]
    pub document_id: Option<String>,
    
    /// Perform a dry run (connect but don't save changes)
    #[arg(long)]
    pub dry_run: bool,
    
    /// Connection timeout in seconds (overrides config)
    #[arg(long)]
    pub timeout: Option<u64>,
}

/// Sync command response
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncResponse {
    /// Server URL we connected to
    pub server: String,
    /// Document ID that was synced
    pub document_id: String,
    /// Number of changes received
    pub changes_received: usize,
    /// Number of changes sent
    pub changes_sent: usize,
    /// Whether the sync was successful
    pub success: bool,
    /// Sync duration in milliseconds
    pub duration_ms: u64,
}

/// Protocol messages for Automerge sync
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ProtocolMessage {
    #[serde(rename = "join")]
    Join {
        #[serde(rename = "senderId")]
        sender_id: String,
        #[serde(rename = "supportedProtocolVersions")]
        supported_protocol_versions: Vec<String>,
        #[serde(rename = "storageId")]
        storage_id: Option<String>,
    },
    #[serde(rename = "peer")]
    Peer {
        #[serde(rename = "senderId")]
        sender_id: String,
        #[serde(rename = "supportedProtocolVersions")]
        supported_protocol_versions: Vec<String>,
        #[serde(rename = "storageId")]
        storage_id: Option<String>,
        #[serde(rename = "selectedProtocolVersion")]
        selected_protocol_version: String,
    },
    #[serde(rename = "sync")]
    Sync {
        #[serde(rename = "documentId")]
        document_id: String,
        #[serde(rename = "senderId")]
        sender_id: String,
        #[serde(rename = "targetId")]
        target_id: String,
        data: Vec<u8>,
    },
    #[serde(rename = "request")]
    Request {
        #[serde(rename = "documentId")]
        document_id: String,
        #[serde(rename = "senderId")]
        sender_id: String,
        #[serde(rename = "targetId")]
        target_id: String,
    },
}

pub async fn handle_sync_command(
    args: &SyncArgs,
    repository: &mut dyn BookmarkRepository,
    config: &Config,
    format: OutputFormat,
) -> BookmarkResult<()> {
    // Check if sync is enabled
    if !config.sync.enabled {
        let error = BookmarkError::SyncError("Sync is disabled in configuration".to_string());
        output::print_error(format, &error);
        return Err(error);
    }
    
    let start_time = std::time::Instant::now();
    
    // Use config values with command-line overrides
    let server_url = args.server.as_ref().unwrap_or(&config.sync.server_url);
    let timeout_secs = args.timeout.unwrap_or(config.sync.timeout_secs);
    
    // Generate ephemeral peer ID
    let peer_id = Uuid::new_v4().to_string();
    let document_id = args.document_id.clone()
        .unwrap_or_else(|| "bookmarks".to_string());
    
    if format == OutputFormat::Human {
        println!("🔄 Connecting to sync server: {}", server_url);
        println!("📄 Document ID: {}", document_id);
        if args.dry_run {
            println!("⚠️  Dry run mode - changes will not be saved");
        }
    }
    
    // Connect to WebSocket server
    let (ws_stream, _) = match connect_async(server_url).await {
        Ok(result) => result,
        Err(e) => {
            let error = BookmarkError::SyncError(format!("Failed to connect to sync server: {}", e));
            output::print_error(format, &error);
            return Err(error);
        }
    };
    
    let (mut write, mut read) = ws_stream.split();
    
    // Send join message
    let join_msg = ProtocolMessage::Join {
        sender_id: peer_id.clone(),
        supported_protocol_versions: vec!["1".to_string()],
        storage_id: None,
    };
    
    let join_data = cbor4ii::serde::to_vec(vec![0], &join_msg)
        .map_err(|e| BookmarkError::SyncError(format!("Failed to encode join message: {}", e)))?;
    
    write.send(Message::Binary(join_data)).await
        .map_err(|e| BookmarkError::SyncError(format!("Failed to send join message: {}", e)))?;
    
    // Handle messages
    let mut changes_received = 0;
    let mut changes_sent = 0;
    let mut _remote_peer_id = None;
    
    // Set up timeout
    let timeout = Duration::from_secs(timeout_secs);
    let timeout_future = tokio::time::sleep(timeout);
    tokio::pin!(timeout_future);
    
    loop {
        tokio::select! {
            _ = &mut timeout_future => {
                if format == OutputFormat::Human {
                    println!("⏱️  Sync timeout reached");
                }
                break;
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        // Parse CBOR message
                        match cbor4ii::serde::from_slice::<ProtocolMessage>(&data[1..]) {
                            Ok(ProtocolMessage::Peer { sender_id, selected_protocol_version, .. }) => {
                                _remote_peer_id = Some(sender_id.clone());
                                if format == OutputFormat::Human {
                                    println!("🤝 Connected to peer: {} (protocol v{})", sender_id, selected_protocol_version);
                                }
                                
                                // Send initial sync message
                                let sync_msg = repository.generate_sync_message(&sender_id).await?;
                                
                                if !sync_msg.is_empty() {
                                    let sync_message = ProtocolMessage::Sync {
                                        document_id: document_id.clone(),
                                        sender_id: peer_id.clone(),
                                        target_id: sender_id.clone(),
                                        data: sync_msg,
                                    };
                                    
                                    let sync_data = cbor4ii::serde::to_vec(vec![0], &sync_message)
                                        .map_err(|e| BookmarkError::SyncError(format!("Failed to encode sync message: {}", e)))?;
                                    
                                    write.send(Message::Binary(sync_data)).await
                                        .map_err(|e| BookmarkError::SyncError(format!("Failed to send sync message: {}", e)))?;
                                    
                                    changes_sent += 1;
                                    
                                    if format == OutputFormat::Human {
                                        println!("📤 Sent initial sync data");
                                    }
                                }
                            }
                            Ok(ProtocolMessage::Sync { document_id: doc_id, data: sync_data, .. }) => {
                                if doc_id == document_id {
                                    changes_received += 1;
                                    
                                    if !args.dry_run {
                                        // Apply sync message to repository
                                        let changed = repository.apply_sync_message(&peer_id, sync_data.clone()).await?;
                                        if changed && format == OutputFormat::Human {
                                            println!("📝 Applied changes from sync message");
                                        }
                                    }
                                    
                                    if format == OutputFormat::Human {
                                        println!("📥 Received sync data for document: {} ({} bytes)", doc_id, sync_data.len());
                                    }
                                }
                            }
                            Ok(ProtocolMessage::Request { document_id: doc_id, sender_id, .. }) => {
                                if doc_id == document_id {
                                    // Generate and send our sync message
                                    let sync_msg = repository.generate_sync_message(&sender_id).await?;
                                    
                                    if !sync_msg.is_empty() {
                                        let sync_message = ProtocolMessage::Sync {
                                            document_id: doc_id.clone(),
                                            sender_id: peer_id.clone(),
                                            target_id: sender_id.clone(),
                                            data: sync_msg,
                                        };
                                        
                                        let sync_data = cbor4ii::serde::to_vec(vec![0], &sync_message)
                                            .map_err(|e| BookmarkError::SyncError(format!("Failed to encode sync message: {}", e)))?;
                                        
                                        write.send(Message::Binary(sync_data)).await
                                            .map_err(|e| BookmarkError::SyncError(format!("Failed to send sync message: {}", e)))?;
                                        
                                        changes_sent += 1;
                                        
                                        if format == OutputFormat::Human {
                                            println!("📤 Sent sync data to peer: {}", sender_id);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        if format == OutputFormat::Human {
                            println!("🔌 Connection closed by server");
                        }
                        break;
                    }
                    Some(Err(e)) => {
                        let error = BookmarkError::SyncError(format!("WebSocket error: {}", e));
                        output::print_error(format, &error);
                        return Err(error);
                    }
                    None => break,
                    _ => {}
                }
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    let response = SyncResponse {
        server: server_url.to_string(),
        document_id,
        changes_received,
        changes_sent,
        success: true,
        duration_ms: duration.as_millis() as u64,
    };
    
    match format {
        OutputFormat::Human => {
            println!("\n✅ Sync completed successfully!");
            println!("📊 Summary:");
            println!("   Changes received: {}", changes_received);
            println!("   Changes sent: {}", changes_sent);
            println!("   Duration: {:.2}s", duration.as_secs_f64());
        }
        OutputFormat::Json => {
            output::print_response(format, &response)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_args_default() {
        let args = SyncArgs {
            server: None,
            document_id: None,
            dry_run: false,
            timeout: None,
        };
        
        assert!(args.server.is_none());
        assert!(args.document_id.is_none());
        assert!(!args.dry_run);
        assert!(args.timeout.is_none());
    }
    
    #[test]
    fn test_protocol_message_serialization() {
        let join_msg = ProtocolMessage::Join {
            sender_id: "test-id".to_string(),
            supported_protocol_versions: vec!["1".to_string()],
            storage_id: None,
        };
        
        let serialized = cbor4ii::serde::to_vec(vec![0], &join_msg);
        assert!(serialized.is_ok());
        
        let data = serialized.unwrap();
        assert!(!data.is_empty());
    }
}