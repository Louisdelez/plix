//! Protocol message definitions

use serde::{Deserialize, Serialize};

use crate::identity::SessionId;
use crate::math::{Rotation, Vec3};
use crate::time::Tick;
use crate::types::{
    BlockPos, BlockType, BotId, GameMode, InputSeq, ItemId, LootEntityId, PlayerId, ProjectileId,
    TeamId,
};

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
        /// Account ID placeholder (v2 - always None in v1)
        #[serde(default)]
        account_id: Option<u64>,
        /// Auth token placeholder (v2 - always None in v1)
        #[serde(default)]
        auth_token: Option<String>,
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

    /// Request to reset training session (training mode only)
    TrainingReset,

    /// Request current training statistics (training mode only)
    TrainingStatsRequest,

    /// Select a hotbar slot (0-8 for 9 slots)
    SelectHotbarSlot {
        /// Slot index to select
        slot: u8,
    },

    /// Use the item in the active hotbar slot
    UseActiveItem,

    /// Request to craft an item using a recipe
    CraftRequest {
        /// Recipe identifier (e.g., "health_pack", "sword", "bow")
        recipe_id: String,
    },

    // ========================================================================
    // Economy Messages
    // ========================================================================
    /// Request to purchase an item from the shop
    BuyRequest {
        /// Offer identifier (e.g., "health_pack", "sword")
        offer_id: String,
    },

    /// Request current coin balance
    BalanceRequest,

    /// Request list of available shop offers (optional v1)
    ShopListRequest,

    // ========================================================================
    // Identity Messages (Feature 025)
    // ========================================================================
    /// Request to change display name (T043)
    RenameRequest {
        /// New display name (will be validated server-side)
        new_name: String,
    },
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

/// Reason for rejecting a rename request (T044)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenameRejectReason {
    /// Rate limited (60 second cooldown)
    RateLimited,
    /// Name validation failed (too long, invalid chars, etc.)
    InvalidName,
    /// Name is unavailable (max suffix reached for this name)
    NameUnavailable,
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
        /// Server-assigned display name (may differ from requested if sanitized/disambiguated)
        #[serde(default)]
        display_name: String,
        /// Session ID for this connection (logging and correlation)
        #[serde(default)]
        session_id: SessionId,
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

    /// Training statistics response (training mode only)
    TrainingStats {
        /// Total hits on bots
        hits: u32,
        /// Total bot kills
        kills: u32,
        /// Total attack attempts
        attacks: u32,
        /// Accuracy percentage (0.0 - 100.0)
        accuracy_pct: f32,
        /// Session duration in seconds
        session_duration_secs: f32,
    },

    /// Inventory update for a player
    InventoryUpdate(InventorySnapshot),
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
    /// Bot snapshots (training mode only, empty otherwise)
    #[serde(default)]
    pub bots: Vec<BotSnapshot>,
}

/// Training bot state snapshot for replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSnapshot {
    /// Bot identifier
    pub id: BotId,
    /// Current world position
    pub position: Vec3,
    /// Current health (0-100)
    pub health: u8,
    /// Is bot dead (waiting to respawn)
    pub is_dead: bool,
}

// ============================================================================
// Inventory Snapshots
// ============================================================================

/// Snapshot of a single hotbar slot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotSnapshot {
    /// Item ID (NONE if empty)
    pub item_id: ItemId,
    /// Quantity (0 if empty)
    pub quantity: u8,
}

impl SlotSnapshot {
    /// Create an empty slot snapshot.
    pub fn empty() -> Self {
        Self {
            item_id: ItemId::NONE,
            quantity: 0,
        }
    }

    /// Create a slot snapshot from item ID and quantity.
    pub fn new(item_id: ItemId, quantity: u8) -> Self {
        Self { item_id, quantity }
    }

    /// Check if this slot is empty.
    pub fn is_empty(&self) -> bool {
        !self.item_id.is_valid() || self.quantity == 0
    }
}

impl Default for SlotSnapshot {
    fn default() -> Self {
        Self::empty()
    }
}

/// Complete inventory snapshot for replication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventorySnapshot {
    /// All hotbar slots
    pub slots: Vec<SlotSnapshot>,
    /// Currently active slot index
    pub active_slot: u8,
}

impl InventorySnapshot {
    /// Create an empty inventory snapshot with the given number of slots.
    pub fn empty(slot_count: usize) -> Self {
        Self {
            slots: vec![SlotSnapshot::empty(); slot_count],
            active_slot: 0,
        }
    }
}

