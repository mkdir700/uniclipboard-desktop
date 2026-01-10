use anyhow::Result;
use clipboard_rs::common::RustImageData;

// 定义一个 trait 来抽象 ClipboardContext 的行为
pub trait ClipboardContextTrait: Send + Sync {
    fn get_text(&self) -> Result<String>;
    fn set_text(&self, text: String) -> Result<()>;
    fn get_image(&self) -> Result<RustImageData>;
    fn set_image(&self, image: RustImageData) -> Result<()>;
    fn get_files(&self) -> Result<Vec<String>>;
    fn set_files(&self, files: Vec<String>) -> Result<()>;
}
