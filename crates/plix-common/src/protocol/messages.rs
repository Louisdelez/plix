//! Protocol message definitions

use serde::{Deserialize, Serialize};

use crate::math::{Rotation, Vec3};
use crate::time::Tick;
use crate::types::{BlockPos, BlockType, InputSeq, PlayerId, TeamId};

// ============================================================================
// Client → Server Messages
// ============================================================================

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Connection handshake
    Connect {
        /// Protocol version
        protocol_version: u8,
        /// Player name (max 32 chars)
        name: String,
    },

    /// Graceful disconnect
    Disconnect,

    /// Player input for a tick
    Input(PlayerInput),

    /// Acknowledge receipt of snapshot
    SnapshotAck {
        /// Tick of acknowledged snapshot
        tick: Tick,
    },

    /// Block edit request (place/remove)
    BlockEdit(BlockEditRequest),

    /// Toggle ready state for match start (only valid in Lobby phase)
    ReadyToggle,
}

// ============================================================================
// Block Edit Types
// ============================================================================

/// Type of block edit operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditKind {
    /// Place a block
    Place,
    /// Remove a block
    Remove,
}

/// Reason for rejecting a block edit request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockEditRejectReason {
    /// Target position is outside world bounds
    OutOfBounds,
    /// Target is too far from player (> 5 blocks)
    OutOfRange,
    /// Cannot place - cell is already occupied
    CellNotEmpty,
    /// Cannot remove - cell is empty (air)
    CellEmpty,
    /// Would trap a player inside the block
    PlayerCollision,
    /// Edit cooldown not expired (rate limited)
    RateLimited,
    /// Player is dead
    PlayerDead,
    /// Not in Playing match phase
    InvalidPhase,
}

/// Client request to edit a block in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRequest {
    /// Type of edit (place or remove)
    pub kind: BlockEditKind,
    /// Target block position
    pub target_pos: BlockPos,
    /// Block type to place (ignored for remove)
    pub block_type: BlockType,
}

/// Server event confirming a successful block edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditApplied {
    /// Block position that was edited
    pub pos: BlockPos,
    /// New block type (Air for remove, block type for place)
    pub new_block: BlockType,
    /// Server tick when edit was applied
    pub tick: Tick,
}

/// Server event indicating a rejected block edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEditRejected {
    /// Reason for rejection
    pub reason: BlockEditRejectReason,
    /// Requested position (echoed back for client correlation)
    pub pos: BlockPos,
}

/// Player input for a single tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Input sequence number
    pub seq: InputSeq,
    /// Client's estimated server tick
    pub tick: Tick,
    /// Forward movement (-1.0 to 1.0)
    pub move_forward: f32,
    /// Right movement (-1.0 to 1.0)
    pub move_right: f32,
    /// Jump pressed
    pub jump: bool,
    /// Crouch pressed
    pub crouch: bool,
    /// Attack pressed
    pub attack: bool,
    /// Look yaw (radians)
    pub yaw: f32,
    /// Look pitch (radians)
    pub pitch: f32,
    /// RTT measurement nonce (client timestamp in microseconds, echoed by server)
    #[serde(default)]
    pub rtt_nonce: u64,
}

impl PlayerInput {
    /// Create an empty input (no movement, no actions)
    pub fn empty(seq: InputSeq, tick: Tick) -> Self {
        Self {
            seq,
            tick,
            move_forward: 0.0,
            move_right: 0.0,
            jump: false,
            crouch: false,
            attack: false,
            yaw: 0.0,
            pitch: 0.0,
            rtt_nonce: 0,
        }
    }
}

// ============================================================================
// Server → Client Messages
// ============================================================================

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Connection accepted
    Connected {
        /// Assigned player ID
        player_id: PlayerId,
        /// Current server tick
        tick: Tick,
        /// Server tick rate
        tick_rate: u8,
        /// Compressed arena data
        arena_data: Vec<u8>,
    },

    /// Connection rejected
    Rejected {
        /// Reason for rejection
        reason: String,
    },

    /// Kicked from server
    Kicked {
        /// Reason for kick
        reason: String,
    },

    /// Anti-cheat warning (player has accumulated infractions)
    Warning {
        /// Warning message
        message: String,
        /// Current strike count
        strikes: u32,
    },

    /// World state snapshot
    Snapshot(WorldSnapshot),

    /// Game event
    Event(GameEvent),
}

/// Complete world snapshot sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Current server tick
    pub tick: Tick,
    /// Last processed input sequence for this client
    pub last_input_seq: InputSeq,
    /// All player states
    pub players: Vec<PlayerSnapshot>,
    /// Current match state
    pub match_state: MatchState,
    /// Echo of client's rtt_nonce for RTT calculation
    #[serde(default)]
    pub rtt_nonce_echo: u64,
}

