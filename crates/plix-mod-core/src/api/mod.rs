//! API modules for mod interaction with game systems

pub mod entities;
pub mod net;
pub mod timers;
pub mod world;

// Re-export key types
pub use entities::{EntityApi, EntityHandle};
pub use net::{MessageTarget, ModChannel, NetApi};
pub use timers::{TimerApi, TimerHandle};
pub use world::{BlockQuery, RaycastHit, WorldApi};
