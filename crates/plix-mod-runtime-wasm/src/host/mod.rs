//! Host function implementations
//!
//! This module contains all the plix_* host functions that WASM mods can call.
//! Host functions are registered via Wasmtime's Linker and provide the bridge
//! between the sandboxed WASM code and the game engine.

pub mod caps;
pub mod entities;
pub mod events;
pub mod log;
pub mod net;
pub mod timers;
pub mod world;

use crate::abi::{ABI_VERSION, API_VERSION};
use crate::instance::ModState;
use crate::memory;
use wasmtime::{Caller, Linker};

/// Create a new Linker with all plix host functions registered
pub fn create_linker(engine: &wasmtime::Engine) -> Result<Linker<ModState>, wasmtime::Error> {
    let mut linker = Linker::new(engine);

    // Register all host functions in the "plix" namespace

    // === Logging ===
    linker.func_wrap("plix", "log", plix_log)?;

    // === Version queries ===
    linker.func_wrap("plix", "get_api_version", plix_get_api_version)?;
    linker.func_wrap("plix", "get_abi_version", plix_get_abi_version)?;

    // === Capabilities ===
    linker.func_wrap("plix", "has_capability", plix_has_capability)?;

    // === Events ===
    linker.func_wrap("plix", "subscribe_event", plix_subscribe_event)?;
    linker.func_wrap("plix", "cancel_event", plix_cancel_event)?;

    // === Response buffer ===
    linker.func_wrap("plix", "response_ptr", plix_response_ptr)?;
    linker.func_wrap("plix", "response_len", plix_response_len)?;

    // === World API ===
    linker.func_wrap("plix", "world_call", plix_world_call)?;

    // === Entity API ===
    linker.func_wrap("plix", "entity_call", plix_entity_call)?;

    // === Net API ===
    linker.func_wrap("plix", "net_call", plix_net_call)?;

    // === Timer API ===
    linker.func_wrap("plix", "timer_call", plix_timer_call)?;

    Ok(linker)
}

// === Logging ===

/// Log a message from the mod
fn plix_log(mut caller: Caller<'_, ModState>, level: i32, ptr: i32, len: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();

    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => return 1, // EMOD001
    };

    // Clamp length to prevent huge log messages (max 4KB)
    let actual_len = (len as usize).min(log::MAX_LOG_MESSAGE_LEN);

    let message = match memory::read_string_lossy(&mut caller, &memory, ptr as usize, actual_len) {
        Ok(s) => s,
        Err(_) => return 4, // EMOD004 - out of bounds
    };

    let mod_id = caller.data().mod_id.clone();

    use crate::abi::LogLevel;
    match LogLevel::from_i32(level) {
        Some(LogLevel::Error) => tracing::error!(mod_id = %mod_id, "{}", message),
        Some(LogLevel::Warn) => tracing::warn!(mod_id = %mod_id, "{}", message),
        Some(LogLevel::Info) => tracing::info!(mod_id = %mod_id, "{}", message),
        Some(LogLevel::Debug) => tracing::debug!(mod_id = %mod_id, "{}", message),
        Some(LogLevel::Trace) => tracing::trace!(mod_id = %mod_id, "{}", message),
        None => tracing::info!(mod_id = %mod_id, "[level={}] {}", level, message),
    }

    0 // Success
}

// === Version queries ===

/// Get the engine API version
fn plix_get_api_version(mut caller: Caller<'_, ModState>) -> i32 {
    caller.data_mut().metrics.record_host_call();
    API_VERSION as i32
}

/// Get the ABI version
fn plix_get_abi_version(mut caller: Caller<'_, ModState>) -> i32 {
    caller.data_mut().metrics.record_host_call();
    ABI_VERSION as i32
}

// === Capabilities ===

/// Check if the mod has a specific capability
fn plix_has_capability(mut caller: Caller<'_, ModState>, cap_id: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();

    use crate::abi::CapabilityId;

    let cap = match CapabilityId::from_u32(cap_id as u32) {
        Some(id) => id.to_capability(),
        None => return 0, // Unknown capability
    };

    if caller.data().has_capability(cap) {
        1
    } else {
        0
    }
}

// === Events ===

/// Subscribe to an event type
fn plix_subscribe_event(mut caller: Caller<'_, ModState>, event_type: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();

    // Validate event type ID range
    if !(0..=31).contains(&event_type) {
        return 1; // EMOD001 - invalid argument
    }

    caller.data_mut().subscribe(event_type as u8);

    if caller.data().debug_mode {
        let mod_id = &caller.data().mod_id;
        tracing::debug!(mod_id = %mod_id, event_type, "Subscribed to event");
    }

    0 // Success
}

/// Cancel the current event (if permitted)
fn plix_cancel_event(mut caller: Caller<'_, ModState>) -> i32 {
    caller.data_mut().metrics.record_host_call();

    let state = caller.data_mut();

    match state.event_context.cancel_event(state.capabilities) {
        Ok(()) => 0, // Success
        Err(e) => {
            if e.code == plix_mod_core::errors::ErrorCode::PermissionDenied {
                state.metrics.record_permission_denied();
                2 // EMOD002
            } else {
                1 // EMOD001
            }
        }
    }
}

// === Response buffer ===

/// Get pointer to response buffer
/// Note: In a real implementation, this would return a pointer in WASM memory
/// For now, we return a fixed offset where we write responses
fn plix_response_ptr(mut caller: Caller<'_, ModState>) -> i32 {
    caller.data_mut().metrics.record_host_call();
    // Response buffer is at a fixed location in mod memory
    // The mod must allocate this space
    0x10000 // 64KB offset - mods should reserve this space
}

/// Get length of data in response buffer
fn plix_response_len(caller: Caller<'_, ModState>) -> i32 {
    caller.data().response_len() as i32
}

// === World API ===

/// World API call (placeholder - will be implemented in User Story 2)
fn plix_world_call(mut caller: Caller<'_, ModState>, _op: i32, _ptr: i32, _len: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();
    7 // EMOD007 - Unsupported (not yet implemented)
}

// === Entity API ===

/// Entity API call (placeholder - will be implemented in User Story 2)
fn plix_entity_call(mut caller: Caller<'_, ModState>, _op: i32, _ptr: i32, _len: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();
    7 // EMOD007 - Unsupported (not yet implemented)
}

// === Net API ===

/// Net API call (placeholder - will be implemented in User Story 2)
fn plix_net_call(mut caller: Caller<'_, ModState>, _op: i32, _ptr: i32, _len: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();
    7 // EMOD007 - Unsupported (not yet implemented)
}

// === Timer API ===

/// Timer API call (placeholder - will be implemented in User Story 2)
fn plix_timer_call(mut caller: Caller<'_, ModState>, _op: i32, _ptr: i32, _len: i32) -> i32 {
    caller.data_mut().metrics.record_host_call();
    7 // EMOD007 - Unsupported (not yet implemented)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_linker() {
        let engine = wasmtime::Engine::default();
        let linker = create_linker(&engine);
        assert!(linker.is_ok());
    }
}
