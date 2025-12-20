//! Timer API: scheduled callbacks with bounded limits

use crate::errors::{err_not_found, err_rate, ModApiError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimum timer interval (50ms)
pub const MIN_TIMER_INTERVAL_MS: u32 = 50;

/// Maximum timers per mod (32)
pub const MAX_TIMERS_PER_MOD: usize = 32;

/// Handle for a registered timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimerHandle {
    /// Unique timer ID within the mod
    pub id: u32,
}

impl TimerHandle {
    /// Create a new timer handle
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

/// Timer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerType {
    /// One-time timer (set_timeout)
    Timeout,
    /// Repeating timer (set_interval)
    Interval,
}

/// Timer entry
#[derive(Debug, Clone)]
pub struct Timer {
    /// Timer handle
    pub handle: TimerHandle,
    /// Timer type
    pub timer_type: TimerType,
    /// Interval in milliseconds
    pub interval_ms: u32,
    /// Callback ID (mod-defined)
    pub callback_id: u32,
    /// Time remaining until next fire (in ms)
    pub remaining_ms: u32,
}

impl Timer {
    /// Create a new timeout timer
    pub fn timeout(handle: TimerHandle, delay_ms: u32, callback_id: u32) -> Self {
        let delay_ms = delay_ms.max(MIN_TIMER_INTERVAL_MS);
        Self {
            handle,
            timer_type: TimerType::Timeout,
            interval_ms: delay_ms,
            callback_id,
            remaining_ms: delay_ms,
        }
    }

    /// Create a new interval timer
    pub fn interval(handle: TimerHandle, interval_ms: u32, callback_id: u32) -> Self {
        let interval_ms = interval_ms.max(MIN_TIMER_INTERVAL_MS);
        Self {
            handle,
            timer_type: TimerType::Interval,
            interval_ms,
            callback_id,
            remaining_ms: interval_ms,
        }
    }
}

/// Trait for timer management
///
/// This trait is implemented by the game engine to handle timer callbacks.
pub trait TimerApi {
    /// Schedule a one-time callback
    ///
    /// # Parameters
    /// - `delay_ms`: Delay in milliseconds (clamped to min 50ms)
    /// - `callback_id`: Mod-defined callback identifier
    ///
    /// # Errors
    /// - EMOD005: Exceeded max timers (32 per mod)
    fn set_timeout(&mut self, delay_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError>;

    /// Schedule a repeating callback
    ///
    /// # Parameters
    /// - `interval_ms`: Interval in milliseconds (clamped to min 50ms)
    /// - `callback_id`: Mod-defined callback identifier
    ///
    /// # Errors
    /// - EMOD005: Exceeded max timers (32 per mod)
    fn set_interval(
        &mut self,
        interval_ms: u32,
        callback_id: u32,
    ) -> Result<TimerHandle, ModApiError>;

    /// Cancel a timer
    ///
    /// # Errors
    /// - EMOD003: Timer not found (already fired or invalid handle)
    fn clear_timer(&mut self, handle: TimerHandle) -> Result<(), ModApiError>;
}

/// Per-mod timer storage
#[derive(Debug, Default)]
pub struct ModTimers {
    /// Active timers by handle ID
    timers: HashMap<u32, Timer>,
    /// Next timer ID
    next_id: u32,
}

impl ModTimers {
    /// Create a new timer storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of active timers
    pub fn count(&self) -> usize {
        self.timers.len()
    }

    /// Check if at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.count() >= MAX_TIMERS_PER_MOD
    }

    /// Create a timeout timer
    pub fn set_timeout(
        &mut self,
        delay_ms: u32,
        callback_id: u32,
    ) -> Result<TimerHandle, ModApiError> {
        if self.is_at_capacity() {
            return Err(err_rate(format!(
                "Exceeded maximum of {} timers per mod",
                MAX_TIMERS_PER_MOD
            )));
        }

        let handle = TimerHandle::new(self.next_id);
        self.next_id += 1;

        let timer = Timer::timeout(handle, delay_ms, callback_id);
        self.timers.insert(handle.id, timer);

        Ok(handle)
    }

