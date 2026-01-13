use clipboard_rs::ClipboardContent;

pub enum PlatformCommand {
    /// 读取剪切板内容
    ReadClipboard,
    /// 写入剪切板内容
    WriteClipboard { content: ClipboardContent },
    /// 启动剪切板监听器
    StartClipboardWatcher,
    /// 停止剪切板监听器
    StopClipboardWatcher,
    /// 关闭
    Shutdown,
}
