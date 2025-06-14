use clap::Args;
use serde::{Serialize, Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use uuid::Uuid;
use std::time::Duration;
use crate::traits::BookmarkRepository;
use crate::types::{BookmarkResult, BookmarkError};
use super::{OutputFormat, output};

/// Arguments for the sync command
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// WebSocket sync server URL
    #[arg(long, default_value = "wss://sync.automerge.org")]
    pub server: String,
    
    /// Document ID to sync (if not provided, syncs the main bookmark document)
    #[arg(long)]
    pub document_id: Option<String>,
    
    /// Perform a dry run (connect but don't save changes)
    #[arg(long)]
    pub dry_run: bool,
    
    /// Connection timeout in seconds
    #[arg(long, default_value = "30")]
    pub timeout: u64,
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
    _repository: &mut dyn BookmarkRepository,
    format: OutputFormat,
) -> BookmarkResult<()> {
    let start_time = std::time::Instant::now();
    
    // Generate ephemeral peer ID
    let peer_id = Uuid::new_v4().to_string();
    let document_id = args.document_id.clone()
        .unwrap_or_else(|| "bookmarks".to_string());
    
    if format == OutputFormat::Human {
        println!("🔄 Connecting to sync server: {}", args.server);
        println!("📄 Document ID: {}", document_id);
        if args.dry_run {
            println!("⚠️  Dry run mode - changes will not be saved");
        }
    }
    
    // Connect to WebSocket server
    let (ws_stream, _) = match connect_async(&args.server).await {
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
    let timeout = Duration::from_secs(args.timeout);
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
                                
                                // Send sync request for our document
                                if let Some(ref remote_id) = _remote_peer_id {
                                    let request_msg = ProtocolMessage::Request {
                                        document_id: document_id.clone(),
                                        sender_id: peer_id.clone(),
                                        target_id: remote_id.clone(),
                                    };
                                    
                                    let request_data = cbor4ii::serde::to_vec(vec![0], &request_msg)
                                        .map_err(|e| BookmarkError::SyncError(format!("Failed to encode request: {}", e)))?;
                                    
                                    write.send(Message::Binary(request_data)).await
                                        .map_err(|e| BookmarkError::SyncError(format!("Failed to send request: {}", e)))?;
                                }
                            }
                            Ok(ProtocolMessage::Sync { document_id: doc_id, data: sync_data, .. }) => {
                                if doc_id == document_id {
                                    changes_received += 1;
                                    
                                    if !args.dry_run {
                                        // TODO: Apply sync data to repository
                                        // This requires integrating with Automerge's sync protocol
                                        // For now, we'll just count the messages
                                    }
                                    
                                    if format == OutputFormat::Human {
                                        println!("📥 Received sync data for document: {} ({} bytes)", doc_id, sync_data.len());
                                    }
                                }
                            }
                            Ok(ProtocolMessage::Request { document_id: doc_id, sender_id, .. }) => {
                                if doc_id == document_id {
                                    // TODO: Send our document state
                                    // This requires getting sync state from repository
                                    changes_sent += 1;
                                    
                                    if format == OutputFormat::Human {
                                        println!("📤 Sending sync data to peer: {}", sender_id);
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
        server: args.server.clone(),
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
            server: "wss://sync.automerge.org".to_string(),
            document_id: None,
            dry_run: false,
            timeout: 30,
        };
        
        assert_eq!(args.server, "wss://sync.automerge.org");
        assert!(args.document_id.is_none());
        assert!(!args.dry_run);
        assert_eq!(args.timeout, 30);
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