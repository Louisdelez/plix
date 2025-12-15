//! Player rendering (placeholder)

use plix_common::math::{Rotation, Vec3};
use plix_common::protocol::AnimationState;

/// Player renderer (placeholder)
#[derive(Debug, Default)]
pub struct PlayerRenderer;

impl PlayerRenderer {
    /// Create a new player renderer
    pub fn new() -> Self {
        Self
    }

    /// Render a player at position
    pub fn render_player(
        &self,
        _position: Vec3,
        _rotation: Rotation,
        _animation: AnimationState,
        _is_local: bool,
    ) {
        // TODO: Render player capsule/model
    }
}