impl Default for InventorySnapshot {
    fn default() -> Self {
        Self::empty(9) // Default 9 slots
    }
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
    /// Who this player is spectating while dead (killer ID for TDM)
    #[serde(default)]
    pub spectate_target: Option<PlayerId>,
    /// Player's display name
    #[serde(default)]
    pub display_name: String,
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
    /// Team scores (used for TDM mode)
    pub scores: Vec<TeamScore>,
    /// Winning team for TDM mode (only set in EndScreen phase, None for tie)
    #[serde(default)]
    pub team_winner: Option<TeamId>,
    /// Game mode for this match (FFA or TDM)
    #[serde(default)]
    pub game_mode: GameMode,
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
            team_winner: None,
            game_mode: GameMode::Tdm,
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

    // ========================================================================
    // BR Lite (Battle Royale) Events
    // ========================================================================
    /// BR zone state update (sent on phase change + every 5 seconds)
    BrZoneUpdate {
        /// Zone center (XZ coordinates)
        center: [f32; 2],
        /// Current zone radius
        current_radius: f32,
        /// Target radius for current phase
        target_radius: f32,
        /// Current phase index (0-indexed)
        phase_index: u8,
        /// Phase mode: 0 = Stable, 1 = Shrinking
        phase_mode: u8,
        /// Time remaining in current phase (seconds)
        phase_time_remaining_secs: u16,
        /// Damage per second outside zone
        damage_per_tick: u16,
    },

    /// BR loot spawn (sent at match start)
    BrLootSpawn {
        /// Unique loot identifier
        loot_id: u16,
        /// World position
        position: [f32; 3],
        /// Loot type: 0 = HealthPack, 1 = SpeedBoost
        loot_type: u8,
        /// Type-specific parameter (heal amount or speed multiplier * 100)
        param: u16,
    },

    /// BR loot collected
    BrLootPickup {
        /// Loot identifier (matches BrLootSpawn.loot_id)
        loot_id: u16,
        /// Player who collected the loot
        player_id: PlayerId,
    },

    /// BR player eliminated (permanent death)
    BrElimination {
        /// Eliminated player
        player_id: PlayerId,
        /// Remaining alive players
        alive_count: u8,
    },

    /// BR match victory
    BrVictory {
        /// Winning player
        winner_id: PlayerId,
    },

    // ========================================================================
    // Training Mode Events
    // ========================================================================
    /// Training session was reset
    TrainingReset {
        /// Player who triggered the reset
        player_id: PlayerId,
    },

    /// Bot was hit by player attack (sent to attacker)
    BotHit {
        /// Bot that was hit
        bot_id: BotId,
        /// Damage dealt
        damage: u8,
        /// Whether the bot was killed
        killed: bool,
    },

    /// Bot respawned after elimination (broadcast to all)
    BotRespawned {
        /// Bot that respawned
        bot_id: BotId,
        /// New position
        position: Vec3,
    },

    // ========================================================================
    // Inventory Events
    // ========================================================================
    /// Loot entity spawned in the world (broadcast to all)
    LootSpawned {
        /// Loot entity ID
        loot_id: LootEntityId,
        /// World position
        position: Vec3,
        /// Item in the loot
        item_id: ItemId,
        /// Quantity
        quantity: u8,
    },

    /// Loot entity removed from the world (broadcast to all)
    LootRemoved {
        /// Loot entity ID
        loot_id: LootEntityId,
    },

    /// Player picked up loot (broadcast to all)
    LootPickedUp {
        /// Loot entity ID
        loot_id: LootEntityId,
        /// Player who picked it up
        player_id: PlayerId,
        /// Item picked up
        item_id: ItemId,
        /// Quantity picked up
        quantity: u8,
    },

    /// Player used an item (broadcast to all for effects)
    ItemUsed {
        /// Player who used the item
        player_id: PlayerId,
        /// Item that was used
        item_id: ItemId,
        /// Slot it was used from
        slot: u8,
    },

    // ========================================================================
    // Weapons & Projectile Events
    // ========================================================================
    /// Projectile spawned (broadcast to all clients)
    ProjectileSpawn {
        /// Unique projectile identifier
        id: ProjectileId,
        /// Player who fired it
        owner: PlayerId,
        /// Initial world position
        position: Vec3,
        /// Velocity vector (blocks per tick)
        velocity: Vec3,
        /// Server tick when spawned (for client interpolation)
        spawn_tick: Tick,
    },

    /// Projectile hit something (broadcast to all clients)
    ProjectileImpact {
        /// Projectile identifier
        id: ProjectileId,
        /// Impact world position
        position: Vec3,
        /// What was hit
        impact_type: ProjectileImpactType,
        /// Target player if applicable
        target: Option<PlayerId>,
    },

