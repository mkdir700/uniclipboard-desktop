use base64::alphabet::ParseAlphabetError;

use super::decision::ClipboardContentActionDecision;
use super::event::ClipboardContentActionEvent;
use crate::clipboard::decision::RejectReason;
use crate::clipboard::event::ClipboardContentAction;
use crate::ports::ClipboardHistoryPort;

/// Map a permission flag to a clipboard content action decision.
///
/// Returns `Allow` when the flag indicates permission; otherwise returns `Reject` with
/// `RejectReason::PolicyDenied`.
///
/// # Examples
///
/// ```
/// let decision = check_permission(true);
/// assert!(matches!(decision, ClipboardContentActionDecision::Allow));
///
/// let decision = check_permission(false);
/// assert!(matches!(decision, ClipboardContentActionDecision::Reject { reason: RejectReason::PolicyDenied }));
/// ```
fn check_permission(permission: bool) -> ClipboardContentActionDecision {
    if permission {
        ClipboardContentActionDecision::Allow
    } else {
        ClipboardContentActionDecision::Reject {
            reason: RejectReason::PolicyDenied,
        }
    }
}

pub struct ClipboardContentDecisionDomain<H>
where
    H: ClipboardHistoryPort,
{
    history: H,
}

impl<H> ClipboardContentDecisionDomain<H>
where
    H: ClipboardHistoryPort,
{
    /// Creates a new ClipboardContentDecisionDomain backed by the given history port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use uc_core::clipboard::domain::ClipboardContentDecisionDomain;
    /// # use uc_core::ports::ClipboardHistoryPort;
    /// let history = /* an implementor of ClipboardHistoryPort */ unimplemented!();
    /// let domain = ClipboardContentDecisionDomain::new(history);
    /// ```
    pub fn new(history: H) -> Self {
        Self { history }
    }

    /// Decides whether a clipboard action should be allowed or rejected based on the event and history.
    ///
    /// Queries the history port for a snapshot decision when a `UserRequested` event arrives:
    /// - Returns `Allow` if the history yields a usable snapshot for the given content hash.
    /// - Returns `Reject { reason: RejectReason::NotFound }` if the snapshot is missing or not usable.
    /// - Returns `Reject { reason: RejectReason::InternalError }` if the history port returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// // This example is illustrative; replace `MockHistory`, `Snapshot`, and construction with real types.
    /// use futures::executor::block_on;
    ///
    /// // Construct a domain with a history port (mock or real) that yields a usable snapshot.
    /// let history = MockHistory::with_usable_snapshot();
    /// let domain = ClipboardContentDecisionDomain::new(history);
    ///
    /// let event = ClipboardContentActionEvent::UserRequested { content_hash: "hash".into(), /* ... */ };
    /// let decision = block_on(domain.apply(event));
    ///
    /// assert!(matches!(decision, ClipboardContentActionDecision::Allow));
    /// ```
    pub async fn apply(
        &self,
        event: ClipboardContentActionEvent,
    ) -> ClipboardContentActionDecision {
        match event {
            ClipboardContentActionEvent::UserRequested { content_hash, .. } => {
                match self.history.get_snapshot_decision(&content_hash).await {
                    Ok(Some(snapshot)) if snapshot.is_usable() => {
                        ClipboardContentActionDecision::Allow
                    }
                    Ok(Some(_)) | Ok(None) => ClipboardContentActionDecision::Reject {
                        reason: RejectReason::NotFound,
                    },
                    Err(_) => ClipboardContentActionDecision::Reject {
                        reason: RejectReason::InternalError,
                    },
                }
            }
        }
    }
}