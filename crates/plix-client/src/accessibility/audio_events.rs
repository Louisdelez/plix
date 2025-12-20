//! Audio events that can be captioned with subtitles

use serde::{Deserialize, Serialize};

/// Audio events that can be captioned with subtitles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Chat message received
    ChatMessage,

    /// Player joined server
    PlayerJoin,

    /// Player left server
    PlayerLeave,

    /// Local player took damage
    PlayerHit,

    /// Block placed
    BlockPlace,

    /// Block destroyed
    BlockBreak,

    /// Match started
    MatchStart,

    /// Match ended
    MatchEnd,
}

impl AudioEvent {
    /// Get all audio events
    pub fn all() -> &'static [AudioEvent] {
        &[
            AudioEvent::ChatMessage,
            AudioEvent::PlayerJoin,
            AudioEvent::PlayerLeave,
            AudioEvent::PlayerHit,
            AudioEvent::BlockPlace,
            AudioEvent::BlockBreak,
            AudioEvent::MatchStart,
            AudioEvent::MatchEnd,
        ]
    }

    /// Get subtitle text for this event
    pub fn subtitle_text(&self) -> &'static str {
        match self {
            AudioEvent::ChatMessage => "[Chat]",
            AudioEvent::PlayerJoin => "[Player Joined]",
            AudioEvent::PlayerLeave => "[Player Left]",
            AudioEvent::PlayerHit => "[Hit!]",
            AudioEvent::BlockPlace => "[Block Placed]",
            AudioEvent::BlockBreak => "[Block Broken]",
            AudioEvent::MatchStart => "[Match Starting]",
            AudioEvent::MatchEnd => "[Match Ended]",
        }
    }

    /// Get variant name as string (for bridge messages)
    pub fn as_str(&self) -> &'static str {
        match self {
            AudioEvent::ChatMessage => "chat_message",
            AudioEvent::PlayerJoin => "player_join",
            AudioEvent::PlayerLeave => "player_leave",
            AudioEvent::PlayerHit => "player_hit",
            AudioEvent::BlockPlace => "block_place",
            AudioEvent::BlockBreak => "block_break",
            AudioEvent::MatchStart => "match_start",
            AudioEvent::MatchEnd => "match_end",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_event_subtitle_text() {
        assert_eq!(AudioEvent::ChatMessage.subtitle_text(), "[Chat]");
        assert_eq!(AudioEvent::PlayerJoin.subtitle_text(), "[Player Joined]");
        assert_eq!(AudioEvent::PlayerLeave.subtitle_text(), "[Player Left]");
        assert_eq!(AudioEvent::PlayerHit.subtitle_text(), "[Hit!]");
        assert_eq!(AudioEvent::BlockPlace.subtitle_text(), "[Block Placed]");
        assert_eq!(AudioEvent::BlockBreak.subtitle_text(), "[Block Broken]");
        assert_eq!(AudioEvent::MatchStart.subtitle_text(), "[Match Starting]");
        assert_eq!(AudioEvent::MatchEnd.subtitle_text(), "[Match Ended]");
    }

    #[test]
    fn test_audio_event_all() {
        assert_eq!(AudioEvent::all().len(), 8);
    }

    #[test]
    fn test_audio_event_as_str() {
        assert_eq!(AudioEvent::ChatMessage.as_str(), "chat_message");
        assert_eq!(AudioEvent::PlayerJoin.as_str(), "player_join");
    }
}
