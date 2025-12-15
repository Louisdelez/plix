//! Input capture for client

use plix_common::math::Rotation;
use plix_common::protocol::PlayerInput;
use plix_common::time::Tick;
use plix_common::types::InputSeq;

/// Input state from current frame
#[derive(Debug, Default, Clone)]
pub struct InputState {
    /// Forward/backward movement (-1 to 1)
    pub move_forward: f32,
    /// Left/right movement (-1 to 1)
    pub move_right: f32,
    /// Jump pressed
    pub jump: bool,
    /// Crouch pressed
    pub crouch: bool,
    /// Attack pressed
    pub attack: bool,
    /// Remove block pressed (LMB)
    pub remove_block: bool,
    /// Place block pressed (RMB)
    pub place_block: bool,
    /// Mouse delta X
    pub mouse_dx: f32,
    /// Mouse delta Y
    pub mouse_dy: f32,
}

/// Input manager
#[derive(Debug)]
pub struct InputManager {
    /// Current input state
    current: InputState,
    /// Current look rotation
    rotation: Rotation,
    /// Mouse sensitivity
    sensitivity: f32,
    /// Next input sequence number
    next_seq: InputSeq,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            current: InputState::default(),
            rotation: Rotation::ZERO,
            sensitivity: 0.003, // Default mouse sensitivity
            next_seq: InputSeq(0),
        }
    }

    /// Update input state from key presses
    pub fn set_key(&mut self, key: Key, pressed: bool) {
        match key {
            Key::W => {
                if pressed {
                    self.current.move_forward = 1.0;
                } else if self.current.move_forward > 0.0 {
                    self.current.move_forward = 0.0;
                }
            }
            Key::S => {
                if pressed {
                    self.current.move_forward = -1.0;
                } else if self.current.move_forward < 0.0 {
                    self.current.move_forward = 0.0;
                }
            }
            Key::A => {
                if pressed {
                    self.current.move_right = -1.0;
                } else if self.current.move_right < 0.0 {
                    self.current.move_right = 0.0;
                }
            }
            Key::D => {
                if pressed {
                    self.current.move_right = 1.0;
                } else if self.current.move_right > 0.0 {
                    self.current.move_right = 0.0;
                }
            }
            Key::Space => self.current.jump = pressed,
            Key::Ctrl => self.current.crouch = pressed,
            Key::LeftClick => {
                self.current.attack = pressed;
                self.current.remove_block = pressed;
            }
            Key::RightClick => {
                self.current.place_block = pressed;
            }
        }
    }

    /// Update look rotation from mouse movement
    pub fn mouse_move(&mut self, dx: f32, dy: f32) {
        self.current.mouse_dx += dx;
        self.current.mouse_dy += dy;

        self.rotation.yaw -= dx * self.sensitivity;
        self.rotation.pitch -= dy * self.sensitivity;

        // Clamp pitch
        self.rotation.pitch = self.rotation.pitch.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        // Wrap yaw
        if self.rotation.yaw > std::f32::consts::PI {
            self.rotation.yaw -= std::f32::consts::TAU;
        } else if self.rotation.yaw < -std::f32::consts::PI {
            self.rotation.yaw += std::f32::consts::TAU;
        }
    }

    /// Generate input for current tick
    pub fn generate_input(&mut self, tick: Tick) -> PlayerInput {
        let input = PlayerInput {
            seq: self.next_seq,
            tick,
            move_forward: self.current.move_forward,
            move_right: self.current.move_right,
            jump: self.current.jump,
            crouch: self.current.crouch,
            attack: self.current.attack,
            yaw: self.rotation.yaw,
            pitch: self.rotation.pitch,
        };

        self.next_seq = self.next_seq.next();

        // Reset per-frame states
        self.current.jump = false;
        self.current.attack = false;
        self.current.remove_block = false;
        self.current.place_block = false;
        self.current.mouse_dx = 0.0;
        self.current.mouse_dy = 0.0;

        input
    }

    /// Check if remove block was pressed this frame
    pub fn remove_block_pressed(&self) -> bool {
        self.current.remove_block
    }

    /// Check if place block was pressed this frame
    pub fn place_block_pressed(&self) -> bool {
        self.current.place_block
    }

    /// Get current rotation
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    /// Set mouse sensitivity
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.0001, 0.01);
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Key enumeration for input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    W,
    A,
    S,
    D,
    Space,
    Ctrl,
    LeftClick,
    RightClick,
}
