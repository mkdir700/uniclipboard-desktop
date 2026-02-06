use serde::Serialize;

use crate::setup::SetupError;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum SetupState {
    /// 尚未开始,
    /// 选择加入方式（New / Join）
    Welcome,

    /// 创建空间：输入口令
    CreateSpaceInputPassphrase {
        error: Option<SetupError>,
    },

    /// 加入空间：选择设备
    JoinSpaceSelectDevice {
        error: Option<SetupError>,
    },

    /// 加入空间：确认设备身份（short code / 指纹）
    JoinSpaceConfirmPeer {
        short_code: String,
        peer_fingerprint: Option<String>,
        error: Option<SetupError>,
    },

    /// 加入空间：输入口令以解锁
    JoinSpaceInputPassphrase {
        error: Option<SetupError>,
    },

    ProcessingCreateSpace {
        message: Option<String>,
    },

    ProcessingJoinSpace {
        message: Option<String>,
    },

    /// 设置完成
    Completed,
}
