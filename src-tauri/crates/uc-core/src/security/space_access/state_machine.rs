use chrono::{DateTime, Duration, Utc};

use super::action::SpaceAccessAction;
use super::event::SpaceAccessEvent;
use super::protocol::{DenyReason, SpaceAccessOffer, SpaceAccessProof, SpaceAccessResult};
use super::state::{CancelReason, SpaceAccessState};
use crate::{ids::SpaceId, network::SessionId};

pub fn transition(
    state: SpaceAccessState,
    event: SpaceAccessEvent,
    now: DateTime<Utc>,
) -> (SpaceAccessState, Vec<SpaceAccessAction>) {
    match (state, event) {
        (
            SpaceAccessState::Idle,
            SpaceAccessEvent::StartAsJoiner {
                pairing_session_id,
                ttl_secs,
            },
        ) => {
            let expires_at = now + Duration::seconds(ttl_secs as i64);
            (
                SpaceAccessState::WaitingOffer {
                    pairing_session_id,
                    expires_at,
                },
                vec![SpaceAccessAction::StartTimer { ttl_secs }],
            )
        }
        (
            SpaceAccessState::Idle,
            SpaceAccessEvent::StartAsSponsor {
                pairing_session_id,
                space_id,
                ttl_secs,
            },
        ) => {
            let expires_at = now + Duration::seconds(ttl_secs as i64);
            let challenge_nonce = placeholder_nonce();
            let keyslot_blob = placeholder_keyslot_blob();
            let offer = SpaceAccessOffer {
                pairing_session_id: pairing_session_id.clone(),
                space_id: space_id.clone(),
                keyslot_blob,
                challenge_nonce,
                expires_at,
                version: 1,
            };

            (
                SpaceAccessState::WaitingProof {
                    pairing_session_id,
                    space_id,
                    challenge_nonce,
                    expires_at,
                },
                vec![
                    SpaceAccessAction::SendOffer(offer),
                    SpaceAccessAction::StartTimer { ttl_secs },
                ],
            )
        }

        (
            SpaceAccessState::WaitingOffer {
                pairing_session_id,
                expires_at,
            },
            SpaceAccessEvent::ReceivedOffer(offer),
        ) => {
            if offer.pairing_session_id == pairing_session_id && offer.expires_at > now {
                (
                    SpaceAccessState::WaitingPassphrase {
                        pairing_session_id: offer.pairing_session_id,
                        space_id: offer.space_id,
                        keyslot_blob: offer.keyslot_blob,
                        challenge_nonce: offer.challenge_nonce,
                        expires_at: offer.expires_at,
                    },
                    vec![SpaceAccessAction::StopTimer],
                )
            } else {
                (
                    SpaceAccessState::WaitingOffer {
                        pairing_session_id,
                        expires_at,
                    },
                    vec![],
                )
            }
        }
        (
            SpaceAccessState::WaitingOffer {
                pairing_session_id, ..
            },
            SpaceAccessEvent::Timeout,
        ) => (
            SpaceAccessState::Denied {
                pairing_session_id,
                space_id: unknown_space_id(),
                reason: DenyReason::Expired,
            },
            vec![SpaceAccessAction::StopTimer],
        ),
        (
            SpaceAccessState::WaitingOffer {
                pairing_session_id, ..
            },
            SpaceAccessEvent::CancelByUser,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id,
                reason: CancelReason::UserCancelled,
            },
            vec![SpaceAccessAction::StopTimer],
        ),
        (
            SpaceAccessState::WaitingOffer {
                pairing_session_id, ..
            },
            SpaceAccessEvent::SessionClosed,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id,
                reason: CancelReason::SessionClosed,
            },
            vec![SpaceAccessAction::StopTimer],
        ),

        (
            SpaceAccessState::WaitingPassphrase {
                pairing_session_id,
                space_id,
                keyslot_blob,
                challenge_nonce,
                expires_at,
            },
            SpaceAccessEvent::SubmitPassphrase { passphrase },
        ) => {
            let ttl_secs = ttl_secs_until(expires_at, now);
            let proof = placeholder_proof(
                pairing_session_id.clone(),
                space_id.clone(),
                challenge_nonce,
            );
            (
                SpaceAccessState::WaitingResult {
                    pairing_session_id: pairing_session_id.clone(),
                    space_id: space_id.clone(),
                    challenge_nonce,
                    sent_at: now,
                },
                vec![
                    SpaceAccessAction::DeriveSpaceKeyFromKeyslot {
                        keyslot_blob,
                        passphrase,
                    },
                    SpaceAccessAction::SendProof(proof),
                    SpaceAccessAction::StartTimer { ttl_secs },
                ],
            )
        }
        (
            SpaceAccessState::WaitingPassphrase {
                pairing_session_id,
                space_id,
                ..
            },
            SpaceAccessEvent::Timeout,
        ) => (
            SpaceAccessState::Denied {
                pairing_session_id,
                space_id,
                reason: DenyReason::Expired,
            },
            vec![SpaceAccessAction::StopTimer],
        ),
        (
            SpaceAccessState::WaitingPassphrase {
                pairing_session_id, ..
            },
            SpaceAccessEvent::CancelByUser,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id,
                reason: CancelReason::UserCancelled,
            },
            vec![SpaceAccessAction::StopTimer],
        ),
        (
            SpaceAccessState::WaitingPassphrase {
                pairing_session_id, ..
            },
            SpaceAccessEvent::SessionClosed,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id,
                reason: CancelReason::SessionClosed,
            },
            vec![SpaceAccessAction::StopTimer],
        ),

        (
            SpaceAccessState::WaitingResult {
                pairing_session_id,
                space_id,
                challenge_nonce,
                sent_at,
            },
            SpaceAccessEvent::ReceivedResult(SpaceAccessResult::Granted {
                pairing_session_id: result_session_id,
                space_id: result_space_id,
            }),
        ) => {
            if pairing_session_id == result_session_id && space_id == result_space_id {
                (
                    SpaceAccessState::Granted {
                        pairing_session_id,
                        space_id,
                    },
                    vec![SpaceAccessAction::StopTimer],
                )
            } else {
                (
                    SpaceAccessState::WaitingResult {
                        pairing_session_id,
                        space_id,
                        challenge_nonce,
                        sent_at,
                    },
                    vec![],
                )
            }
        }
        (
            SpaceAccessState::WaitingResult {
                pairing_session_id,
                space_id,
                challenge_nonce,
                sent_at,
            },
            SpaceAccessEvent::ReceivedResult(SpaceAccessResult::Denied {
                pairing_session_id: result_session_id,
                space_id: result_space_id,
                reason,
            }),
        ) => {
            if pairing_session_id == result_session_id && space_id == result_space_id {
                (
                    SpaceAccessState::Denied {
                        pairing_session_id,
                        space_id,
                        reason,
                    },
                    vec![SpaceAccessAction::StopTimer],
                )
            } else {
                (
                    SpaceAccessState::WaitingResult {
                        pairing_session_id,
                        space_id,
                        challenge_nonce,
                        sent_at,
                    },
                    vec![],
                )
            }
        }
        (
            SpaceAccessState::WaitingResult {
                pairing_session_id,
                space_id,
                ..
            },
            SpaceAccessEvent::Timeout,
        ) => (
            SpaceAccessState::Denied {
                pairing_session_id,
                space_id,
                reason: DenyReason::Expired,
            },
            vec![SpaceAccessAction::StopTimer],
        ),

        (
            SpaceAccessState::WaitingProof {
                pairing_session_id,
                space_id,
                challenge_nonce,
                expires_at,
            },
            SpaceAccessEvent::ReceivedProof(proof),
        ) => {
            if proof.pairing_session_id != pairing_session_id {
                return deny_sponsor(pairing_session_id, space_id, DenyReason::SessionMismatch);
            }
            if proof.space_id != space_id {
                return deny_sponsor(pairing_session_id, space_id, DenyReason::SpaceMismatch);
            }
            if proof.challenge_nonce != challenge_nonce {
                return deny_sponsor(pairing_session_id, space_id, DenyReason::InvalidProof);
            }
            if expires_at <= now {
                return deny_sponsor(pairing_session_id, space_id, DenyReason::Expired);
            }

            if proof.proof.is_empty() {
                return deny_sponsor(pairing_session_id, space_id, DenyReason::InvalidProof);
            }

            let pairing_session_id_for_result = pairing_session_id.clone();
            let space_id_for_result = space_id.clone();
            (
                SpaceAccessState::Granted {
                    pairing_session_id,
                    space_id: space_id.clone(),
                },
                vec![
                    SpaceAccessAction::VerifyProof { proof },
                    SpaceAccessAction::SendResult(SpaceAccessResult::Granted {
                        pairing_session_id: pairing_session_id_for_result,
                        space_id: space_id_for_result,
                    }),
                    // TODO: action layer must supply the real peer_id when persisting.
                    SpaceAccessAction::PersistSponsorPairedDevice {
                        space_id,
                        peer_id: String::new(),
                    },
                    SpaceAccessAction::StopTimer,
                ],
            )
        }
        (
            SpaceAccessState::WaitingProof {
                pairing_session_id,
                space_id,
                ..
            },
            SpaceAccessEvent::Timeout,
        ) => deny_sponsor(pairing_session_id, space_id, DenyReason::Expired),
        (
            SpaceAccessState::WaitingProof {
                pairing_session_id,
                space_id,
                ..
            },
            SpaceAccessEvent::CancelByUser,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id: pairing_session_id.clone(),
                reason: CancelReason::UserCancelled,
            },
            vec![
                SpaceAccessAction::SendResult(SpaceAccessResult::Denied {
                    pairing_session_id,
                    space_id,
                    reason: DenyReason::SessionMismatch,
                }),
                SpaceAccessAction::StopTimer,
            ],
        ),
        (
            SpaceAccessState::WaitingProof {
                pairing_session_id,
                space_id,
                ..
            },
            SpaceAccessEvent::SessionClosed,
        ) => (
            SpaceAccessState::Cancelled {
                pairing_session_id: pairing_session_id.clone(),
                reason: CancelReason::SessionClosed,
            },
            vec![
                SpaceAccessAction::SendResult(SpaceAccessResult::Denied {
                    pairing_session_id,
                    space_id,
                    reason: DenyReason::SessionMismatch,
                }),
                SpaceAccessAction::StopTimer,
            ],
        ),

        (state @ SpaceAccessState::Granted { .. }, _)
        | (state @ SpaceAccessState::Denied { .. }, _)
        | (state @ SpaceAccessState::Cancelled { .. }, _) => (state, vec![]),

        (state, _) => (state, vec![]),
    }
}

