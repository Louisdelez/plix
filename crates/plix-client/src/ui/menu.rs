//! Pause menu and settings menu
//!
//! T042: PauseMenuItem enum (Resume, Settings, Quit)
//! T043: PauseMenu struct with selected item and navigation
//! T045: Render pause menu as colored rectangles

use crate::render::UIQuad;

/// T042: Pause menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuItem {
    Resume,
    Settings,
    Quit,
}

impl PauseMenuItem {
    /// Get all menu items in display order
    pub fn all() -> &'static [PauseMenuItem] {
        &[
            PauseMenuItem::Resume,
            PauseMenuItem::Settings,
            PauseMenuItem::Quit,
        ]
    }

    /// Get display name for the menu item
    pub fn display_name(&self) -> &'static str {
        match self {
            PauseMenuItem::Resume => "Resume",
            PauseMenuItem::Settings => "Settings",
            PauseMenuItem::Quit => "Quit",
        }
    }
}

/// T043: Pause menu state with selection and navigation
pub struct PauseMenu {
    /// Currently selected item index
    selected: usize,
}

impl Default for PauseMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl PauseMenu {
    /// Create a new pause menu with Resume selected
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> PauseMenuItem {
        PauseMenuItem::all()[self.selected]
    }

    /// Move selection up (wrap around)
    pub fn move_up(&mut self) {
        let items = PauseMenuItem::all();
        if self.selected == 0 {
            self.selected = items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Move selection down (wrap around)
    pub fn move_down(&mut self) {
        let items = PauseMenuItem::all();
        self.selected = (self.selected + 1) % items.len();
    }

    /// Reset selection to first item (Resume)
    pub fn reset(&mut self) {
        self.selected = 0;
    }

    /// T045: Render pause menu as colored rectangles
    /// Returns UI quads centered on screen
    pub fn render(&self, screen_width: f32, screen_height: f32) -> Vec<UIQuad> {
        let mut quads = Vec::new();

        let items = PauseMenuItem::all();
        let item_height = 40.0;
        let item_width = 200.0;
        let item_spacing = 10.0;

        // Calculate total menu height
        let total_height =
            (items.len() as f32) * item_height + ((items.len() - 1) as f32) * item_spacing;

        // Semi-transparent background overlay
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
            color: [0.0, 0.0, 0.0, 0.5], // Dark semi-transparent
        });

        // Menu background box
        let menu_bg_padding = 20.0;
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: item_width + menu_bg_padding * 2.0,
            height: total_height + menu_bg_padding * 2.0,
            color: [0.2, 0.2, 0.25, 0.9],
        });

        // Render each menu item
        let start_y = -total_height / 2.0 + item_height / 2.0;

        for (i, item) in items.iter().enumerate() {
            let y = start_y + (i as f32) * (item_height + item_spacing);

            // Item background (highlighted if selected)
            let bg_color = if i == self.selected {
                [0.4, 0.4, 0.8, 1.0] // Highlighted blue
            } else {
                [0.3, 0.3, 0.35, 0.8] // Normal gray
            };

            quads.push(UIQuad {
                x: 0.0,
                y,
                width: item_width,
                height: item_height,
                color: bg_color,
            });

            // For text rendering, we'd need font support
            // For now, just show the selection indicator as a small bar
            if i == self.selected {
                quads.push(UIQuad {
                    x: -item_width / 2.0 + 5.0,
                    y,
                    width: 4.0,
                    height: item_height - 8.0,
                    color: [1.0, 1.0, 1.0, 1.0], // White selection indicator
                });
            }

            // Item label placeholder - small indicator squares to show item index
            // In a real implementation, this would be text
            let indicator_x = -item_width / 4.0;
            for j in 0..=i {
                quads.push(UIQuad {
                    x: indicator_x + (j as f32) * 12.0,
                    y,
                    width: 8.0,
                    height: 8.0,
                    color: [1.0, 1.0, 1.0, 0.9],
                });
            }
        }

        quads
    }
}

/// T053: Settings menu items (for Phase 5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuItem {
    Sensitivity,
    Fov,
    Fullscreen,
    Audio,
    Keybinds,
    Back,
}

impl SettingsMenuItem {
    /// Get all settings items in display order
    pub fn all() -> &'static [SettingsMenuItem] {
        &[
            SettingsMenuItem::Sensitivity,
            SettingsMenuItem::Fov,
            SettingsMenuItem::Fullscreen,
            SettingsMenuItem::Audio,
            SettingsMenuItem::Keybinds,
            SettingsMenuItem::Back,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsMenuItem::Sensitivity => "Sensitivity",
            SettingsMenuItem::Fov => "Field of View",
            SettingsMenuItem::Fullscreen => "Fullscreen",
            SettingsMenuItem::Audio => "Audio",
            SettingsMenuItem::Keybinds => "Keybinds",
            SettingsMenuItem::Back => "Back",
        }
    }
}

