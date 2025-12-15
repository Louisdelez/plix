//! In-game HUD

use std::collections::VecDeque;
use std::time::Instant;

use plix_common::protocol::MatchState;
use plix_common::types::PlayerId;

/// Combat event for display
#[derive(Debug, Clone)]
pub enum CombatEvent {
    /// Hit confirmed (attacker feedback)
    HitConfirmed { damage: u8 },
    /// Damage taken (victim feedback)
    DamageTaken { damage: u8, new_health: u8 },
    /// Kill feed entry
    Kill { killer: PlayerId, victim: PlayerId },
    /// Player respawned
    Respawned { player: PlayerId },
    /// Block removed successfully (T058)
    BlockRemoved,
    /// Block placed successfully (T059)
    BlockPlaced,
    /// Block edit rejected (T060)
    BlockEditRejected { reason: String },
}

/// Timed event for display
#[derive(Debug, Clone)]
struct TimedEvent {
    event: CombatEvent,
    timestamp: Instant,
}

/// HUD data to display
#[derive(Debug, Default)]
pub struct HudData {
    /// Current FPS
    pub fps: u32,
    /// Ping in milliseconds
    pub ping_ms: u32,
    /// Player health
    pub health: u8,
    /// Player kills
    pub kills: u16,
    /// Player deaths
    pub deaths: u16,
    /// Match state
    pub match_phase: String,
    /// Round time remaining
    pub round_time: u32,
    /// Team scores
    pub scores: Vec<(String, u32)>,
}

/// Maximum events to keep in log
const MAX_EVENTS: usize = 5;

/// Event display duration in seconds
const EVENT_DISPLAY_DURATION: f32 = 3.0;

/// HUD renderer
pub struct Hud {
    /// Whether to show HUD
    pub visible: bool,
    /// Current HUD data
    pub data: HudData,
    /// Combat event log
    events: VecDeque<TimedEvent>,
    /// Show hit confirmed indicator
    pub hit_confirmed: bool,
    /// Hit confirmed timestamp
    hit_confirmed_time: Option<Instant>,
    /// Show damage taken indicator
    pub damage_taken: bool,
    /// Damage taken timestamp
    damage_taken_time: Option<Instant>,
    /// Last damage amount
    pub last_damage: u8,
}

impl Default for Hud {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Hud {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hud")
            .field("visible", &self.visible)
            .field("data", &self.data)
            .field("events_count", &self.events.len())
            .field("hit_confirmed", &self.hit_confirmed)
            .field("damage_taken", &self.damage_taken)
            .finish()
    }
}

impl Hud {
    /// Create a new HUD
    pub fn new() -> Self {
        Self {
            visible: true,
            data: HudData::default(),
            events: VecDeque::with_capacity(MAX_EVENTS),
            hit_confirmed: false,
            hit_confirmed_time: None,
            damage_taken: false,
            damage_taken_time: None,
            last_damage: 0,
        }
    }

    /// Update HUD data
    pub fn update(
        &mut self,
        fps: u32,
        ping_ms: u32,
        health: u8,
        kills: u16,
        deaths: u16,
        match_state: &MatchState,
    ) {
        self.data.fps = fps;
        self.data.ping_ms = ping_ms;
        self.data.health = health;
        self.data.kills = kills;
        self.data.deaths = deaths;

        self.data.match_phase = format!("{:?}", match_state.phase);
        self.data.round_time = match_state.round_time_limit;

        self.data.scores = match_state
            .scores
            .iter()
            .map(|s| (format!("Team {}", s.team.0), s.score))
            .collect();
    }

    /// Toggle HUD visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Push a combat event to the log
    pub fn push_event(&mut self, event: CombatEvent) {
        // Handle hit confirmed indicator
        if matches!(event, CombatEvent::HitConfirmed { .. }) {
            self.hit_confirmed = true;
            self.hit_confirmed_time = Some(Instant::now());
        }

        // Handle damage taken indicator
        if let CombatEvent::DamageTaken { damage, .. } = event {
            self.damage_taken = true;
            self.damage_taken_time = Some(Instant::now());
            self.last_damage = damage;
        }

        // Add to event log
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(TimedEvent {
            event,
            timestamp: Instant::now(),
        });
    }

    /// Update event indicators (call each frame)
    pub fn update_events(&mut self) {
        let now = Instant::now();
        let indicator_duration = std::time::Duration::from_secs_f32(0.3);
        let event_duration = std::time::Duration::from_secs_f32(EVENT_DISPLAY_DURATION);

        // Clear hit confirmed indicator
        if let Some(time) = self.hit_confirmed_time {
            if now.duration_since(time) > indicator_duration {
                self.hit_confirmed = false;
                self.hit_confirmed_time = None;
            }
        }

        // Clear damage taken indicator
        if let Some(time) = self.damage_taken_time {
            if now.duration_since(time) > indicator_duration {
                self.damage_taken = false;
                self.damage_taken_time = None;
            }
        }

        // Prune old events
        while let Some(front) = self.events.front() {
            if now.duration_since(front.timestamp) > event_duration {
                self.events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get recent events for display
    pub fn recent_events(&self) -> Vec<&CombatEvent> {
        self.events.iter().map(|e| &e.event).collect()
    }

    /// Render HUD (placeholder)
    pub fn render(&self) {
        if !self.visible {
            return;
        }
        // TODO: Implement HUD rendering
        // The actual rendering will be done by the render engine using HUD data
    }
}
