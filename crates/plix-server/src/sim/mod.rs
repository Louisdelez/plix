//! Server-side simulation

pub mod block_edit;
pub mod collision;
pub mod combat;
pub mod movement;

pub use block_edit::BlockEditSystem;
pub use collision::CollisionWorld;
pub use combat::CombatSystem;
pub use movement::MovementSystem;

// Re-export unified hitbox constants from collision module
// Combat and movement use the same capsule dimensions
pub use collision::{PLAYER_HALF_HEIGHT, PLAYER_HALF_WIDTH};
pub use movement::{PLAYER_HEIGHT, PLAYER_RADIUS};
