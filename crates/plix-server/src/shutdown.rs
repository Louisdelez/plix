//! Graceful Shutdown Handling
//!
//! Provides signal handling (SIGINT/SIGTERM) and coordinated shutdown
//! for the headless server.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::exit_codes::ExitCode;

/// Graceful shutdown coordinator
///
/// Handles OS signals (Ctrl+C on all platforms, SIGTERM on Unix) and
/// coordinates shutdown across all server tasks.
pub struct GracefulShutdown {
    /// Timeout in seconds for graceful shutdown
    timeout_secs: u64,
    /// Broadcast sender for shutdown signal
    shutdown_tx: broadcast::Sender<()>,
    /// Flag indicating shutdown has been initiated
    shutting_down: Arc<AtomicBool>,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown handler.
    ///
    /// # Arguments
    /// * `timeout_secs` - Maximum time to wait for graceful shutdown before force exit
    pub fn new(timeout_secs: u64) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            timeout_secs,
            shutdown_tx,
            shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a receiver for shutdown notifications.
    ///
    /// Tasks should select on this receiver to know when to stop.
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Check if shutdown has been initiated.
    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::Relaxed)
    }

    /// Initiate shutdown.
    ///
    /// Notifies all subscribers that shutdown has begun.
    pub fn initiate_shutdown(&self) {
        if self
            .shutting_down
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            info!("Shutdown initiated");
            let _ = self.shutdown_tx.send(());
        }
    }

    /// Wait for a shutdown signal (SIGINT, SIGTERM, or Ctrl+C).
    ///
    /// Returns the appropriate exit code based on the signal received.
    pub async fn wait_for_shutdown(&self) -> ExitCode {
        // Wait for Ctrl+C (works on all platforms)
        let ctrl_c = tokio::signal::ctrl_c();

        // On Unix, also listen for SIGTERM
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");

            tokio::select! {
                _ = ctrl_c => {
                    info!("Received SIGINT (Ctrl+C)");
                    self.initiate_shutdown();
                    ExitCode::Success
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                    self.initiate_shutdown();
                    ExitCode::Success
                }
            }
        }

        #[cfg(not(unix))]
        {
            ctrl_c.await.expect("Failed to wait for Ctrl+C");
            info!("Received Ctrl+C");
            self.initiate_shutdown();
            ExitCode::Success
        }
    }

    /// Get the shutdown timeout in seconds.
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

impl Clone for GracefulShutdown {
    fn clone(&self) -> Self {
        Self {
            timeout_secs: self.timeout_secs,
            shutdown_tx: self.shutdown_tx.clone(),
            shutting_down: Arc::clone(&self.shutting_down),
        }
    }
}

/// Helper to run shutdown with a timeout.
///
/// This is a convenience function that wraps the common pattern of
/// waiting for shutdown and then executing cleanup with a timeout.
pub async fn shutdown_with_timeout<F, Fut>(
    shutdown: &GracefulShutdown,
    cleanup: F,
) -> Result<(), ShutdownError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    let code = shutdown.wait_for_shutdown().await;

    info!("Running cleanup with {}s timeout...", shutdown.timeout_secs());

    let cleanup_result = tokio::time::timeout(
        std::time::Duration::from_secs(shutdown.timeout_secs()),
        cleanup(),
    )
    .await;

    match cleanup_result {
        Ok(Ok(())) => {
            info!("Cleanup completed successfully");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("Cleanup failed: {}", e);
            Err(ShutdownError::CleanupFailed(e.to_string()))
        }
        Err(_) => {
            warn!(
                "Cleanup timeout after {}s, forcing exit",
                shutdown.timeout_secs()
            );
            Err(ShutdownError::Timeout)
        }
    }
}

/// Errors that can occur during shutdown.
#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    /// Cleanup took too long and timed out.
    #[error("Shutdown timeout exceeded")]
    Timeout,

    /// Cleanup failed with an error.
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_initiation() {
        let shutdown = GracefulShutdown::new(5);

        assert!(!shutdown.is_shutting_down());

        shutdown.initiate_shutdown();

        assert!(shutdown.is_shutting_down());
    }

    #[tokio::test]
    async fn test_shutdown_broadcast() {
        let shutdown = GracefulShutdown::new(5);
        let mut rx = shutdown.subscribe();

        // Initiate shutdown in a separate task
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            shutdown_clone.initiate_shutdown();
        });

        // Wait for shutdown signal
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv()).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_clone() {
        let shutdown1 = GracefulShutdown::new(5);
        let shutdown2 = shutdown1.clone();

        shutdown1.initiate_shutdown();

        // Both should show shutting down
        assert!(shutdown1.is_shutting_down());
        assert!(shutdown2.is_shutting_down());
    }
}