/// Individual player state in snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    /// Player ID
    pub id: PlayerId,
    /// World position
    pub position: Vec3,
    /// Look rotation
    pub rotation: Rotation,
    /// Health (0-100)
    pub health: u8,
    /// Is player dead
    pub is_dead: bool,
    /// Current animation state
    pub animation: AnimationState,
}

/// Player animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimationState {
    #[default]
    Idle,
    Walking,
    Running,
    Attacking,
    Dead,
}

// ============================================================================
// Match State
// ============================================================================

/// Current match/round state broadcast to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    /// Current phase
    pub phase: MatchPhase,
    /// Countdown remaining (seconds, only valid in Countdown phase)
    pub countdown_remaining: u8,
    /// Match time remaining (seconds, only valid in Playing phase)
    pub time_remaining: u32,
    /// Score limit to win
    pub score_limit: u16,
    /// Per-player scores
    pub player_scores: Vec<PlayerScore>,
    /// Winner player ID (only set in EndScreen phase, None for tie)
    pub winner: Option<PlayerId>,
    /// Current arena name
    pub arena_name: String,
    /// Team scores (kept for backwards compatibility)
    pub scores: Vec<TeamScore>,
}

impl Default for MatchState {
    fn default() -> Self {
        Self {
            phase: MatchPhase::Lobby,
            countdown_remaining: 3,
            time_remaining: 300, // 5 minutes
            score_limit: 5,
            player_scores: Vec::new(),
            winner: None,
            arena_name: String::new(),
            scores: vec![
                TeamScore {
                    team: TeamId::TEAM_0,
                    score: 0,
                },
                TeamScore {
                    team: TeamId::TEAM_1,
                    score: 0,
                },
            ],
        }
    }
}

/// Match phase - server-authoritative state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchPhase {
    /// Waiting for players in lobby, movement allowed, no combat
    Lobby,
    /// Countdown before match start (3 seconds default)
    Countdown,
    /// Match in progress - full gameplay
    Playing,
    /// Match ended - showing final scores
    EndScreen,
    /// Resetting world for next match
    Resetting,
}

impl Default for MatchPhase {
    fn default() -> Self {
        Self::Lobby
    }
}

/// Per-player score for match results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    /// Player ID
    pub player_id: PlayerId,
    /// Player display name
    pub name: String,
    /// Kill count (= score)
    pub kills: u16,
    /// Death count
    pub deaths: u16,
}

/// Why the match ended
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchEndReason {
    /// A player reached the score limit
    ScoreLimit,
    /// Time limit expired
    TimeLimit,
    /// All other players disconnected
    Forfeit,
}

/// Team score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamScore {
    /// Team ID
    pub team: TeamId,
    /// Current score
    pub score: u32,
}

// ============================================================================
// Game Events
// ============================================================================

/// Game events (reliable delivery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    /// Player joined the server
    PlayerJoined {
        id: PlayerId,
        name: String,
        team: TeamId,
    },

    /// Player left the server
    PlayerLeft { id: PlayerId },

    /// Attack hit confirmed (sent to attacker only)
    HitConfirmed {
        /// Attacker player ID
        attacker: PlayerId,
        /// Target player ID
        target: PlayerId,
        /// Damage dealt
        damage: u8,
    },

    /// Damage taken (sent to victim only)
    DamageTaken {
        /// Victim player ID
        victim: PlayerId,
        /// Attacker player ID
        attacker: PlayerId,
        /// Damage amount
        damage: u8,
        /// Remaining health after damage
        new_health: u8,
    },

    /// Player died
    PlayerDied {
        /// Victim player ID
        victim: PlayerId,
        /// Killer player ID (None for environmental death)
        killer: Option<PlayerId>,
    },

    /// Player respawned
    PlayerRespawned { id: PlayerId },

    /// Round started
    RoundStart { round: u16 },

    /// Round ended
    RoundEnd {
        /// Winning team (None for draw)
        winner: Option<TeamId>,
    },

    /// Match ended with final results
    MatchEnd {
        /// Winner player ID (None for tie)
        winner: Option<PlayerId>,
        /// Final scores for all players
        scores: Vec<PlayerScore>,
        /// Reason match ended
        reason: MatchEndReason,
    },

    /// Block edit applied successfully (broadcast to all)
    BlockEditApplied(BlockEditApplied),

    /// Block edit rejected (sent to requester only)
    BlockEditRejected(BlockEditRejected),

    /// Phase transition notification
    MatchPhaseChanged {
        /// Previous phase
        from: MatchPhase,
        /// New phase
        to: MatchPhase,
    },

    /// Countdown tick (broadcast each second during Countdown phase)
    CountdownTick {
        /// Seconds remaining (3, 2, 1, 0)
        remaining: u8,
    },

    /// Player score changed
    ScoreUpdate {
        /// Player whose score changed
        player_id: PlayerId,
        /// New kill count
        kills: u16,
        /// New death count
        deaths: u16,
    },

    /// Arena changed (during Resetting phase)
    ArenaChanged {
        /// New arena name
        name: String,
    },
}
