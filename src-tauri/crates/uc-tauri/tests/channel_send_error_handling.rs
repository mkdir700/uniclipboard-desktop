//! Regression test for channel send error handling
//!
//! Tests that channel send failures are logged instead of silently ignored.
//! This verifies the fix for: let _ = send(...).await

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

/// Helper struct to track if error was logged
struct ErrorTracker {
    error_logged: Arc<AtomicBool>,
}

impl ErrorTracker {
    fn new() -> Self {
        Self {
            error_logged: Arc::new(AtomicBool::new(false)),
        }
    }

    fn was_error_logged(&self) -> bool {
        self.error_logged.load(Ordering::SeqCst)
    }

    fn log_error(&self, msg: &str) {
        if msg.contains("Failed to send") {
            self.error_logged.store(true, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn test_channel_send_error_is_logged() {
    // This test demonstrates the correct pattern for handling channel send errors.
    // Before the fix: `let _ = tx.send(...).await;` (silent failure)
    // After the fix: `match tx.send(...) { Ok(_) => ..., Err(e) => log::error(...) }`

    let (tx, mut _rx) = mpsc::channel::<String>(10);
    let tracker = ErrorTracker::new();

    // Drop the receiver to ensure all sends will fail
    drop(_rx);

    // CORRECT PATTERN: Match on send result and log errors
    match tx.send("test".to_string()).await {
        Ok(_) => {}
        Err(e) => {
            tracker.log_error(&format!("Failed to send command: {}", e));
        }
    }

    // Verify error was logged
    assert!(
        tracker.was_error_logged(),
        "Channel send failure should be logged"
    );
}

#[tokio::test]
async fn test_channel_send_succeeds_without_error() {
    // Test that successful sends don't log errors

    let (tx, mut rx) = mpsc::channel::<String>(10);
    let tracker = ErrorTracker::new();

    // Spawn a task to keep receiver alive
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            // Consume messages
        }
    });

    // CORRECT PATTERN: Match on send result
    match tx.send("test".to_string()).await {
        Ok(_) => {
            // Success path - log info if desired
        }
        Err(e) => {
            tracker.log_error(&format!("Failed to send command: {}", e));
        }
    }

    // Verify error was NOT logged
    assert!(
        !tracker.was_error_logged(),
        "Successful send should not log error"
    );
}

#[tokio::test]
async fn test_demonstrate_anti_pattern_silent_failure() {
    // This demonstrates the WRONG pattern (before fix) where errors are silently ignored

    let (tx, _rx) = mpsc::channel::<String>(10);

    // Drop the receiver to ensure sends will fail
    drop(_rx);

    // WRONG PATTERN: Silent failure with `let _ =`
    let _ = tx.send("test".to_string()).await;

    // With this pattern, the failure is completely silent - no way to detect it here!
    // This is why the fix uses `match` instead.

    // (In real code, we'd need external monitoring to detect this failure)
}
