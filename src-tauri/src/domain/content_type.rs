use serde::{Deserialize, Serialize};

// 同步内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypes {
    pub text: bool,
    pub image: bool,
    pub link: bool,
    pub file: bool,
    pub code_snippet: bool,
    pub rich_text: bool,
}