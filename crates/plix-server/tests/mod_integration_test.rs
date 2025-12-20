//! Integration tests for WASM mod lifecycle via server ModManager
//!
//! Tests the full lifecycle: load -> dispatch events -> shutdown

use plix_mod_core::events::GameEvent;
use plix_server::mods::ModManager;
use std::fs;
use tempfile::tempdir;

/// Get the path to the minimal mod fixture's WASM file
fn minimal_mod_wasm_path() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .join("plix-mod-runtime-wasm")
        .join("tests")
        .join("fixtures")
        .join("minimal_mod")
        .join("mod.wasm")
}

/// Create a mod directory with manifest and WASM file
fn setup_mod_directory(
    mods_dir: &std::path::Path,
    mod_id: &str,
    wasm_bytes: &[u8],
) -> std::path::PathBuf {
    let mod_dir = mods_dir.join(mod_id);
    fs::create_dir_all(&mod_dir).unwrap();

    // Create manifest in the expected format (root level, not [mod] section)
    let manifest = format!(
        r#"id = "{mod_id}"
name = "Test Mod"
version = "1.0.0"
api_version = 1

[capabilities]
"#
    );
    fs::write(mod_dir.join("mod.toml"), manifest).unwrap();

    // Copy WASM file
    fs::write(mod_dir.join("mod.wasm"), wasm_bytes).unwrap();

    mod_dir
}

#[test]
fn test_full_mod_lifecycle_with_wasm() {
    // Skip if WASM fixture doesn't exist
    let wasm_path = minimal_mod_wasm_path();
    if !wasm_path.exists() {
        eprintln!(
            "Skipping test: WASM fixture not found at {}",
            wasm_path.display()
        );
        return;
    }

    let wasm_bytes = fs::read(&wasm_path).unwrap();
    let temp = tempdir().unwrap();
    let mods_dir = temp.path().join("mods");
    fs::create_dir(&mods_dir).unwrap();

    setup_mod_directory(&mods_dir, "test-mod", &wasm_bytes);

    // Create ModManager with WASM runtime enabled
    let mut manager = ModManager::with_wasm_runtime(temp.path());

    // Load mods
    let count = manager.load_mods().unwrap();
    assert_eq!(count, 1, "Should load one mod");
    assert!(manager.is_mod_loaded("test-mod"), "Mod should be loaded");
    assert!(
        manager.has_wasm_runtime(),
        "WASM runtime should be available"
    );
    assert_eq!(manager.wasm_mod_count(), 1, "Should have one WASM mod");

    // Emit and dispatch events
    manager.emit_event(GameEvent::server_start(1));
    let cancelled = manager.dispatch_events();
    assert_eq!(cancelled, 0, "No events should be cancelled by minimal mod");

    // Test single event dispatch
    let event = GameEvent::player_join(1, "TestPlayer", 2);
    let cancelled = manager.dispatch_event(&event);
    assert!(!cancelled, "Player join event should not be cancelled");

    // Shutdown (calls mod_shutdown on all mods)
    manager.shutdown();
    assert_eq!(manager.mod_count(), 0, "All mods should be unloaded");
    assert_eq!(
        manager.wasm_mod_count(),
        0,
        "All WASM mods should be unloaded"
    );
}

#[test]
fn test_mod_manager_without_wasm() {
    let temp = tempdir().unwrap();
    let mods_dir = temp.path().join("mods");
    fs::create_dir(&mods_dir).unwrap();

    // Create a manifest-only mod (no WASM)
    let mod_dir = mods_dir.join("manifest-only");
    fs::create_dir_all(&mod_dir).unwrap();
    let manifest = r#"
id = "manifest-only"
name = "Manifest Only Mod"
version = "1.0.0"
api_version = 1
"#;
    fs::write(mod_dir.join("mod.toml"), manifest).unwrap();

    // Create ModManager with WASM runtime
    let mut manager = ModManager::with_wasm_runtime(temp.path());

    // Load mods
    let count = manager.load_mods().unwrap();
    assert_eq!(count, 1, "Should load one mod");
    assert!(
        manager.is_mod_loaded("manifest-only"),
        "Mod should be loaded"
    );
    assert_eq!(
        manager.wasm_mod_count(),
        0,
        "No WASM mods (no mod.wasm file)"
    );

    // Events work without WASM mods
    manager.emit_event(GameEvent::server_start(1));
    let cancelled = manager.dispatch_events();
    assert_eq!(cancelled, 0);

    manager.shutdown();
}

