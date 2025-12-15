//! FPS camera

use plix_common::math::{Rotation, Vec3};

/// First-person camera
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position (eye point)
    pub position: Vec3,
    /// Camera rotation
    pub rotation: Rotation,
    /// Field of view in radians
    pub fov: f32,
    /// Near clip plane
    pub near: f32,
    /// Far clip plane
    pub far: f32,
    /// Aspect ratio (width/height)
    pub aspect: f32,
}

impl Camera {
    /// Create a new camera
    pub fn new(fov_degrees: f32, aspect: f32) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Rotation::ZERO,
            fov: fov_degrees.to_radians(),
            near: 0.1,
            far: 1000.0,
            aspect,
        }
    }

    /// Get forward direction
    pub fn forward(&self) -> Vec3 {
        self.rotation.forward()
    }

    /// Get right direction
    pub fn right(&self) -> Vec3 {
        self.rotation.right()
    }

    /// Get up direction
    pub fn up(&self) -> Vec3 {
        self.forward().cross(self.right())
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> glam::Mat4 {
        let forward = self.forward();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let target = self.position + forward;

        glam::Mat4::look_at_rh(
            glam::Vec3::new(self.position.x, self.position.y, self.position.z),
            glam::Vec3::new(target.x, target.y, target.z),
            glam::Vec3::new(up.x, up.y, up.z),
        )
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> glam::Mat4 {
        glam::Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update camera from player state
    pub fn follow_player(&mut self, position: Vec3, rotation: Rotation, eye_height: f32) {
        self.position = Vec3::new(position.x, position.y + eye_height, position.z);
        self.rotation = rotation;
    }

    /// Update aspect ratio
    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// T064: Set field of view in degrees
    pub fn set_fov(&mut self, fov_degrees: f32) {
        self.fov = fov_degrees.to_radians();
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(70.0, 16.0 / 9.0)
    }
}