    /// Create an interval timer
    pub fn set_interval(
        &mut self,
        interval_ms: u32,
        callback_id: u32,
    ) -> Result<TimerHandle, ModApiError> {
        if self.is_at_capacity() {
            return Err(err_rate(format!(
                "Exceeded maximum of {} timers per mod",
                MAX_TIMERS_PER_MOD
            )));
        }

        let handle = TimerHandle::new(self.next_id);
        self.next_id += 1;

        let timer = Timer::interval(handle, interval_ms, callback_id);
        self.timers.insert(handle.id, timer);

        Ok(handle)
    }

    /// Cancel a timer
    pub fn clear_timer(&mut self, handle: TimerHandle) -> Result<(), ModApiError> {
        if self.timers.remove(&handle.id).is_some() {
            Ok(())
        } else {
            Err(err_not_found("Timer"))
        }
    }

    /// Get a timer by handle
    pub fn get(&self, handle: TimerHandle) -> Option<&Timer> {
        self.timers.get(&handle.id)
    }

    /// Process timers: advance time and collect fired timers
    ///
    /// Returns (callback_id, handle) for each fired timer.
    /// Timeout timers are removed, interval timers are reset.
    pub fn tick(&mut self, delta_ms: u32) -> Vec<(u32, TimerHandle)> {
        let mut fired = Vec::new();
        let mut to_remove = Vec::new();

        for (id, timer) in self.timers.iter_mut() {
            if timer.remaining_ms <= delta_ms {
                // Timer fired
                fired.push((timer.callback_id, timer.handle));

                match timer.timer_type {
                    TimerType::Timeout => {
                        to_remove.push(*id);
                    }
                    TimerType::Interval => {
                        // Reset for next interval
                        timer.remaining_ms = timer.interval_ms;
                    }
                }
            } else {
                timer.remaining_ms -= delta_ms;
            }
        }

        // Remove fired timeouts
        for id in to_remove {
            self.timers.remove(&id);
        }

        fired
    }

    /// Clear all timers
    pub fn clear_all(&mut self) {
        self.timers.clear();
    }

    /// Iterate over all timers
    pub fn iter(&self) -> impl Iterator<Item = &Timer> {
        self.timers.values()
    }
}

impl TimerApi for ModTimers {
    fn set_timeout(&mut self, delay_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError> {
        ModTimers::set_timeout(self, delay_ms, callback_id)
    }

    fn set_interval(
        &mut self,
        interval_ms: u32,
        callback_id: u32,
    ) -> Result<TimerHandle, ModApiError> {
        ModTimers::set_interval(self, interval_ms, callback_id)
    }

    fn clear_timer(&mut self, handle: TimerHandle) -> Result<(), ModApiError> {
        ModTimers::clear_timer(self, handle)
    }
}

/// Global timer storage for all mods
#[derive(Debug, Default)]
pub struct GlobalTimers {
    /// Per-mod timer storage
    mod_timers: HashMap<String, ModTimers>,
}

impl GlobalTimers {
    /// Create a new global timer storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create timer storage for a mod
    pub fn get_or_create(&mut self, mod_id: &str) -> &mut ModTimers {
        self.mod_timers.entry(mod_id.to_string()).or_default()
    }

    /// Remove a mod's timers
    pub fn remove_mod(&mut self, mod_id: &str) {
        self.mod_timers.remove(mod_id);
    }

    /// Process all timers
    ///
    /// Returns (mod_id, callback_id, handle) for each fired timer.
    pub fn tick(&mut self, delta_ms: u32) -> Vec<(String, u32, TimerHandle)> {
        let mut all_fired = Vec::new();

        for (mod_id, timers) in self.mod_timers.iter_mut() {
            let fired = timers.tick(delta_ms);
            for (callback_id, handle) in fired {
                all_fired.push((mod_id.clone(), callback_id, handle));
            }
        }

        all_fired
    }

