//! Economy shared types for protocol messages.

use serde::{Deserialize, Serialize};

/// Reason for rejecting a purchase request (protocol-level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PurchaseRejectReason {
    /// Economy not active for current game mode
    EconomyDisabled,
    /// offer_id not found in shop registry
    UnknownOffer,
    /// Offer not available in current game mode
    ModeRestricted,
    /// Player does not have enough coins
    InsufficientBalance,
    /// No space in hotbar for purchased item
    HotbarFull,
    /// max_per_match limit exceeded for this offer
    PurchaseLimitReached,
    /// Too many purchase requests (rate limited)
    RateLimited,
    /// Player is dead and cannot purchase
    PlayerDead,
}

impl std::fmt::Display for PurchaseRejectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PurchaseRejectReason::EconomyDisabled => write!(f, "Economy not available"),
            PurchaseRejectReason::UnknownOffer => write!(f, "Unknown offer"),
            PurchaseRejectReason::ModeRestricted => write!(f, "Not available in this mode"),
            PurchaseRejectReason::InsufficientBalance => write!(f, "Not enough coins"),
            PurchaseRejectReason::HotbarFull => write!(f, "Inventory full"),
            PurchaseRejectReason::PurchaseLimitReached => write!(f, "Purchase limit reached"),
            PurchaseRejectReason::RateLimited => write!(f, "Too many requests"),
            PurchaseRejectReason::PlayerDead => write!(f, "Cannot purchase while dead"),
        }
    }
}
