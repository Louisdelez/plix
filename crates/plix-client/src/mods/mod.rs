//! Client-side mod handling for server mod synchronization
//!
//! This module provides:
//! - Handshake handling for mod set negotiation
//! - Payload cache for client data storage
//! - Payload receiver for chunked transfer reception
//! - Client data loader for mod data access

pub mod client_data_loader;
pub mod handshake;
pub mod payload_cache;
pub mod payload_receiver;

pub use client_data_loader::ClientModDataRegistry;
pub use handshake::{ClientCapabilities, HandshakeState, ModHandshake};
pub use payload_cache::PayloadCache;
pub use payload_receiver::PayloadReceiver;
