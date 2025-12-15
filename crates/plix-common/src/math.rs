//! Math types for Plix
//!
//! Re-exports glam types with custom traits and additional utilities.

use serde::{Deserialize, Serialize};

/// 3D position/vector (re-export glam::Vec3)
pub type Vec3 = glam::Vec3;

/// Rotation (yaw/pitch in radians)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rotation {
    /// Horizontal rotation (-PI to PI)
    pub yaw: f32,
    /// Vertical rotation (-PI/2 to PI/2)
    pub pitch: f32,
}

impl Rotation {
    /// Create a new rotation from yaw and pitch
    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }

    /// Zero rotation (facing +Z)
    pub const ZERO: Self = Self {
        yaw: 0.0,
        pitch: 0.0,
    };

    /// Get forward direction vector
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(sin_yaw * cos_pitch, sin_pitch, cos_yaw * cos_pitch)
    }

    /// Get right direction vector
    pub fn right(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_yaw, 0.0, -sin_yaw)
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB from min and max corners
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB centered at position with given half-extents
    pub fn from_center(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if this AABB intersects another
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Check if a point is inside this AABB
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get the center of this AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of this AABB
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_forward() {
        let rot = Rotation::ZERO;
        let fwd = rot.forward();
        assert!((fwd.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_intersects() {
        let a = AABB::new(Vec3::ZERO, Vec3::ONE);
        let b = AABB::new(Vec3::splat(0.5), Vec3::splat(1.5));
        assert!(a.intersects(&b));

        let c = AABB::new(Vec3::splat(2.0), Vec3::splat(3.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_aabb_contains() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains(Vec3::splat(0.5)));
        assert!(!aabb.contains(Vec3::splat(2.0)));
    }
}
