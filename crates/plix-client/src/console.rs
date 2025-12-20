//! Console command system for client-side slash commands.
//!
//! Supports economy commands: /balance, /buy, /shop
//! Supports identity commands: /name
//! Supports server browser commands: /servers, /connect
//! Supports matchmaking commands: /quickjoin, /play, /quickjoin-prefs
//! Supports accessibility commands: /rebind, /ui_scale, /colorblind, /highcontrast, /subtitles
//! Supports quest debug commands: /quest (Feature 043)

use plix_common::protocol::ClientMessage;

use crate::accessibility::{ColorblindPreset, UI_SCALE_MAX, UI_SCALE_MIN};
use crate::config::{Action, Key};
use crate::matchmaking::request::{VALID_MODES, VALID_REGIONS};

/// Result of parsing a console command
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command parsed successfully - send this message to server
    SendMessage(ClientMessage),
    /// Command parsed but handled client-side (no server message needed)
    ClientOnly(String),
    /// Not a command (regular chat message)
    NotACommand,
    /// Invalid command syntax
    InvalidSyntax(String),
    /// Unknown command
    UnknownCommand(String),
    /// Server browser command - refresh server list
    RefreshServers,
    /// Server browser command - connect to server by index
    ConnectToServer(usize),
    /// Quick join command - find and connect to a server matching criteria
    QuickJoin {
        /// Game mode (e.g., "tdm", "ffa", "ctf", "any")
        mode: String,
        /// Region (e.g., "eu", "na", "asia", "any")
        region: String,
    },
    /// View or modify quick join preferences
    QuickJoinPrefs {
        /// What to do: "view", "mode", or "region"
        action: String,
        /// New value for mode/region (None for view)
        value: Option<String>,
    },
    // === Feature 042: Accessibility commands ===
    /// Rebind an action to a key
    RebindAction {
        action: Action,
        key: Key,
    },
    /// List all keybindings
    RebindList,
    /// Reset keybindings to defaults
    RebindReset,
    /// Set UI scale percentage
    SetUiScale(u8),
    /// Set colorblind preset
    SetColorblind(ColorblindPreset),
    /// Toggle high contrast mode
    SetHighContrast(bool),
    /// Toggle subtitles
    SetSubtitles(bool),

    // === Feature 043: Quest debug commands ===
    /// List active quests
    QuestList,
    /// Show quest details
    QuestInfo { quest_id: String },
    /// Accept a quest (debug)
    QuestAccept { quest_id: String },
    /// Abandon a quest (debug)
    QuestAbandon { quest_id: String },
    /// Pin a quest to HUD
    QuestPin { quest_id: String },
    /// Unpin the current pinned quest
    QuestUnpin,

    // === Feature 043: Dungeon debug commands ===
    /// List available dungeons
    DungeonList,
    /// Show dungeon info
    DungeonInfo { dungeon_id: String },
    /// Reset dungeon state (debug)
    DungeonReset { dungeon_id: String },
    /// Complete dungeon (debug)
    DungeonComplete { dungeon_id: String },
}

