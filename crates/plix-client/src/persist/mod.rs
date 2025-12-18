//! World Persistence Module for plix-client.
//!
//! Provides solo mode world persistence.
//!
//! # Usage
//!
//! ```rust,ignore
//! use plix_client::persist::{LocalStore, list_worlds};
//!
//! // List available worlds
//! let worlds = list_worlds()?;
//!
//! // Create or open a world
//! let mut store = LocalStore::open_or_create("My World", 12345)?;
//!
//! // Load chunks into world
//! store.load_all_chunks(&mut world)?;
//!
//! // ... play ...
//!
//! // Save on quit
//! store.save_world(&world)?;
//! ```

pub mod local_store;

pub use local_store::{default_worlds_dir, delete_world, list_worlds, LocalStore};
