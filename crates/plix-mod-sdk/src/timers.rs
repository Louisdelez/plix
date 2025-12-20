//! Timer API - schedule delayed and repeated callbacks
//!
//! Mods can set timeouts (one-shot) and intervals (repeating) to execute
//! code at specified times. Callbacks are identified by a user-defined ID.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::timers::{set_timeout, set_interval, clear_timer};
//!
//! // One-shot timer after 1 second
//! let timer = set_timeout(1000, 1)?; // callback_id = 1
//!
//! // Repeating timer every 250ms
//! let interval = set_interval(250, 2)?; // callback_id = 2
//!
//! // Cancel a timer
//! clear_timer(timer)?;
//! ```
//!
//! # Constraints
//!
//! - Minimum interval: 50ms
//! - Maximum timers per mod: 32

use crate::abi::{self, HostCallOp};
use crate::codec::{
    self, decode, encode, ClearTimerRequest, SetIntervalRequest, SetTimeoutRequest,
    TimerHandleResponse,
};
use crate::error::{err_invalid, Result};
use serde::{Deserialize, Serialize};

/// Minimum timer interval in milliseconds
pub const MIN_TIMER_INTERVAL_MS: u32 = 50;

/// Maximum number of active timers per mod
pub const MAX_TIMERS_PER_MOD: u32 = 32;

/// Handle to an active timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimerHandle {
    /// Timer ID assigned by the runtime
    pub id: u32,
}

impl TimerHandle {
    /// Create a new timer handle
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

/// Set a one-shot timeout
///
/// The callback will be triggered once after the specified delay.
///
/// # Arguments
///
/// * `delay_ms` - Delay in milliseconds (minimum 50ms)
/// * `callback_id` - User-defined ID to identify this callback
///
/// # Returns
///
/// A handle that can be used to cancel the timer.
///
/// # Errors
///
/// - `InvalidArgument` if delay_ms < 50
/// - `RateLimited` if mod has too many active timers (>32)
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::timers::set_timeout;
///
/// // Execute callback 1 after 1 second
/// let timer = set_timeout(1000, 1)?;
///
/// // In your timer callback handler:
/// fn on_timer(callback_id: u32) {
///     match callback_id {
///         1 => handle_delayed_action(),
///         _ => {}
///     }
/// }
/// ```
pub fn set_timeout(delay_ms: u32, callback_id: u32) -> Result<TimerHandle> {
    if delay_ms < MIN_TIMER_INTERVAL_MS {
        return Err(err_invalid(alloc::format!(
            "delay_ms must be >= {} (got {})",
            MIN_TIMER_INTERVAL_MS,
            delay_ms
        )));
    }

    let request = SetTimeoutRequest {
        delay_ms,
        callback_id,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::TimerSetTimeout, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let timer_response: TimerHandleResponse = decode(&result)?;

    Ok(TimerHandle::new(timer_response.timer_id))
}

/// Set a repeating interval
///
/// The callback will be triggered repeatedly at the specified interval.
///
/// # Arguments
///
/// * `interval_ms` - Interval in milliseconds (minimum 50ms)
/// * `callback_id` - User-defined ID to identify this callback
///
/// # Returns
///
/// A handle that can be used to cancel the timer.
///
/// # Errors
///
/// - `InvalidArgument` if interval_ms < 50
/// - `RateLimited` if mod has too many active timers (>32)
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::timers::set_interval;
///
/// // Execute callback 2 every 250ms
/// let timer = set_interval(250, 2)?;
///
/// // In your timer callback handler:
/// fn on_timer(callback_id: u32) {
///     match callback_id {
///         2 => update_periodic_state(),
///         _ => {}
///     }
/// }
/// ```
pub fn set_interval(interval_ms: u32, callback_id: u32) -> Result<TimerHandle> {
    if interval_ms < MIN_TIMER_INTERVAL_MS {
        return Err(err_invalid(alloc::format!(
            "interval_ms must be >= {} (got {})",
            MIN_TIMER_INTERVAL_MS,
            interval_ms
        )));
    }

    let request = SetIntervalRequest {
        interval_ms,
        callback_id,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::TimerSetInterval, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let timer_response: TimerHandleResponse = decode(&result)?;

    Ok(TimerHandle::new(timer_response.timer_id))
}

/// Clear (cancel) a timer
///
/// Stops a timeout or interval from firing. Safe to call with an already-cleared handle.
///
/// # Arguments
///
/// * `handle` - Timer handle returned from set_timeout or set_interval
///
/// # Errors
///
/// - `NotFound` if the timer doesn't exist (may have already fired or been cleared)
///
/// # Example
///
/// ```rust,ignore
/// use plix_mod_sdk::timers::{set_interval, clear_timer};
///
/// let timer = set_interval(250, 1)?;
///
/// // Later, stop the interval
/// clear_timer(timer)?;
/// ```
pub fn clear_timer(handle: TimerHandle) -> Result<()> {
    let request = ClearTimerRequest {
        timer_id: handle.id,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::TimerClear, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_handle_new() {
        let handle = TimerHandle::new(42);
        assert_eq!(handle.id, 42);
    }

    #[test]
    fn test_min_interval_constant() {
        assert_eq!(MIN_TIMER_INTERVAL_MS, 50);
    }

    #[test]
    fn test_max_timers_constant() {
        assert_eq!(MAX_TIMERS_PER_MOD, 32);
    }
}
