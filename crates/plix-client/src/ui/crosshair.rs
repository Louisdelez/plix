//! Crosshair display for gameplay
//!
//! T035: Crosshair struct with render() method
//! T036: Crosshair as 2 white rectangles (horizontal + vertical)

use crate::render::UIQuad;

/// Crosshair configuration
pub struct Crosshair {
    /// Length of each crosshair line in pixels
    pub length: f32,
    /// Thickness of each crosshair line in pixels
    pub thickness: f32,
    /// Gap in the center (empty space around center point)
    pub gap: f32,
    /// Color (RGBA)
    pub color: [f32; 4],
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            length: 10.0,
            thickness: 2.0,
            gap: 2.0,
            color: [1.0, 1.0, 1.0, 0.9], // White with slight transparency
        }
    }
}

impl Crosshair {
    /// Create a new crosshair with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// T036: Generate UI quads for the crosshair (2 rectangles: horizontal + vertical)
    /// Returns quads centered at screen center (0, 0)
    pub fn render(&self) -> Vec<UIQuad> {
        let mut quads = Vec::with_capacity(4);

        // Horizontal line - left segment
        quads.push(UIQuad {
            x: -(self.gap + self.length / 2.0),
            y: 0.0,
            width: self.length,
            height: self.thickness,
            color: self.color,
        });

        // Horizontal line - right segment
        quads.push(UIQuad {
            x: self.gap + self.length / 2.0,
            y: 0.0,
            width: self.length,
            height: self.thickness,
            color: self.color,
        });

        // Vertical line - top segment
        quads.push(UIQuad {
            x: 0.0,
            y: -(self.gap + self.length / 2.0),
            width: self.thickness,
            height: self.length,
            color: self.color,
        });

        // Vertical line - bottom segment
        quads.push(UIQuad {
            x: 0.0,
            y: self.gap + self.length / 2.0,
            width: self.thickness,
            height: self.length,
            color: self.color,
        });

        quads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crosshair_default() {
        let ch = Crosshair::default();
        assert!((ch.length - 10.0).abs() < f32::EPSILON);
        assert!((ch.thickness - 2.0).abs() < f32::EPSILON);
        assert!((ch.gap - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_crosshair_render_produces_4_quads() {
        let ch = Crosshair::new();
        let quads = ch.render();
        assert_eq!(quads.len(), 4);
    }

    #[test]
    fn test_crosshair_quads_centered() {
        let ch = Crosshair::new();
        let quads = ch.render();

        // Horizontal quads should have y = 0
        assert!((quads[0].y).abs() < f32::EPSILON);
        assert!((quads[1].y).abs() < f32::EPSILON);

        // Vertical quads should have x = 0
        assert!((quads[2].x).abs() < f32::EPSILON);
        assert!((quads[3].x).abs() < f32::EPSILON);
    }
}