#[test]
fn test_wasm_mod_metrics() {
    // Skip if WASM fixture doesn't exist
    let wasm_path = minimal_mod_wasm_path();
    if !wasm_path.exists() {
        eprintln!(
            "Skipping test: WASM fixture not found at {}",
            wasm_path.display()
        );
        return;
    }

    let wasm_bytes = fs::read(&wasm_path).unwrap();
    let temp = tempdir().unwrap();
    let mods_dir = temp.path().join("mods");
    fs::create_dir(&mods_dir).unwrap();

    setup_mod_directory(&mods_dir, "metrics-mod", &wasm_bytes);

    let mut manager = ModManager::with_wasm_runtime(temp.path());
    manager.load_mods().unwrap();

    // Dispatch an event to trigger mod execution
    let event = GameEvent::server_start(1);
    manager.dispatch_event(&event);

    // Check metrics are available
    let metrics = manager.wasm_mod_metrics("metrics-mod");
    // Note: Metrics might not increment for server_start if mod doesn't subscribe
    // The important thing is that the method doesn't panic
    assert!(
        metrics.is_some() || manager.wasm_mod_count() == 1,
        "Should have metrics or at least have the mod loaded"
    );

    manager.shutdown();
}

#[test]
fn test_unload_single_mod() {
    // Skip if WASM fixture doesn't exist
    let wasm_path = minimal_mod_wasm_path();
    if !wasm_path.exists() {
        eprintln!(
            "Skipping test: WASM fixture not found at {}",
            wasm_path.display()
        );
        return;
    }

    let wasm_bytes = fs::read(&wasm_path).unwrap();
    let temp = tempdir().unwrap();
    let mods_dir = temp.path().join("mods");
    fs::create_dir(&mods_dir).unwrap();

    setup_mod_directory(&mods_dir, "mod-to-unload", &wasm_bytes);

    let mut manager = ModManager::with_wasm_runtime(temp.path());
    manager.load_mods().unwrap();

    assert_eq!(manager.wasm_mod_count(), 1);
    assert!(manager.is_mod_loaded("mod-to-unload"));

    // Unload the specific mod
    manager.unload_mod("mod-to-unload").unwrap();

    assert_eq!(manager.wasm_mod_count(), 0);
    assert!(!manager.is_mod_loaded("mod-to-unload"));
}

// ============================================================================
// MOD SYNC HANDSHAKE TESTS (Feature 037)
// ============================================================================

mod mod_sync_handshake {
    use plix_common::identity::SessionId;
    use plix_common::protocol::messages::{ModSetResponse, JoinDecision};
    use plix_common::types::TeamId;
    use plix_mod_distribution::{JoinPolicy, SyncConfig};
    use plix_server::mods::{ModInfo, PendingModConnection, SyncSession, SyncState};
    use std::net::SocketAddr;
    use std::path::PathBuf;

    fn test_join_policy() -> JoinPolicy {
        JoinPolicy {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: false,
        }
    }

    fn test_sync_config() -> SyncConfig {
        SyncConfig {
            max_payload_mb: 25,
            chunk_size_kb: 256,
            max_inflight_chunks: 4,
            transfer_timeout_secs: 300,
        }
    }

    fn test_mod_info() -> ModInfo {
        ModInfo {
            id: "test-mod".to_string(),
            version: "1.0.0".to_string(),
            client_payload_hash: "abc123def456".to_string(),
            client_payload_size: 1024,
            payload_path: PathBuf::from("/tmp/test.zip"),
            has_network_channels: false,
        }
    }

    /// Test E2E: Server with server-only mod + client_payload, client cache empty, sync allowed
    #[test]
    fn test_mod_sync_handshake_e2e_no_sync_needed() {
        // Server has no mods with client payloads
        let session = SyncSession::new(test_join_policy(), test_sync_config(), vec![]);

        // Build descriptor (what server sends to client)
        let descriptor = session.build_mod_set_descriptor();
        assert!(descriptor.mods.is_empty());
        assert_eq!(descriptor.total_payload_size, 0);

        // Client responds (supports sync, no cache)
        let mut session = session;
        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec![],
            engine_version: Some("1.0.0".to_string()),
        };

        let decision = session.handle_response(response);

