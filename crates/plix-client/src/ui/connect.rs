//! Connect screen UI

/// Connect screen state
#[derive(Debug)]
pub struct ConnectScreen {
    /// Server address input
    pub server_address: String,
    /// Player name input
    pub player_name: String,
    /// Error message to display
    pub error: Option<String>,
    /// Whether currently connecting
    pub connecting: bool,
}

impl ConnectScreen {
    /// Create a new connect screen
    pub fn new() -> Self {
        Self {
            server_address: "127.0.0.1:7777".to_string(),
            player_name: "Player".to_string(),
            error: None,
            connecting: false,
        }
    }

    /// Validate inputs
    pub fn validate(&self) -> Result<(), String> {
        if self.server_address.is_empty() {
            return Err("Server address is required".to_string());
        }

        if self.player_name.is_empty() {
            return Err("Player name is required".to_string());
        }

        if self.player_name.len() > 32 {
            return Err("Player name must be 32 characters or less".to_string());
        }

        // Try to parse address
        if self.server_address.parse::<std::net::SocketAddr>().is_err() {
            return Err("Invalid server address format".to_string());
        }

        Ok(())
    }

    /// Set error message
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.connecting = false;
    }

    /// Clear error
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Start connecting
    pub fn start_connecting(&mut self) {
        self.connecting = true;
        self.error = None;
    }

    /// Stop connecting
    pub fn stop_connecting(&mut self) {
        self.connecting = false;
    }
}

impl Default for ConnectScreen {
    fn default() -> Self {
        Self::new()
    }
}
