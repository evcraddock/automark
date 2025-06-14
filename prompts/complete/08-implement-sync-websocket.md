# Task 8: Implement Sync Command with WebSocket Support

**GitHub Issue**: [#8](https://github.com/evcraddock/automark/issues/8)

## Objective
Create sync command for synchronizing Automerge documents with the community sync server at `wss://sync.automerge.org`.

## Requirements

1. **Add WebSocket dependencies** to Cargo.toml:
   - `tokio-tungstenite` for WebSocket client
   - `futures-util` for stream handling
   - `cbor4ii` for CBOR encoding/decoding

2. **Create sync command structure** in `src/commands/sync.rs`:
   - SyncArgs struct with optional server URL (default: wss://sync.automerge.org)
   - Document ID for syncing specific documents
   - Dry-run mode for testing connections
   - JSON output support

3. **Implement WebSocket client**:
   - Connect to sync server using tokio-tungstenite
   - Implement handshake protocol with peer ID exchange
   - Handle CBOR-encoded messages
   - Support protocol version "1"

4. **Implement sync protocol messages**:
   - Join message with peer ID and protocol version
   - Sync messages for document synchronization
   - Handle sync responses and merge changes
   - Support ephemeral messaging if needed

5. **Define sync flow**:
   - Generate ephemeral peer ID
   - Connect to WebSocket server
   - Send join/handshake message
   - Exchange sync messages for document
   - Merge received changes locally
   - Save updated document

6. **Add configuration support**:
   - Allow custom sync server URL in config
   - Option to disable sync functionality
   - Timeout and retry settings
   - Authentication support (future)

7. **Error handling**:
   - Handle connection failures gracefully
   - Validate server responses
   - Handle protocol version mismatches
   - Provide clear error messages

8. **Write comprehensive tests**:
   - Mock WebSocket server for testing
   - Test handshake protocol
   - Test sync message exchange
   - Test error scenarios
   - Test configuration options

## Implementation Notes

- The Automerge sync protocol uses CBOR encoding
- Messages follow the automerge-repo protocol format
- Peer IDs are ephemeral (per-session)
- Storage IDs can be used for persistent identification
- The community server at wss://sync.automerge.org is for development/prototyping

## Success Criteria
- Can connect to wss://sync.automerge.org
- Successfully exchanges sync messages
- Merges remote changes into local document
- Handles disconnections gracefully
- Provides clear feedback on sync status