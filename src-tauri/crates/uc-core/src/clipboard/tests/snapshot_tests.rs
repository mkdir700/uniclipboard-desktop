//! Tests for [`ClipboardDecisionSnapshot`].

use super::fixtures::*;
use crate::clipboard::ClipboardDecisionSnapshot;

#[test]
fn test_snapshot_new_true() {
    let snapshot = ClipboardDecisionSnapshot::new(true);
    assert_eq!(snapshot.blobs_exist, true);
}

#[test]
fn test_snapshot_new_false() {
    let snapshot = ClipboardDecisionSnapshot::new(false);
    assert_eq!(snapshot.blobs_exist, false);
}

#[test]
fn test_snapshot_is_usable_true() {
    let snapshot = create_snapshot(true);
    assert!(snapshot.is_usable());
}

#[test]
fn test_snapshot_is_usable_false() {
    let snapshot = create_snapshot(false);
    assert!(!snapshot.is_usable());
}

#[test]
fn test_snapshot_clone() {
    let snapshot1 = create_snapshot(true);
    let snapshot2 = snapshot1.clone();
    assert_eq!(snapshot1.blobs_exist, snapshot2.blobs_exist);
}

#[test]
fn test_snapshot_debug() {
    let snapshot = create_snapshot(true);
    let debug_str = format!("{:?}", snapshot);
    assert!(debug_str.contains("ClipboardDecisionSnapshot"));
    assert!(debug_str.contains("blobs_exist"));
}