    /// Get total active timer count across all mods
    pub fn total_count(&self) -> usize {
        self.mod_timers.values().map(|t| t.count()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_handle() {
        let h1 = TimerHandle::new(1);
        let h2 = TimerHandle::new(1);
        let h3 = TimerHandle::new(2);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_min_interval_clamping() {
        let timer = Timer::timeout(TimerHandle::new(0), 10, 0);
        assert_eq!(timer.remaining_ms, MIN_TIMER_INTERVAL_MS);

        let timer = Timer::interval(TimerHandle::new(0), 25, 0);
        assert_eq!(timer.interval_ms, MIN_TIMER_INTERVAL_MS);

        let timer = Timer::timeout(TimerHandle::new(0), 100, 0);
        assert_eq!(timer.remaining_ms, 100);
    }

    #[test]
    fn test_set_timeout() {
        let mut timers = ModTimers::new();

        let handle = timers.set_timeout(100, 42).unwrap();
        assert_eq!(timers.count(), 1);

        let timer = timers.get(handle).unwrap();
        assert_eq!(timer.timer_type, TimerType::Timeout);
        assert_eq!(timer.callback_id, 42);
        assert_eq!(timer.remaining_ms, 100);
    }

    #[test]
    fn test_set_interval() {
        let mut timers = ModTimers::new();

        let handle = timers.set_interval(200, 10).unwrap();
        let timer = timers.get(handle).unwrap();

        assert_eq!(timer.timer_type, TimerType::Interval);
        assert_eq!(timer.interval_ms, 200);
    }

    #[test]
    fn test_clear_timer() {
        let mut timers = ModTimers::new();

        let handle = timers.set_timeout(100, 1).unwrap();
        assert_eq!(timers.count(), 1);

        timers.clear_timer(handle).unwrap();
        assert_eq!(timers.count(), 0);
    }

    #[test]
    fn test_clear_invalid_timer() {
        let mut timers = ModTimers::new();
        let handle = TimerHandle::new(999);

        let result = timers.clear_timer(handle);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, crate::errors::ErrorCode::NotFound);
    }

    #[test]
    fn test_max_timers() {
        let mut timers = ModTimers::new();

        // Fill to capacity
        for i in 0..MAX_TIMERS_PER_MOD {
            timers.set_timeout(100, i as u32).unwrap();
        }

        assert!(timers.is_at_capacity());

        // Next one should fail
        let result = timers.set_timeout(100, 999);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::RateLimited
        );
    }

    #[test]
    fn test_timeout_tick() {
        let mut timers = ModTimers::new();

        let handle1 = timers.set_timeout(100, 1).unwrap();
        let _handle2 = timers.set_timeout(200, 2).unwrap();

        // Tick 100ms - first timer fires
        let fired = timers.tick(100);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].0, 1); // callback_id
        assert_eq!(fired[0].1, handle1);

        // Timer was removed
        assert_eq!(timers.count(), 1);
    }

    #[test]
    fn test_interval_tick() {
        let mut timers = ModTimers::new();

        let handle = timers.set_interval(100, 1).unwrap();

        // First fire
        let fired = timers.tick(100);
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].1, handle);

        // Timer still exists
        assert_eq!(timers.count(), 1);

        // Second fire
        let fired = timers.tick(100);
        assert_eq!(fired.len(), 1);

        // Still exists
        assert_eq!(timers.count(), 1);
    }

    #[test]
    fn test_tick_partial() {
        let mut timers = ModTimers::new();

        timers.set_timeout(100, 1).unwrap();

        // Tick 50ms - no fire
        let fired = timers.tick(50);
        assert!(fired.is_empty());
        assert_eq!(timers.count(), 1);

        // Timer has 50ms remaining
        let timer = timers.iter().next().unwrap();
        assert_eq!(timer.remaining_ms, 50);
    }

    #[test]
    fn test_global_timers() {
        let mut global = GlobalTimers::new();

        // Create timers for two mods
        global.get_or_create("mod1").set_timeout(100, 1).unwrap();
        global.get_or_create("mod2").set_timeout(100, 2).unwrap();

        assert_eq!(global.total_count(), 2);

        // Tick and collect
        let fired = global.tick(100);
        assert_eq!(fired.len(), 2);

        // Verify mod IDs
        let mod_ids: Vec<_> = fired.iter().map(|(id, _, _)| id.as_str()).collect();
        assert!(mod_ids.contains(&"mod1"));
        assert!(mod_ids.contains(&"mod2"));
    }

    #[test]
    fn test_global_timers_remove_mod() {
        let mut global = GlobalTimers::new();

        global.get_or_create("mod1").set_timeout(100, 1).unwrap();
        global.get_or_create("mod1").set_timeout(100, 2).unwrap();

        assert_eq!(global.total_count(), 2);

        global.remove_mod("mod1");

        assert_eq!(global.total_count(), 0);
    }
}