/// Parse a console input string and return the appropriate action.
///
/// Commands start with `/`. Supported commands:
/// - `/balance` or `/bal` - Request current coin balance
/// - `/buy <offer_id>` - Purchase an item from the shop
/// - `/shop` - Request list of available shop offers
/// - `/name <new_name>` - Change your display name (60s cooldown)
/// - `/servers` - Refresh and display server list from master
/// - `/connect <index>` - Connect to a server by index from list
/// - `/quickjoin [mode] [region]` - Quick join a server (mode/region optional)
/// - `/play [mode] [region]` - Alias for /quickjoin
/// - `/quickjoin-prefs` - View matchmaking preferences
/// - `/quickjoin-prefs mode <value>` - Set preferred mode
/// - `/quickjoin-prefs region <value>` - Set preferred region
/// - `/rebind <action> <key>` - Rebind an action to a key
/// - `/rebind list` - List all keybindings
/// - `/rebind reset` - Reset keybindings to defaults
/// - `/ui_scale <75-150>` - Set UI scale percentage
/// - `/colorblind <preset>` - Set colorblind preset (none, protanopia, deuteranopia, tritanopia)
/// - `/highcontrast <on|off>` - Toggle high contrast mode
/// - `/subtitles <on|off>` - Toggle subtitles
/// - `/quest list` - List active quests (debug)
/// - `/quest info <id>` - Show quest details (debug)
/// - `/quest accept <id>` - Accept a quest (debug)
/// - `/quest abandon <id>` - Abandon a quest (debug)
/// - `/quest pin <id>` - Pin a quest to HUD
/// - `/quest unpin` - Unpin the current quest
/// - `/dungeon list` - List available dungeons (debug)
/// - `/dungeon info <id>` - Show dungeon info (debug)
/// - `/dungeon reset <id>` - Reset dungeon state (debug)
/// - `/dungeon complete <id>` - Complete dungeon (debug)
///
/// Returns `CommandResult` indicating what action to take.
pub fn parse_command(input: &str) -> CommandResult {
    let input = input.trim();

    // Not a command if it doesn't start with /
    if !input.starts_with('/') {
        return CommandResult::NotACommand;
    }

    // Remove the leading / and split into parts
    let command_str = &input[1..];
    let mut parts = command_str.split_whitespace();

    let command = match parts.next() {
        Some(cmd) => cmd.to_lowercase(),
        None => return CommandResult::InvalidSyntax("Empty command".to_string()),
    };

    match command.as_str() {
        "balance" | "bal" => CommandResult::SendMessage(ClientMessage::BalanceRequest),

        "buy" => {
            match parts.next() {
                Some(offer_id) => {
                    // Validate offer_id is not empty and alphanumeric with underscores
                    let offer_id = offer_id.to_string();
                    if offer_id.is_empty() {
                        return CommandResult::InvalidSyntax("Usage: /buy <offer_id>".to_string());
                    }
                    if !offer_id.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return CommandResult::InvalidSyntax(
                            "Offer ID must be alphanumeric with underscores".to_string(),
                        );
                    }
                    CommandResult::SendMessage(ClientMessage::BuyRequest { offer_id })
                }
                None => CommandResult::InvalidSyntax("Usage: /buy <offer_id>".to_string()),
            }
        }

        "shop" => CommandResult::SendMessage(ClientMessage::ShopListRequest),

        "name" => {
            // Collect all remaining parts as the new name
            let new_name: String = parts.collect::<Vec<_>>().join(" ");
            if new_name.is_empty() {
                return CommandResult::InvalidSyntax("Usage: /name <new_name>".to_string());
            }
            // Basic client-side validation (server will do full validation)
            if new_name.len() > 32 {
                return CommandResult::InvalidSyntax(
                    "Name must be 32 characters or less".to_string(),
                );
            }
            CommandResult::SendMessage(ClientMessage::RenameRequest { new_name })
        }

        "servers" => CommandResult::RefreshServers,

        "connect" => match parts.next() {
            Some(index_str) => match index_str.parse::<usize>() {
                Ok(index) if index > 0 => CommandResult::ConnectToServer(index),
                Ok(_) => {
                    CommandResult::InvalidSyntax("Server index must be 1 or higher".to_string())
                }
                Err(_) => CommandResult::InvalidSyntax("Usage: /connect <number>".to_string()),
            },
            None => CommandResult::InvalidSyntax("Usage: /connect <number>".to_string()),
        },

        // T016, T017: Quick join commands
        "quickjoin" | "play" => {
            // Parse optional mode and region arguments
            let mode = parts.next().map(|s| s.to_lowercase());
            let region = parts.next().map(|s| s.to_lowercase());

            // T018: Validate mode if provided
            if let Some(ref m) = mode {
                if !VALID_MODES.contains(&m.as_str()) {
                    return CommandResult::InvalidSyntax(format!(
                        "Invalid mode '{}'. Valid modes: {}",
                        m,
                        VALID_MODES.join(", ")
                    ));
                }
            }

            // T018: Validate region if provided
            if let Some(ref r) = region {
                if !VALID_REGIONS.contains(&r.as_str()) {
                    return CommandResult::InvalidSyntax(format!(
                        "Invalid region '{}'. Valid regions: {}",
                        r,
                        VALID_REGIONS.join(", ")
                    ));
                }
            }

            CommandResult::QuickJoin {
                mode: mode.unwrap_or_else(|| "any".to_string()),
                region: region.unwrap_or_else(|| "any".to_string()),
            }
        }

        // T049-T051: Quick join preferences commands
        "quickjoin-prefs" => {
            let action = parts.next().map(|s| s.to_lowercase());

            match action.as_deref() {
                None => {
                    // View current preferences
                    CommandResult::QuickJoinPrefs {
                        action: "view".to_string(),
                        value: None,
                    }
                }
                Some("mode") => {
                    let value = parts.next().map(|s| s.to_lowercase());
                    if let Some(ref v) = value {
                        if !VALID_MODES.contains(&v.as_str()) {
                            return CommandResult::InvalidSyntax(format!(
                                "Invalid mode '{}'. Valid modes: {}",
                                v,
                                VALID_MODES.join(", ")
                            ));
                        }
                    } else {
                        return CommandResult::InvalidSyntax(
                            "Usage: /quickjoin-prefs mode <value>".to_string(),
                        );
                    }
                    CommandResult::QuickJoinPrefs {
                        action: "mode".to_string(),
                        value,
                    }
                }
                Some("region") => {
                    let value = parts.next().map(|s| s.to_lowercase());
                    if let Some(ref v) = value {
                        if !VALID_REGIONS.contains(&v.as_str()) {
                            return CommandResult::InvalidSyntax(format!(
                                "Invalid region '{}'. Valid regions: {}",
                                v,
                                VALID_REGIONS.join(", ")
                            ));
                        }
                    } else {
                        return CommandResult::InvalidSyntax(
                            "Usage: /quickjoin-prefs region <value>".to_string(),
                        );
                    }
                    CommandResult::QuickJoinPrefs {
                        action: "region".to_string(),
                        value,
                    }
                }
                Some(unknown) => CommandResult::InvalidSyntax(format!(
                    "Unknown preference '{}'. Use: mode, region",
                    unknown
                )),
            }
        }

        "help" => {
            let help_text = r#"Available commands:
/balance, /bal  - Show your current coin balance
/buy <offer_id> - Purchase an item from the shop
/shop           - List available shop offers
/name <name>    - Change your display name (60s cooldown)
/servers        - List available game servers
/connect <n>    - Connect to server #n from the list
/quickjoin [mode] [region] - Quick join a matching server
/play [mode] [region]      - Alias for /quickjoin
/quickjoin-prefs           - View matchmaking preferences
/quickjoin-prefs mode <v>  - Set preferred mode
/quickjoin-prefs region <v>- Set preferred region
/rebind <action> <key>     - Rebind an action to a key
/rebind list               - List all keybindings
/rebind reset              - Reset keybindings to defaults
/ui_scale <75-150>         - Set UI scale percentage
/colorblind <preset>       - Set colorblind mode (none/protanopia/deuteranopia/tritanopia)
/highcontrast <on|off>     - Toggle high contrast mode
/subtitles <on|off>        - Toggle subtitles
/quest list                - List active quests
/quest info <id>           - Show quest details
/quest accept <id>         - Accept a quest
/quest abandon <id>        - Abandon a quest
/quest pin <id>            - Pin quest to HUD
/quest unpin               - Unpin current quest
/dungeon list              - List available dungeons
/dungeon info <id>         - Show dungeon info
/dungeon reset <id>        - Reset dungeon state
/dungeon complete <id>     - Complete dungeon
/help           - Show this help message"#;
            CommandResult::ClientOnly(help_text.to_string())
        }

        // === Feature 042: Accessibility commands ===
        "rebind" => {
            let subcommand = parts.next();
            match subcommand {
                Some("list") => CommandResult::RebindList,
                Some("reset") => CommandResult::RebindReset,
                Some(action_str) => {
                    // Parse action
                    let action = parse_action(action_str);
                    if action.is_none() {
                        return CommandResult::InvalidSyntax(format!(
                            "Unknown action '{}'. Valid actions: {}",
                            action_str,
                            action_names().join(", ")
                        ));
                    }
                    let action = action.unwrap();

                    // Parse key
                    match parts.next() {
                        Some(key_str) => {
                            let key = parse_key(key_str);
                            if key.is_none() {
                                return CommandResult::InvalidSyntax(format!(
                                    "Unknown key '{}'. Examples: W, Space, LeftClick, Up",
                                    key_str
                                ));
                            }
                            CommandResult::RebindAction {
                                action,
                                key: key.unwrap(),
                            }
                        }
                        None => CommandResult::InvalidSyntax(
                            "Usage: /rebind <action> <key> or /rebind list or /rebind reset"
                                .to_string(),
                        ),
                    }
                }
                None => CommandResult::InvalidSyntax(
                    "Usage: /rebind <action> <key> or /rebind list or /rebind reset".to_string(),
                ),
            }
        }

        "ui_scale" => match parts.next() {
            Some(value_str) => match value_str.parse::<u8>() {
                Ok(value) if value >= UI_SCALE_MIN && value <= UI_SCALE_MAX => {
                    CommandResult::SetUiScale(value)
                }
                Ok(_) => CommandResult::InvalidSyntax(format!(
                    "UI scale must be between {} and {}",
                    UI_SCALE_MIN, UI_SCALE_MAX
                )),
                Err(_) => CommandResult::InvalidSyntax("Usage: /ui_scale <75-150>".to_string()),
            },
            None => CommandResult::InvalidSyntax("Usage: /ui_scale <75-150>".to_string()),
        },

        "colorblind" => match parts.next() {
            Some(preset_str) => match ColorblindPreset::from_str(preset_str) {
                Some(preset) => CommandResult::SetColorblind(preset),
                None => CommandResult::InvalidSyntax(
                    "Invalid preset. Valid: none, protanopia, deuteranopia, tritanopia".to_string(),
                ),
            },
            None => CommandResult::InvalidSyntax(
                "Usage: /colorblind <preset> (none/protanopia/deuteranopia/tritanopia)".to_string(),
            ),
        },

        "highcontrast" => match parts.next() {
            Some(value) => match value.to_lowercase().as_str() {
                "on" | "true" | "1" | "yes" => CommandResult::SetHighContrast(true),
                "off" | "false" | "0" | "no" => CommandResult::SetHighContrast(false),
                _ => CommandResult::InvalidSyntax("Usage: /highcontrast <on|off>".to_string()),
            },
            None => CommandResult::InvalidSyntax("Usage: /highcontrast <on|off>".to_string()),
        },

        "subtitles" => match parts.next() {
            Some(value) => match value.to_lowercase().as_str() {
                "on" | "true" | "1" | "yes" => CommandResult::SetSubtitles(true),
                "off" | "false" | "0" | "no" => CommandResult::SetSubtitles(false),
                _ => CommandResult::InvalidSyntax("Usage: /subtitles <on|off>".to_string()),
            },
            None => CommandResult::InvalidSyntax("Usage: /subtitles <on|off>".to_string()),
        },

        // === Feature 043: Quest debug commands ===
        "quest" => {
            let subcommand = parts.next();
            match subcommand {
                Some("list") | Some("ls") => CommandResult::QuestList,
                Some("info") | Some("show") => {
                    match parts.next() {
                        Some(quest_id) => CommandResult::QuestInfo {
                            quest_id: quest_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /quest info <quest_id>".to_string()),
                    }
                }
                Some("accept") | Some("start") => {
                    match parts.next() {
                        Some(quest_id) => CommandResult::QuestAccept {
                            quest_id: quest_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /quest accept <quest_id>".to_string()),
                    }
                }
                Some("abandon") | Some("drop") => {
                    match parts.next() {
                        Some(quest_id) => CommandResult::QuestAbandon {
                            quest_id: quest_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /quest abandon <quest_id>".to_string()),
                    }
                }
                Some("pin") | Some("track") => {
                    match parts.next() {
                        Some(quest_id) => CommandResult::QuestPin {
                            quest_id: quest_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /quest pin <quest_id>".to_string()),
                    }
                }
                Some("unpin") | Some("untrack") => CommandResult::QuestUnpin,
                Some(unknown) => CommandResult::InvalidSyntax(format!(
                    "Unknown quest subcommand '{}'. Use: list, info, accept, abandon, pin, unpin",
                    unknown
                )),
                None => CommandResult::InvalidSyntax(
                    "Usage: /quest <list|info|accept|abandon|pin|unpin> [args]".to_string(),
                ),
            }
        }

        // === Feature 043: Dungeon debug commands ===
        "dungeon" => {
            let subcommand = parts.next();
            match subcommand {
                Some("list") | Some("ls") => CommandResult::DungeonList,
                Some("info") | Some("show") => {
                    match parts.next() {
                        Some(dungeon_id) => CommandResult::DungeonInfo {
                            dungeon_id: dungeon_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /dungeon info <dungeon_id>".to_string()),
                    }
                }
                Some("reset") => {
                    match parts.next() {
                        Some(dungeon_id) => CommandResult::DungeonReset {
                            dungeon_id: dungeon_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /dungeon reset <dungeon_id>".to_string()),
                    }
                }
                Some("complete") => {
                    match parts.next() {
                        Some(dungeon_id) => CommandResult::DungeonComplete {
                            dungeon_id: dungeon_id.to_string(),
                        },
                        None => CommandResult::InvalidSyntax("Usage: /dungeon complete <dungeon_id>".to_string()),
                    }
                }
                Some(unknown) => CommandResult::InvalidSyntax(format!(
                    "Unknown dungeon subcommand '{}'. Use: list, info, reset, complete",
                    unknown
                )),
                None => CommandResult::InvalidSyntax(
                    "Usage: /dungeon <list|info|reset|complete> [args]".to_string(),
                ),
            }
        }

        _ => CommandResult::UnknownCommand(format!("Unknown command: /{}", command)),
    }
}

/// Get list of action names for help text
fn action_names() -> Vec<&'static str> {
    Action::all().iter().map(|a| a.display_name()).collect()
}

/// Parse action from string (case-insensitive)
fn parse_action(s: &str) -> Option<Action> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "forward" => Some(Action::Forward),
        "backward" | "back" => Some(Action::Backward),
        "left" => Some(Action::Left),
        "right" => Some(Action::Right),
        "jump" => Some(Action::Jump),
        "attack" => Some(Action::Attack),
        "placeblock" | "place" | "place_block" => Some(Action::PlaceBlock),
        "removeblock" | "remove" | "remove_block" => Some(Action::RemoveBlock),
        "pause" | "menu" | "escape" => Some(Action::Pause),
        "toggledebugoverlay" | "debug" | "debugoverlay" | "toggle_debug_overlay" => {
            Some(Action::ToggleDebugOverlay)
        }
        _ => None,
    }
}

/// Parse key from string (case-insensitive)
fn parse_key(s: &str) -> Option<Key> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        // Letters
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),
        // Numbers
        "0" | "key0" => Some(Key::Key0),
        "1" | "key1" => Some(Key::Key1),
        "2" | "key2" => Some(Key::Key2),
        "3" | "key3" => Some(Key::Key3),
        "4" | "key4" => Some(Key::Key4),
        "5" | "key5" => Some(Key::Key5),
        "6" | "key6" => Some(Key::Key6),
        "7" | "key7" => Some(Key::Key7),
        "8" | "key8" => Some(Key::Key8),
        "9" | "key9" => Some(Key::Key9),
        // Special keys
        "space" => Some(Key::Space),
        "escape" | "esc" => Some(Key::Escape),
        "enter" | "return" => Some(Key::Enter),
        "tab" => Some(Key::Tab),
        "backspace" => Some(Key::Backspace),
        // Modifiers
        "ctrl" | "control" => Some(Key::Ctrl),
        "shift" => Some(Key::Shift),
        "alt" => Some(Key::Alt),
        // Arrow keys
        "up" | "arrowup" | "uparrow" => Some(Key::Up),
        "down" | "arrowdown" | "downarrow" => Some(Key::Down),
        "left" | "arrowleft" | "leftarrow" => Some(Key::Left),
        "right" | "arrowright" | "rightarrow" => Some(Key::Right),
        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),
        // Mouse buttons
        "leftclick" | "lmb" | "mouse1" => Some(Key::LeftClick),
        "rightclick" | "rmb" | "mouse2" => Some(Key::RightClick),
        "middleclick" | "mmb" | "mouse3" => Some(Key::MiddleClick),
        _ => None,
    }
}