    /// Projectile removed without impact (broadcast to all clients)
    ProjectileDespawn {
        /// Projectile identifier
        id: ProjectileId,
        /// Reason for despawn
        reason: ProjectileDespawnReason,
    },

    /// Weapon attack rejected due to cooldown (sent to attacker only)
    WeaponCooldown {
        /// Item that was on cooldown
        item_id: ItemId,
        /// Ticks remaining until ready
        remaining_ticks: u32,
    },

    /// Projectile spawn rejected due to limit (sent to shooter only)
    ProjectileLimitReached {
        /// Current projectile count
        current_count: u8,
        /// Maximum allowed
        max_count: u8,
    },

    // ========================================================================
    // Crafting Events
    // ========================================================================
    /// Craft operation result (sent to crafter only)
    CraftResult {
        /// Whether the craft succeeded
        success: bool,
        /// Recipe that was attempted
        recipe_id: String,
        /// Output item if successful
        output_item: Option<ItemId>,
        /// Output quantity if successful
        output_quantity: Option<u8>,
        /// Failure reason if unsuccessful
        fail_reason: Option<CraftRejectReason>,
    },

    // ========================================================================
    // Economy Events
    // ========================================================================
    /// Player's coin balance changed (sent to player on earn/spend/request)
    BalanceUpdate {
        /// Current coin balance after change
        balance: u32,
    },

    /// Purchase operation result (sent to buyer only)
    PurchaseResult {
        /// Whether purchase succeeded
        success: bool,
        /// Offer ID that was attempted
        offer_id: String,
        /// Item received (if successful)
        output_item: Option<ItemId>,
        /// Quantity received (if successful)
        output_quantity: Option<u8>,
        /// Failure reason (if unsuccessful)
        fail_reason: Option<PurchaseRejectReason>,
    },

    /// List of available shop offers (response to ShopListRequest)
    ShopList {
        /// Available offers in current mode
        offers: Vec<ShopOfferInfo>,
    },

    // ========================================================================
    // Identity Events (Feature 025)
    // ========================================================================
    /// Rename request result (sent to requesting player only) [T045]
    RenameResult {
        /// Whether the rename succeeded
        success: bool,
        /// New display name if successful
        new_name: Option<String>,
        /// Rejection reason if unsuccessful
        reason: Option<RenameRejectReason>,
    },

    /// Player changed their display name (broadcast to all) [T046]
    PlayerRenamed {
        /// Player who changed name
        player_id: PlayerId,
        /// Previous display name
        old_name: String,
        /// New display name
        new_name: String,
    },
}

// ============================================================================
// Projectile Types
// ============================================================================

/// What a projectile hit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileImpactType {
    /// Hit a player
    Player,
    /// Hit a training bot
    Bot,
    /// Hit a solid block
    Block,
}

/// Why a projectile was despawned without impact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectileDespawnReason {
    /// Lifetime expired
    Timeout,
    /// Server projectile limit exceeded
    LimitPurge,
    /// Owner disconnected
    OwnerLeft,
}

// ============================================================================
// Crafting Types
// ============================================================================

/// Reason for rejecting a craft request (protocol-level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CraftRejectReason {
    /// Recipe ID not found in registry
    UnknownRecipe,
    /// Crafting is disabled for the current game mode
    CraftingDisabled,
    /// Recipe is not allowed in the current game mode
    RecipeDisabled,
    /// Player is on cooldown from a recent successful craft
    CooldownActive,
    /// Player doesn't have all required ingredients
    MissingIngredients,
    /// No space in hotbar for the output item
    HotbarFull,
    /// Player is dead and cannot craft
    PlayerDead,
}

impl std::fmt::Display for CraftRejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CraftRejectReason::UnknownRecipe => write!(f, "Unknown recipe"),
            CraftRejectReason::CraftingDisabled => write!(f, "Crafting not available"),
            CraftRejectReason::RecipeDisabled => write!(f, "Recipe not available"),
            CraftRejectReason::CooldownActive => write!(f, "Cooldown active"),
            CraftRejectReason::MissingIngredients => write!(f, "Not enough materials"),
            CraftRejectReason::HotbarFull => write!(f, "Inventory full"),
            CraftRejectReason::PlayerDead => write!(f, "Cannot craft while dead"),
        }
    }
}

// ============================================================================
// Economy Types
// ============================================================================

/// Minimal offer info for client display (used in ShopList event).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferInfo {
    /// Offer identifier
    pub offer_id: String,
    /// Item that will be delivered
    pub item_id: ItemId,
    /// Quantity per purchase
    pub quantity: u8,
    /// Price in coins
    pub price: u32,
}

// Re-export PurchaseRejectReason from economy module
pub use crate::economy::PurchaseRejectReason;