fn ttl_secs_until(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
    let delta = expires_at.signed_duration_since(now).num_seconds();
    if delta <= 0 {
        0
    } else {
        delta as u64
    }
}

fn unknown_space_id() -> SpaceId {
    SpaceId::from("unknown")
}

fn placeholder_nonce() -> [u8; 32] {
    // TODO: action layer must supply a real nonce; this is a placeholder.
    [0; 32]
}

fn placeholder_keyslot_blob() -> Vec<u8> {
    // TODO: action layer must supply a real keyslot blob; this is a placeholder.
    Vec::new()
}

fn placeholder_proof(
    pairing_session_id: SessionId,
    space_id: SpaceId,
    challenge_nonce: [u8; 32],
) -> SpaceAccessProof {
    // TODO: action layer must supply a real proof/client nonce; this is a placeholder.
    SpaceAccessProof {
        pairing_session_id,
        space_id,
        challenge_nonce,
        proof: Vec::new(),
        client_nonce: [0; 32],
        version: 1,
    }
}

fn deny_sponsor(
    pairing_session_id: SessionId,
    space_id: SpaceId,
    reason: DenyReason,
) -> (SpaceAccessState, Vec<SpaceAccessAction>) {
    (
        SpaceAccessState::Denied {
            pairing_session_id: pairing_session_id.clone(),
            space_id: space_id.clone(),
            reason: reason.clone(),
        },
        vec![
            SpaceAccessAction::SendResult(SpaceAccessResult::Denied {
                pairing_session_id,
                space_id,
                reason,
            }),
            SpaceAccessAction::StopTimer,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn build_offer(
        pairing_session_id: SessionId,
        space_id: SpaceId,
        expires_at: DateTime<Utc>,
    ) -> SpaceAccessOffer {
        SpaceAccessOffer {
            pairing_session_id,
            space_id,
            keyslot_blob: vec![1, 2, 3],
            challenge_nonce: [7; 32],
            expires_at,
            version: 1,
        }
    }

    #[test]
    fn joiner_success_flow() {
        let now = Utc::now();
        let session_id: SessionId = "session-1".into();
        let space_id: SpaceId = "space-1".into();

        let (state, actions) = transition(
            SpaceAccessState::Idle,
            SpaceAccessEvent::StartAsJoiner {
                pairing_session_id: session_id.clone(),
                ttl_secs: 30,
            },
            now,
        );

        assert!(matches!(
            &state,
            SpaceAccessState::WaitingOffer {
                pairing_session_id,
                ..
            } if pairing_session_id == &session_id
        ));
        assert!(matches!(
            actions.as_slice(),
            [SpaceAccessAction::StartTimer { ttl_secs: 30 }]
        ));

        let expires_at = now + Duration::seconds(20);
        let offer = build_offer(session_id.clone(), space_id.clone(), expires_at);
        let (state, actions) =
            transition(state, SpaceAccessEvent::ReceivedOffer(offer.clone()), now);

        assert!(matches!(
            &state,
            SpaceAccessState::WaitingPassphrase {
                pairing_session_id,
                space_id,
                keyslot_blob,
                challenge_nonce,
                expires_at
            } if pairing_session_id == &session_id
                && space_id == &offer.space_id
                && keyslot_blob == &offer.keyslot_blob
                && challenge_nonce == &offer.challenge_nonce
                && expires_at == &offer.expires_at
        ));
        assert!(matches!(actions.as_slice(), [SpaceAccessAction::StopTimer]));

        let submit_time = now + Duration::seconds(1);
        let (state, actions) = transition(
            state,
            SpaceAccessEvent::SubmitPassphrase {
                passphrase: "secret".to_string(),
            },
            submit_time,
        );

        assert!(matches!(
            &state,
            SpaceAccessState::WaitingResult {
                pairing_session_id,
                space_id,
                challenge_nonce,
                sent_at
            } if pairing_session_id == &session_id
                && space_id == &offer.space_id
                && challenge_nonce == &offer.challenge_nonce
                && sent_at == &submit_time
        ));
        assert_eq!(actions.len(), 3);
        match &actions[0] {
            SpaceAccessAction::DeriveSpaceKeyFromKeyslot {
                keyslot_blob,
                passphrase,
            } => {
                assert_eq!(keyslot_blob, &offer.keyslot_blob);
                assert_eq!(passphrase, "secret");
            }
            _ => panic!("expected DeriveSpaceKeyFromKeyslot"),
        }
        assert!(matches!(actions[1], SpaceAccessAction::SendProof(_)));
        assert!(matches!(
            actions[2],
            SpaceAccessAction::StartTimer { ttl_secs: 19 }
        ));

        let (state, actions) = transition(
            state,
            SpaceAccessEvent::ReceivedResult(SpaceAccessResult::Granted {
                pairing_session_id: session_id.clone(),
                space_id: offer.space_id.clone(),
            }),
            submit_time,
        );

        assert!(matches!(
            &state,
            SpaceAccessState::Granted {
                pairing_session_id,
                space_id
            } if pairing_session_id == &session_id && space_id == &offer.space_id
        ));
        assert!(matches!(actions.as_slice(), [SpaceAccessAction::StopTimer]));
    }

    #[test]
    fn joiner_passphrase_invalid_result_denied() {
        let now = Utc::now();
        let session_id: SessionId = "session-1".into();
        let space_id: SpaceId = "space-1".into();
        let expires_at = now + Duration::seconds(20);
        let offer = build_offer(session_id.clone(), space_id.clone(), expires_at);

        let state = SpaceAccessState::WaitingResult {
            pairing_session_id: session_id.clone(),
            space_id: space_id.clone(),
            challenge_nonce: offer.challenge_nonce,
            sent_at: now,
        };

        let (state, actions) = transition(
            state,
            SpaceAccessEvent::ReceivedResult(SpaceAccessResult::Denied {
                pairing_session_id: session_id.clone(),
                space_id: space_id.clone(),
                reason: DenyReason::InvalidProof,
            }),
            now,
        );

        assert!(matches!(
            &state,
            SpaceAccessState::Denied {
                pairing_session_id,
                space_id: state_space_id,
                reason: DenyReason::InvalidProof
            } if pairing_session_id == &session_id && state_space_id == &space_id
        ));
        assert!(matches!(actions.as_slice(), [SpaceAccessAction::StopTimer]));
    }

    #[test]
    fn joiner_timeout_from_waiting_offer() {
        let now = Utc::now();
        let session_id: SessionId = "session-1".into();

        let state = SpaceAccessState::WaitingOffer {
            pairing_session_id: session_id.clone(),
            expires_at: now + Duration::seconds(10),
        };

        let (state, actions) = transition(state, SpaceAccessEvent::Timeout, now);

        assert!(matches!(
            &state,
            SpaceAccessState::Denied {
                pairing_session_id,
                reason: DenyReason::Expired,
                ..
            } if pairing_session_id == &session_id
        ));
        assert!(matches!(actions.as_slice(), [SpaceAccessAction::StopTimer]));
    }
}
