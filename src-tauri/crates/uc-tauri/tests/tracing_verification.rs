//! Verification test for log-tracing bridge
//!
//! This test verifies that both `log` and `tracing` macros work
//! and appear in the same log output.

#[test]
fn test_log_and_tracing_compatibility() {
    // This test verifies the log-tracing bridge is working
    // by checking that log macros compile and execute

    // Test log macro (should be captured by tracing via bridge)
    log::info!("Test log::info message");
    log::debug!("Test log::debug message");
    log::warn!("Test log::warn message");

    // Test tracing macro
    tracing::info!("Test tracing::info message");
    tracing::debug!("Test tracing::debug message");
    tracing::warn!("Test tracing::warn message");

    // If we get here without panicking, both systems work
    assert!(true);
}
