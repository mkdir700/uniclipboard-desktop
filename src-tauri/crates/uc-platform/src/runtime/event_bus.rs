use tokio::sync::mpsc;

use crate::ipc::{PlatformCommand, PlatformEvent};

pub type EventBus = mpsc::Sender<PlatformEvent>;
pub type EventReceiver = mpsc::Receiver<PlatformEvent>;

pub type CommandBus = mpsc::Sender<PlatformCommand>;
pub type CommandReceiver = mpsc::Receiver<PlatformCommand>;