/// Format a balance for display
pub fn format_balance(balance: u32) -> String {
    format!("Balance: {} coins", balance)
}

/// Format a purchase result for display
pub fn format_purchase_result(
    success: bool,
    offer_id: &str,
    quantity: Option<u8>,
    fail_reason: Option<&str>,
) -> String {
    if success {
        let qty = quantity.unwrap_or(1);
        if qty > 1 {
            format!("Purchased {}x {}", qty, offer_id)
        } else {
            format!("Purchased {}", offer_id)
        }
    } else {
        match fail_reason {
            Some(reason) => format!("Purchase failed: {}", reason),
            None => format!("Purchase of {} failed", offer_id),
        }
    }
}

/// Format shop offer for display
pub fn format_shop_offer(offer_id: &str, quantity: u8, price: u32) -> String {
    if quantity > 1 {
        format!("  {} (x{}) - {} coins", offer_id, quantity, price)
    } else {
        format!("  {} - {} coins", offer_id, price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_balance_command() {
        match parse_command("/balance") {
            CommandResult::SendMessage(ClientMessage::BalanceRequest) => {}
            _ => panic!("Expected BalanceRequest"),
        }

        match parse_command("/bal") {
            CommandResult::SendMessage(ClientMessage::BalanceRequest) => {}
            _ => panic!("Expected BalanceRequest for /bal"),
        }
    }

    #[test]
    fn test_parse_buy_command() {
        match parse_command("/buy health_pack") {
            CommandResult::SendMessage(ClientMessage::BuyRequest { offer_id }) => {
                assert_eq!(offer_id, "health_pack");
            }
            _ => panic!("Expected BuyRequest"),
        }
    }

    #[test]
    fn test_parse_buy_missing_arg() {
        match parse_command("/buy") {
            CommandResult::InvalidSyntax(_) => {}
            _ => panic!("Expected InvalidSyntax"),
        }
    }

    #[test]
    fn test_parse_shop_command() {
        match parse_command("/shop") {
            CommandResult::SendMessage(ClientMessage::ShopListRequest) => {}
            _ => panic!("Expected ShopListRequest"),
        }
    }

    #[test]
    fn test_parse_help_command() {
        match parse_command("/help") {
            CommandResult::ClientOnly(text) => {
                assert!(text.contains("/balance"));
                assert!(text.contains("/buy"));
                assert!(text.contains("/shop"));
            }
            _ => panic!("Expected ClientOnly help text"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        match parse_command("/unknown") {
            CommandResult::UnknownCommand(msg) => {
                assert!(msg.contains("unknown"));
            }
            _ => panic!("Expected UnknownCommand"),
        }
    }

    #[test]
    fn test_parse_not_a_command() {
        match parse_command("hello world") {
            CommandResult::NotACommand => {}
            _ => panic!("Expected NotACommand"),
        }
    }

    #[test]
    fn test_parse_case_insensitive() {
        match parse_command("/BALANCE") {
            CommandResult::SendMessage(ClientMessage::BalanceRequest) => {}
            _ => panic!("Expected BalanceRequest for uppercase"),
        }

        match parse_command("/BUY sword") {
            CommandResult::SendMessage(ClientMessage::BuyRequest { offer_id }) => {
                assert_eq!(offer_id, "sword");
            }
            _ => panic!("Expected BuyRequest for uppercase"),
        }
    }

    // T059: /name command tests
    #[test]
    fn test_parse_name_command() {
        match parse_command("/name NewPlayer") {
            CommandResult::SendMessage(ClientMessage::RenameRequest { new_name }) => {
                assert_eq!(new_name, "NewPlayer");
            }
            _ => panic!("Expected RenameRequest"),
        }
    }

    #[test]
    fn test_parse_name_command_with_spaces() {
        match parse_command("/name My Cool Name") {
            CommandResult::SendMessage(ClientMessage::RenameRequest { new_name }) => {
                assert_eq!(new_name, "My Cool Name");
            }
            _ => panic!("Expected RenameRequest with spaces"),
        }
    }

    #[test]
    fn test_parse_name_missing_arg() {
        match parse_command("/name") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax"),
        }
    }

    #[test]
    fn test_parse_name_too_long() {
        let long_name = "a".repeat(33);
        match parse_command(&format!("/name {}", long_name)) {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("32 characters"));
            }
            _ => panic!("Expected InvalidSyntax for too long name"),
        }
    }

    #[test]
    fn test_parse_name_case_insensitive() {
        match parse_command("/NAME TestPlayer") {
            CommandResult::SendMessage(ClientMessage::RenameRequest { new_name }) => {
                assert_eq!(new_name, "TestPlayer");
            }
            _ => panic!("Expected RenameRequest for uppercase"),
        }
    }

    #[test]
    fn test_format_balance() {
        assert_eq!(format_balance(100), "Balance: 100 coins");
        assert_eq!(format_balance(0), "Balance: 0 coins");
    }

    #[test]
    fn test_format_purchase_result() {
        assert_eq!(
            format_purchase_result(true, "health_pack", Some(1), None),
            "Purchased health_pack"
        );
        assert_eq!(
            format_purchase_result(true, "arrows", Some(10), None),
            "Purchased 10x arrows"
        );
        assert_eq!(
            format_purchase_result(false, "sword", None, Some("Not enough coins")),
            "Purchase failed: Not enough coins"
        );
    }

    #[test]
    fn test_format_shop_offer() {
        assert_eq!(
            format_shop_offer("health_pack", 1, 20),
            "  health_pack - 20 coins"
        );
        assert_eq!(
            format_shop_offer("arrows", 10, 15),
            "  arrows (x10) - 15 coins"
        );
    }

    // Server browser command tests
    #[test]
    fn test_parse_servers_command() {
        match parse_command("/servers") {
            CommandResult::RefreshServers => {}
            _ => panic!("Expected RefreshServers"),
        }

        match parse_command("/SERVERS") {
            CommandResult::RefreshServers => {}
            _ => panic!("Expected RefreshServers for uppercase"),
        }
    }

    #[test]
    fn test_parse_connect_command() {
        match parse_command("/connect 1") {
            CommandResult::ConnectToServer(1) => {}
            _ => panic!("Expected ConnectToServer(1)"),
        }

        match parse_command("/connect 5") {
            CommandResult::ConnectToServer(5) => {}
            _ => panic!("Expected ConnectToServer(5)"),
        }

        match parse_command("/CONNECT 3") {
            CommandResult::ConnectToServer(3) => {}
            _ => panic!("Expected ConnectToServer(3) for uppercase"),
        }
    }

    #[test]
    fn test_parse_connect_invalid_index() {
        match parse_command("/connect 0") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("1 or higher"));
            }
            _ => panic!("Expected InvalidSyntax for index 0"),
        }

        match parse_command("/connect abc") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for non-numeric"),
        }

        match parse_command("/connect") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing arg"),
        }
    }

    #[test]
    fn test_help_includes_server_commands() {
        match parse_command("/help") {
            CommandResult::ClientOnly(text) => {
                assert!(text.contains("/servers"));
                assert!(text.contains("/connect"));
            }
            _ => panic!("Expected ClientOnly help text"),
        }
    }

    // T019: Quick join command parsing tests
    #[test]
    fn test_parse_quickjoin_no_args() {
        match parse_command("/quickjoin") {
            CommandResult::QuickJoin { mode, region } => {
                assert_eq!(mode, "any");
                assert_eq!(region, "any");
            }
            _ => panic!("Expected QuickJoin"),
        }
    }

    #[test]
    fn test_parse_quickjoin_mode_only() {
        match parse_command("/quickjoin tdm") {
            CommandResult::QuickJoin { mode, region } => {
                assert_eq!(mode, "tdm");
                assert_eq!(region, "any");
            }
            _ => panic!("Expected QuickJoin with mode"),
        }
    }

    #[test]
    fn test_parse_quickjoin_mode_and_region() {
        match parse_command("/quickjoin ctf eu") {
            CommandResult::QuickJoin { mode, region } => {
                assert_eq!(mode, "ctf");
                assert_eq!(region, "eu");
            }
            _ => panic!("Expected QuickJoin with mode and region"),
        }
    }

    #[test]
    fn test_parse_quickjoin_case_insensitive() {
        match parse_command("/QUICKJOIN TDM US") {
            CommandResult::QuickJoin { mode, region } => {
                assert_eq!(mode, "tdm");
                assert_eq!(region, "us");
            }
            _ => panic!("Expected QuickJoin for uppercase"),
        }
    }

    #[test]
    fn test_parse_play_alias() {
        match parse_command("/play ffa asia") {
            CommandResult::QuickJoin { mode, region } => {
                assert_eq!(mode, "ffa");
                assert_eq!(region, "asia");
            }
            _ => panic!("Expected QuickJoin via /play alias"),
        }
    }

    #[test]
    fn test_parse_quickjoin_invalid_mode() {
        match parse_command("/quickjoin invalid_mode") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Invalid mode"));
                assert!(msg.contains("Valid modes"));
            }
            _ => panic!("Expected InvalidSyntax for invalid mode"),
        }
    }

    #[test]
    fn test_parse_quickjoin_invalid_region() {
        match parse_command("/quickjoin tdm invalid_region") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Invalid region"));
                assert!(msg.contains("Valid regions"));
            }
            _ => panic!("Expected InvalidSyntax for invalid region"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_view() {
        match parse_command("/quickjoin-prefs") {
            CommandResult::QuickJoinPrefs { action, value } => {
                assert_eq!(action, "view");
                assert!(value.is_none());
            }
            _ => panic!("Expected QuickJoinPrefs view"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_mode() {
        match parse_command("/quickjoin-prefs mode ctf") {
            CommandResult::QuickJoinPrefs { action, value } => {
                assert_eq!(action, "mode");
                assert_eq!(value, Some("ctf".to_string()));
            }
            _ => panic!("Expected QuickJoinPrefs mode"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_region() {
        match parse_command("/quickjoin-prefs region eu") {
            CommandResult::QuickJoinPrefs { action, value } => {
                assert_eq!(action, "region");
                assert_eq!(value, Some("eu".to_string()));
            }
            _ => panic!("Expected QuickJoinPrefs region"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_invalid_action() {
        match parse_command("/quickjoin-prefs invalid") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Unknown preference"));
            }
            _ => panic!("Expected InvalidSyntax for invalid action"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_mode_missing_value() {
        match parse_command("/quickjoin-prefs mode") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing mode value"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_region_missing_value() {
        match parse_command("/quickjoin-prefs region") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing region value"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_invalid_mode() {
        match parse_command("/quickjoin-prefs mode invalid") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Invalid mode"));
            }
            _ => panic!("Expected InvalidSyntax for invalid mode"),
        }
    }

    #[test]
    fn test_parse_quickjoin_prefs_invalid_region() {
        match parse_command("/quickjoin-prefs region invalid") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Invalid region"));
            }
            _ => panic!("Expected InvalidSyntax for invalid region"),
        }
    }

    #[test]
    fn test_help_includes_quickjoin_commands() {
        match parse_command("/help") {
            CommandResult::ClientOnly(text) => {
                assert!(text.contains("/quickjoin"));
                assert!(text.contains("/play"));
                assert!(text.contains("/quickjoin-prefs"));
            }
            _ => panic!("Expected ClientOnly help text"),
        }
    }

    // === Feature 042: Accessibility command tests ===

    #[test]
    fn test_parse_rebind_action() {
        match parse_command("/rebind forward up") {
            CommandResult::RebindAction { action, key } => {
                assert_eq!(action, Action::Forward);
                assert_eq!(key, Key::Up);
            }
            _ => panic!("Expected RebindAction"),
        }
    }

    #[test]
    fn test_parse_rebind_with_mouse() {
        match parse_command("/rebind attack leftclick") {
            CommandResult::RebindAction { action, key } => {
                assert_eq!(action, Action::Attack);
                assert_eq!(key, Key::LeftClick);
            }
            _ => panic!("Expected RebindAction with mouse"),
        }
    }

    #[test]
    fn test_parse_rebind_list() {
        match parse_command("/rebind list") {
            CommandResult::RebindList => {}
            _ => panic!("Expected RebindList"),
        }
    }

    #[test]
    fn test_parse_rebind_reset() {
        match parse_command("/rebind reset") {
            CommandResult::RebindReset => {}
            _ => panic!("Expected RebindReset"),
        }
    }

    #[test]
    fn test_parse_rebind_invalid_action() {
        match parse_command("/rebind invalid_action w") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Unknown action"));
            }
            _ => panic!("Expected InvalidSyntax for invalid action"),
        }
    }

    #[test]
    fn test_parse_rebind_invalid_key() {
        match parse_command("/rebind forward invalidkey") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Unknown key"));
            }
            _ => panic!("Expected InvalidSyntax for invalid key"),
        }
    }

    #[test]
    fn test_parse_rebind_missing_args() {
        match parse_command("/rebind") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing args"),
        }

        match parse_command("/rebind forward") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing key"),
        }
    }

    #[test]
    fn test_parse_ui_scale() {
        match parse_command("/ui_scale 100") {
            CommandResult::SetUiScale(100) => {}
            _ => panic!("Expected SetUiScale(100)"),
        }

        match parse_command("/ui_scale 75") {
            CommandResult::SetUiScale(75) => {}
            _ => panic!("Expected SetUiScale(75)"),
        }

        match parse_command("/ui_scale 150") {
            CommandResult::SetUiScale(150) => {}
            _ => panic!("Expected SetUiScale(150)"),
        }
    }

    #[test]
    fn test_parse_ui_scale_out_of_range() {
        match parse_command("/ui_scale 50") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("between"));
            }
            _ => panic!("Expected InvalidSyntax for value too low"),
        }

        match parse_command("/ui_scale 200") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("between"));
            }
            _ => panic!("Expected InvalidSyntax for value too high"),
        }
    }

    #[test]
    fn test_parse_ui_scale_missing_arg() {
        match parse_command("/ui_scale") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing arg"),
        }
    }

    #[test]
    fn test_parse_colorblind() {
        match parse_command("/colorblind none") {
            CommandResult::SetColorblind(ColorblindPreset::None) => {}
            _ => panic!("Expected SetColorblind(None)"),
        }

        match parse_command("/colorblind protanopia") {
            CommandResult::SetColorblind(ColorblindPreset::Protanopia) => {}
            _ => panic!("Expected SetColorblind(Protanopia)"),
        }

        match parse_command("/colorblind deuteranopia") {
            CommandResult::SetColorblind(ColorblindPreset::Deuteranopia) => {}
            _ => panic!("Expected SetColorblind(Deuteranopia)"),
        }

        match parse_command("/colorblind tritanopia") {
            CommandResult::SetColorblind(ColorblindPreset::Tritanopia) => {}
            _ => panic!("Expected SetColorblind(Tritanopia)"),
        }
    }

    #[test]
    fn test_parse_colorblind_case_insensitive() {
        match parse_command("/COLORBLIND PROTANOPIA") {
            CommandResult::SetColorblind(ColorblindPreset::Protanopia) => {}
            _ => panic!("Expected SetColorblind for uppercase"),
        }
    }

    #[test]
    fn test_parse_colorblind_invalid() {
        match parse_command("/colorblind invalid") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Invalid preset"));
            }
            _ => panic!("Expected InvalidSyntax for invalid preset"),
        }
    }

    #[test]
    fn test_parse_highcontrast() {
        match parse_command("/highcontrast on") {
            CommandResult::SetHighContrast(true) => {}
            _ => panic!("Expected SetHighContrast(true)"),
        }

        match parse_command("/highcontrast off") {
            CommandResult::SetHighContrast(false) => {}
            _ => panic!("Expected SetHighContrast(false)"),
        }

        match parse_command("/highcontrast true") {
            CommandResult::SetHighContrast(true) => {}
            _ => panic!("Expected SetHighContrast(true) for 'true'"),
        }

        match parse_command("/highcontrast false") {
            CommandResult::SetHighContrast(false) => {}
            _ => panic!("Expected SetHighContrast(false) for 'false'"),
        }
    }

    #[test]
    fn test_parse_highcontrast_invalid() {
        match parse_command("/highcontrast maybe") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for invalid value"),
        }
    }

    #[test]
    fn test_parse_subtitles() {
        match parse_command("/subtitles on") {
            CommandResult::SetSubtitles(true) => {}
            _ => panic!("Expected SetSubtitles(true)"),
        }

        match parse_command("/subtitles off") {
            CommandResult::SetSubtitles(false) => {}
            _ => panic!("Expected SetSubtitles(false)"),
        }
    }

    #[test]
    fn test_parse_subtitles_invalid() {
        match parse_command("/subtitles maybe") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for invalid value"),
        }
    }

    #[test]
    fn test_help_includes_accessibility_commands() {
        match parse_command("/help") {
            CommandResult::ClientOnly(text) => {
                assert!(text.contains("/rebind"));
                assert!(text.contains("/ui_scale"));
                assert!(text.contains("/colorblind"));
                assert!(text.contains("/highcontrast"));
                assert!(text.contains("/subtitles"));
            }
            _ => panic!("Expected ClientOnly help text"),
        }
    }

    // === Feature 043: Quest command tests ===

    #[test]
    fn test_parse_quest_list() {
        match parse_command("/quest list") {
            CommandResult::QuestList => {}
            _ => panic!("Expected QuestList"),
        }

        match parse_command("/quest ls") {
            CommandResult::QuestList => {}
            _ => panic!("Expected QuestList for ls alias"),
        }
    }

    #[test]
    fn test_parse_quest_info() {
        match parse_command("/quest info test_quest") {
            CommandResult::QuestInfo { quest_id } => {
                assert_eq!(quest_id, "test_quest");
            }
            _ => panic!("Expected QuestInfo"),
        }

        match parse_command("/quest show another_quest") {
            CommandResult::QuestInfo { quest_id } => {
                assert_eq!(quest_id, "another_quest");
            }
            _ => panic!("Expected QuestInfo for show alias"),
        }
    }

    #[test]
    fn test_parse_quest_info_missing_id() {
        match parse_command("/quest info") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing quest_id"),
        }
    }

    #[test]
    fn test_parse_quest_accept() {
        match parse_command("/quest accept main_story_1") {
            CommandResult::QuestAccept { quest_id } => {
                assert_eq!(quest_id, "main_story_1");
            }
            _ => panic!("Expected QuestAccept"),
        }

        match parse_command("/quest start side_quest_2") {
            CommandResult::QuestAccept { quest_id } => {
                assert_eq!(quest_id, "side_quest_2");
            }
            _ => panic!("Expected QuestAccept for start alias"),
        }
    }

    #[test]
    fn test_parse_quest_abandon() {
        match parse_command("/quest abandon old_quest") {
            CommandResult::QuestAbandon { quest_id } => {
                assert_eq!(quest_id, "old_quest");
            }
            _ => panic!("Expected QuestAbandon"),
        }

        match parse_command("/quest drop failed_quest") {
            CommandResult::QuestAbandon { quest_id } => {
                assert_eq!(quest_id, "failed_quest");
            }
            _ => panic!("Expected QuestAbandon for drop alias"),
        }
    }

    #[test]
    fn test_parse_quest_pin() {
        match parse_command("/quest pin priority_quest") {
            CommandResult::QuestPin { quest_id } => {
                assert_eq!(quest_id, "priority_quest");
            }
            _ => panic!("Expected QuestPin"),
        }

        match parse_command("/quest track tracked_quest") {
            CommandResult::QuestPin { quest_id } => {
                assert_eq!(quest_id, "tracked_quest");
            }
            _ => panic!("Expected QuestPin for track alias"),
        }
    }

    #[test]
    fn test_parse_quest_unpin() {
        match parse_command("/quest unpin") {
            CommandResult::QuestUnpin => {}
            _ => panic!("Expected QuestUnpin"),
        }

        match parse_command("/quest untrack") {
            CommandResult::QuestUnpin => {}
            _ => panic!("Expected QuestUnpin for untrack alias"),
        }
    }

    #[test]
    fn test_parse_quest_invalid_subcommand() {
        match parse_command("/quest invalid") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Unknown quest subcommand"));
            }
            _ => panic!("Expected InvalidSyntax for invalid subcommand"),
        }
    }

    #[test]
    fn test_parse_quest_missing_subcommand() {
        match parse_command("/quest") {
            CommandResult::InvalidSyntax(msg) => {
                assert!(msg.contains("Usage"));
            }
            _ => panic!("Expected InvalidSyntax for missing subcommand"),
        }
    }

    #[test]
    fn test_help_includes_quest_commands() {
        match parse_command("/help") {
            CommandResult::ClientOnly(text) => {
                assert!(text.contains("/quest list"));
                assert!(text.contains("/quest info"));
                assert!(text.contains("/quest accept"));
                assert!(text.contains("/quest abandon"));
                assert!(text.contains("/quest pin"));
                assert!(text.contains("/quest unpin"));
            }
            _ => panic!("Expected ClientOnly help text"),
        }
    }
}