/// Settings menu state
pub struct SettingsMenu {
    /// Currently selected item index
    selected: usize,
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsMenu {
    /// Create new settings menu
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> SettingsMenuItem {
        SettingsMenuItem::all()[self.selected]
    }

    /// Move selection up (wrap around)
    pub fn move_up(&mut self) {
        let items = SettingsMenuItem::all();
        if self.selected == 0 {
            self.selected = items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Move selection down (wrap around)
    pub fn move_down(&mut self) {
        let items = SettingsMenuItem::all();
        self.selected = (self.selected + 1) % items.len();
    }

    /// Reset selection
    pub fn reset(&mut self) {
        self.selected = 0;
    }

    /// Render settings menu
    pub fn render(&self, screen_width: f32, screen_height: f32) -> Vec<UIQuad> {
        let mut quads = Vec::new();

        let items = SettingsMenuItem::all();
        let item_height = 35.0;
        let item_width = 250.0;
        let item_spacing = 8.0;

        let total_height =
            (items.len() as f32) * item_height + ((items.len() - 1) as f32) * item_spacing;

        // Semi-transparent background
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
            color: [0.0, 0.0, 0.0, 0.6],
        });

        // Menu background
        let menu_bg_padding = 20.0;
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: item_width + menu_bg_padding * 2.0,
            height: total_height + menu_bg_padding * 2.0,
            color: [0.15, 0.15, 0.2, 0.95],
        });

        let start_y = -total_height / 2.0 + item_height / 2.0;

        for (i, _item) in items.iter().enumerate() {
            let y = start_y + (i as f32) * (item_height + item_spacing);

            let bg_color = if i == self.selected {
                [0.3, 0.5, 0.7, 1.0] // Highlighted blue
            } else {
                [0.25, 0.25, 0.3, 0.8]
            };

            quads.push(UIQuad {
                x: 0.0,
                y,
                width: item_width,
                height: item_height,
                color: bg_color,
            });

            // Selection indicator
            if i == self.selected {
                quads.push(UIQuad {
                    x: -item_width / 2.0 + 4.0,
                    y,
                    width: 3.0,
                    height: item_height - 6.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }

            // Item indicator dots (simple visual without text)
            let indicator_x = -item_width / 4.0;
            for j in 0..=i {
                quads.push(UIQuad {
                    x: indicator_x + (j as f32) * 10.0,
                    y,
                    width: 6.0,
                    height: 6.0,
                    color: [1.0, 1.0, 1.0, 0.8],
                });
            }
        }

        quads
    }
}

/// T080: Keybinds menu items - one per rebindable action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindsMenuItem {
    Forward,
    Backward,
    StrafeLeft,
    StrafeRight,
    Jump,
    Attack,
    PlaceBlock,
    RemoveBlock,
    Pause,
    Back,
}

impl KeybindsMenuItem {
    /// Get all menu items in display order
    pub fn all() -> &'static [KeybindsMenuItem] {
        &[
            KeybindsMenuItem::Forward,
            KeybindsMenuItem::Backward,
            KeybindsMenuItem::StrafeLeft,
            KeybindsMenuItem::StrafeRight,
            KeybindsMenuItem::Jump,
            KeybindsMenuItem::Attack,
            KeybindsMenuItem::PlaceBlock,
            KeybindsMenuItem::RemoveBlock,
            KeybindsMenuItem::Pause,
            KeybindsMenuItem::Back,
        ]
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            KeybindsMenuItem::Forward => "Forward",
            KeybindsMenuItem::Backward => "Backward",
            KeybindsMenuItem::StrafeLeft => "Strafe Left",
            KeybindsMenuItem::StrafeRight => "Strafe Right",
            KeybindsMenuItem::Jump => "Jump",
            KeybindsMenuItem::Attack => "Attack",
            KeybindsMenuItem::PlaceBlock => "Place Block",
            KeybindsMenuItem::RemoveBlock => "Remove Block",
            KeybindsMenuItem::Pause => "Pause",
            KeybindsMenuItem::Back => "Back",
        }
    }

    /// Convert to Action if applicable
    pub fn to_action(&self) -> Option<crate::config::Action> {
        use crate::config::Action;
        match self {
            KeybindsMenuItem::Forward => Some(Action::Forward),
            KeybindsMenuItem::Backward => Some(Action::Backward),
            KeybindsMenuItem::StrafeLeft => Some(Action::Left),
            KeybindsMenuItem::StrafeRight => Some(Action::Right),
            KeybindsMenuItem::Jump => Some(Action::Jump),
            KeybindsMenuItem::Attack => Some(Action::Attack),
            KeybindsMenuItem::PlaceBlock => Some(Action::PlaceBlock),
            KeybindsMenuItem::RemoveBlock => Some(Action::RemoveBlock),
            KeybindsMenuItem::Pause => Some(Action::Pause),
            KeybindsMenuItem::Back => None,
        }
    }
}

