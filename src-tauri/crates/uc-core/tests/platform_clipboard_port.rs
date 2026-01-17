//! Regression test for PlatformClipboardPort blanket implementation
//!
//! Verifies that the blanket implementation does NOT recursively call itself,
//! which would cause a stack overflow.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use uc_core::clipboard::SystemClipboardSnapshot;
use uc_core::ports::clipboard::{PlatformClipboardPort, SystemClipboardPort};

/// Mock SystemClipboardPort that counts how many times read_snapshot is called
///
/// If PlatformClipboardPort blanket impl calls `self.read_snapshot()` instead of
/// `SystemClipboardPort::read_snapshot(self)`, it will recurse infinitely and
/// the counter will increment until stack overflow.
struct CountingClipboardPort {
    call_count: Arc<AtomicU8>,
}

impl CountingClipboardPort {
    fn new() -> Self {
        Self {
            call_count: Arc::new(AtomicU8::new(0)),
        }
    }

    fn call_count(&self) -> u8 {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl SystemClipboardPort for CountingClipboardPort {
    fn read_snapshot(&self) -> anyhow::Result<SystemClipboardSnapshot> {
        // Increment counter each time this method is called
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        // If counter >= 1, we're being called recursively (BUG!)
        // fetch_add returns the OLD value, so first call returns 0, second returns 1
        if count >= 1 {
            panic!("read_snapshot called recursively! This indicates PlatformClipboardPort blanket impl is calling self.read_snapshot() instead of SystemClipboardPort::read_snapshot(self)");
        }

        Ok(SystemClipboardSnapshot {
            ts_ms: 0,
            representations: vec![],
        })
    }

    fn write_snapshot(&self, _snapshot: SystemClipboardSnapshot) -> anyhow::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_platform_clipboard_port_does_not_recurse() {
    let port = CountingClipboardPort::new();

    // Call through PlatformClipboardPort trait (blanket impl)
    let result: anyhow::Result<SystemClipboardSnapshot> =
        PlatformClipboardPort::read_snapshot(&port);

    // Verify it succeeds without recursion
    assert!(result.is_ok(), "read_snapshot should succeed");

    // Verify SystemClipboardPort::read_snapshot was called exactly once
    // If blanket impl calls `self.read_snapshot()` instead of `SystemClipboardPort::read_snapshot(self)`,
    // it would recurse infinitely and panic
    assert_eq!(
        port.call_count(),
        1,
        "read_snapshot should be called exactly once"
    );
}

#[tokio::test]
async fn test_platform_clipboard_port_returns_correct_snapshot() {
    let port = CountingClipboardPort::new();

    let snapshot = PlatformClipboardPort::read_snapshot(&port).unwrap();

    // Verify the snapshot is what we expect
    assert_eq!(snapshot.ts_ms, 0);
    assert_eq!(snapshot.representations.len(), 0);
}
