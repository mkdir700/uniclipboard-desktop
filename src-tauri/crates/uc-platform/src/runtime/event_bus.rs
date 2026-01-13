use tokio::sync::mpsc;

use crate::ipc::{PlatformCommand, PlatformEvent};

pub type PlatformEventSender = mpsc::Sender<PlatformEvent>;
pub type PlatformEventReceiver = mpsc::Receiver<PlatformEvent>;

pub type PlatformCommandSender = mpsc::Sender<PlatformCommand>;
pub type PlatformCommandReceiver = mpsc::Receiver<PlatformCommand>;
