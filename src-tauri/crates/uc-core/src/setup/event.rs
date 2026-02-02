use crate::{security::SecretString, setup::SetupError};

#[derive(Debug, PartialEq)]
pub enum SetupEvent {
    // Path selection
    StartNewSpace,
    StartJoinSpace,

    // Create space
    SubmitPassphrase { passphrase: SecretString },

    // Join space
    ChooseJoinPeer { peer_id: String },
    ConfirmPeerTrust,
    VerifyPassphrase { passphrase: SecretString },

    // Results (from orchestrator)
    JoinSpaceSucceeded,
    JoinSpaceFailed { error: SetupError },
    CreateSpaceSucceeded,
    CreateSpaceFailed { error: SetupError },

    // Control
    CancelSetup,
    RefreshPeerList,
}