/// T081: Keybinds menu state with action list and navigation
pub struct KeybindsMenu {
    /// Currently selected item index
    selected: usize,
}

impl Default for KeybindsMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl KeybindsMenu {
    /// Create new keybinds menu
    pub fn new() -> Self {
        Self { selected: 0 }
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> KeybindsMenuItem {
        KeybindsMenuItem::all()[self.selected]
    }

    /// Move selection up (wrap around)
    pub fn move_up(&mut self) {
        let items = KeybindsMenuItem::all();
        if self.selected == 0 {
            self.selected = items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Move selection down (wrap around)
    pub fn move_down(&mut self) {
        let items = KeybindsMenuItem::all();
        self.selected = (self.selected + 1) % items.len();
    }

    /// Reset selection
    pub fn reset(&mut self) {
        self.selected = 0;
    }

    /// T083: Render keybinds menu showing action -> key mappings
    pub fn render(
        &self,
        screen_width: f32,
        screen_height: f32,
        keybinds: &crate::config::Keybinds,
    ) -> Vec<UIQuad> {
        let mut quads = Vec::new();

        let items = KeybindsMenuItem::all();
        let item_height = 32.0;
        let item_width = 300.0;
        let item_spacing = 6.0;

        let total_height =
            (items.len() as f32) * item_height + ((items.len() - 1) as f32) * item_spacing;

        // Semi-transparent background
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: screen_width,
            height: screen_height,
            color: [0.0, 0.0, 0.0, 0.7],
        });

        // Menu background
        let menu_bg_padding = 20.0;
        quads.push(UIQuad {
            x: 0.0,
            y: 0.0,
            width: item_width + menu_bg_padding * 2.0,
            height: total_height + menu_bg_padding * 2.0,
            color: [0.1, 0.12, 0.15, 0.95],
        });

        let start_y = -total_height / 2.0 + item_height / 2.0;

        for (i, item) in items.iter().enumerate() {
            let y = start_y + (i as f32) * (item_height + item_spacing);

            let bg_color = if i == self.selected {
                [0.25, 0.45, 0.65, 1.0] // Highlighted blue
            } else {
                [0.2, 0.2, 0.25, 0.8]
            };

            quads.push(UIQuad {
                x: 0.0,
                y,
                width: item_width,
                height: item_height,
                color: bg_color,
            });

            // Selection indicator
            if i == self.selected {
                quads.push(UIQuad {
                    x: -item_width / 2.0 + 3.0,
                    y,
                    width: 3.0,
                    height: item_height - 4.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }

            // Action name indicator (left side - dots for action index)
            let action_x = -item_width / 2.0 + 15.0;
            for j in 0..=i.min(8) {
                quads.push(UIQuad {
                    x: action_x + (j as f32) * 8.0,
                    y,
                    width: 5.0,
                    height: 5.0,
                    color: [1.0, 1.0, 1.0, 0.7],
                });
            }

            // Key binding indicator (right side)
            // Show different colors based on key type
            if let Some(action) = item.to_action() {
                if let Some(key) = keybinds.get(action) {
                    // Show key indicator on right side
                    let key_x = item_width / 2.0 - 30.0;
                    let key_color = match key {
                        crate::config::Key::LeftClick
                        | crate::config::Key::RightClick
                        | crate::config::Key::MiddleClick => [0.8, 0.4, 0.4, 1.0], // Red for mouse
                        crate::config::Key::Space => [0.4, 0.8, 0.4, 1.0], // Green for space
                        _ => [0.6, 0.6, 0.8, 1.0],                         // Light blue for keys
                    };

                    quads.push(UIQuad {
                        x: key_x,
                        y,
                        width: 40.0,
                        height: item_height - 8.0,
                        color: key_color,
                    });
                }
            }
        }

        quads
    }

    /// Render with "Press key..." indicator for rebinding state
    pub fn render_rebinding(
        &self,
        screen_width: f32,
        screen_height: f32,
        keybinds: &crate::config::Keybinds,
    ) -> Vec<UIQuad> {
        let mut quads = self.render(screen_width, screen_height, keybinds);

        // Add "Press key..." overlay
        quads.push(UIQuad {
            x: 0.0,
            y: 80.0,
            width: 200.0,
            height: 40.0,
            color: [0.8, 0.6, 0.2, 1.0], // Orange indicator
        });

        quads
    }

    /// Render with swap confirmation indicator
    pub fn render_confirm_swap(
        &self,
        screen_width: f32,
        screen_height: f32,
        keybinds: &crate::config::Keybinds,
    ) -> Vec<UIQuad> {
        let mut quads = self.render(screen_width, screen_height, keybinds);

        // Add swap confirmation overlay
        quads.push(UIQuad {
            x: 0.0,
            y: 80.0,
            width: 250.0,
            height: 50.0,
            color: [0.7, 0.3, 0.3, 1.0], // Red-ish for conflict warning
        });

        // Confirm/Cancel buttons indicators
        quads.push(UIQuad {
            x: -50.0,
            y: 130.0,
            width: 70.0,
            height: 30.0,
            color: [0.3, 0.6, 0.3, 1.0], // Green for confirm
        });

        quads.push(UIQuad {
            x: 50.0,
            y: 130.0,
            width: 70.0,
            height: 30.0,
            color: [0.6, 0.3, 0.3, 1.0], // Red for cancel
        });

        quads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_menu_default_selection() {
        let menu = PauseMenu::new();
        assert_eq!(menu.selected_item(), PauseMenuItem::Resume);
    }

    #[test]
    fn test_pause_menu_navigation() {
        let mut menu = PauseMenu::new();

        menu.move_down();
        assert_eq!(menu.selected_item(), PauseMenuItem::Settings);

        menu.move_down();
        assert_eq!(menu.selected_item(), PauseMenuItem::Quit);

        // Wrap around
        menu.move_down();
        assert_eq!(menu.selected_item(), PauseMenuItem::Resume);

        // Up wrap
        menu.move_up();
        assert_eq!(menu.selected_item(), PauseMenuItem::Quit);
    }

    #[test]
    fn test_pause_menu_reset() {
        let mut menu = PauseMenu::new();
        menu.move_down();
        menu.move_down();
        menu.reset();
        assert_eq!(menu.selected_item(), PauseMenuItem::Resume);
    }

    #[test]
    fn test_pause_menu_renders_quads() {
        let menu = PauseMenu::new();
        let quads = menu.render(1280.0, 720.0);
        // Should have: overlay + bg + 3 items + selection indicator + item indicators
        assert!(quads.len() >= 5);
    }

    #[test]
    fn test_settings_menu_navigation() {
        let mut menu = SettingsMenu::new();

        assert_eq!(menu.selected_item(), SettingsMenuItem::Sensitivity);

        menu.move_down();
        assert_eq!(menu.selected_item(), SettingsMenuItem::Fov);

        menu.move_up();
        assert_eq!(menu.selected_item(), SettingsMenuItem::Sensitivity);

        // Wrap up
        menu.move_up();
        assert_eq!(menu.selected_item(), SettingsMenuItem::Back);
    }

    #[test]
    fn test_keybinds_menu_navigation() {
        let mut menu = KeybindsMenu::new();

        assert_eq!(menu.selected_item(), KeybindsMenuItem::Forward);

        menu.move_down();
        assert_eq!(menu.selected_item(), KeybindsMenuItem::Backward);

        menu.move_down();
        assert_eq!(menu.selected_item(), KeybindsMenuItem::StrafeLeft);

        menu.move_up();
        assert_eq!(menu.selected_item(), KeybindsMenuItem::Backward);

        // Wrap up to last item
        menu.reset();
        menu.move_up();
        assert_eq!(menu.selected_item(), KeybindsMenuItem::Back);
    }

    #[test]
    fn test_keybinds_menu_to_action() {
        use crate::config::Action;

        assert_eq!(KeybindsMenuItem::Forward.to_action(), Some(Action::Forward));
        assert_eq!(KeybindsMenuItem::Jump.to_action(), Some(Action::Jump));
        assert_eq!(KeybindsMenuItem::Back.to_action(), None);
    }

    #[test]
    fn test_keybinds_menu_renders_quads() {
        use crate::config::Keybinds;

        let menu = KeybindsMenu::new();
        let keybinds = Keybinds::default();
        let quads = menu.render(1280.0, 720.0, &keybinds);

        // Should have overlay + bg + items + selection indicators + key indicators
        assert!(quads.len() >= 12);
    }
}