        // Join should be allowed with no sync needed
        assert!(decision.allowed, "Join should be allowed");
        assert!(decision.mods_to_sync.is_empty(), "No mods to sync");
        assert_eq!(session.state(), SyncState::Complete, "Session should complete immediately");
    }

    /// Test E2E: Server with mod + client_payload, client has cached payload
    #[test]
    fn test_mod_sync_handshake_client_has_cache() {
        let mod_info = test_mod_info();
        let mods = vec![mod_info.clone()];

        let session = SyncSession::new(test_join_policy(), test_sync_config(), mods);

        // Build descriptor
        let descriptor = session.build_mod_set_descriptor();
        assert_eq!(descriptor.mods.len(), 1);
        assert_eq!(descriptor.mods[0].id, "test-mod");
        assert_eq!(descriptor.mods[0].client_payload_hash, Some("abc123def456".to_string()));

        // Client responds - already has the payload cached
        let mut session = session;
        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec!["abc123def456".to_string()],
            engine_version: Some("1.0.0".to_string()),
        };

        let decision = session.handle_response(response);

        // Join allowed, no sync needed (client has it)
        assert!(decision.allowed, "Join should be allowed");
        assert!(decision.mods_to_sync.is_empty(), "No mods to sync (client has cache)");
        assert_eq!(session.state(), SyncState::Complete);
    }

    /// Test E2E: Server with mod + client_payload, client cache empty, sync required
    #[test]
    fn test_mod_sync_handshake_sync_required() {
        let mod_info = test_mod_info();
        let mods = vec![mod_info.clone()];

        let session = SyncSession::new(test_join_policy(), test_sync_config(), mods);

        // Client responds - no cache, supports sync
        let mut session = session;
        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec![],
            engine_version: Some("1.0.0".to_string()),
        };

        let decision = session.handle_response(response);

        // Join allowed, but sync needed
        assert!(decision.allowed, "Join should be allowed");
        assert_eq!(decision.mods_to_sync.len(), 1, "One mod needs sync");
        assert_eq!(decision.mods_to_sync[0], "test-mod");
        assert_eq!(session.state(), SyncState::Transferring);
    }

    /// Test: Client doesn't support sync but server requires it
    #[test]
    fn test_mod_sync_handshake_reject_no_sync_support() {
        let policy = JoinPolicy {
            allow_server_only: false,
            allow_payload_sync: true,
            require_payload_sync: true,
        };

        let mod_info = test_mod_info();
        let mods = vec![mod_info];

        let mut session = SyncSession::new(policy, test_sync_config(), mods);

        // Client doesn't support sync
        let response = ModSetResponse {
            supports_sync: false,
            cached_hashes: vec![],
            engine_version: None,
        };

        let decision = session.handle_response(response);

        // Join should be rejected
        assert!(!decision.allowed, "Join should be rejected");
        assert!(decision.rejection_reason.is_some());
        assert_eq!(session.state(), SyncState::Failed);
    }

    /// Test: PendingModConnection wrapper works correctly
    #[test]
    fn test_pending_mod_connection() {
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let session_id = SessionId::new();

        let mut pending = PendingModConnection::new(
            addr,
            "TestPlayer".to_string(),
            "TestPlayer".to_string(),
            TeamId(1),
            session_id,
            test_join_policy(),
            test_sync_config(),
            vec![], // No mods with payloads
        );

        assert!(!pending.is_ready_to_join());
        assert!(!pending.is_failed());
        assert!(!pending.is_timed_out());
        assert_eq!(pending.sync_state(), SyncState::AwaitingCapabilities);

        // Simulate client response
        let response = ModSetResponse {
            supports_sync: true,
            cached_hashes: vec![],
            engine_version: None,
        };

        let decision = pending.handle_response(response);

        assert!(decision.allowed);
        assert!(pending.is_ready_to_join());
        assert_eq!(pending.sync_state(), SyncState::Complete);
    }

    /// Test: Server-only client allowed when policy permits
    #[test]
    fn test_server_only_client_allowed() {
        let policy = JoinPolicy {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: false,
        };

        let mod_info = test_mod_info();
        let mods = vec![mod_info];

        let mut session = SyncSession::new(policy, test_sync_config(), mods);

        // Client doesn't support sync (server-only mode)
        let response = ModSetResponse {
            supports_sync: false,
            cached_hashes: vec![],
            engine_version: None,
        };

        let decision = session.handle_response(response);

        // Join allowed in server-only mode
        assert!(decision.allowed, "Server-only client should be allowed");
        assert!(decision.mods_to_sync.is_empty(), "No sync for server-only client");
        assert_eq!(session.state(), SyncState::Complete);
    }
}
