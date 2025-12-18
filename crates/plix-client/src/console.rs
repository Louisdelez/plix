//! Console command system for client-side slash commands.
//!
//! Supports economy commands: /balance, /buy, /shop
//! Supports identity commands: /name
//! Supports server browser commands: /servers, /connect
//! Supports matchmaking commands: /quickjoin, /play, /quickjoin-prefs

use plix_common::protocol::ClientMessage;

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
/help           - Show this help message"#;
            CommandResult::ClientOnly(help_text.to_string())
        }

        _ => CommandResult::UnknownCommand(format!("Unknown command: /{}", command)),
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
}
