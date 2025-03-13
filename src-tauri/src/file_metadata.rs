use chrono::{DateTime, Utc};
use reqwest_dav::list_cmd::ListFile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String, // {device_id}_{content_hash}.json
    pub dir: String,
    pub size: u64,
    pub last_modified: DateTime<Utc>,
    pub content_type: String,
    pub tag: Option<String>,
}

impl FileMetadata {
    pub fn from_list_file(list_file: &ListFile, host: &str) -> Self {
        let prefix = Self::get_prefix(host).unwrap();
        let path = list_file.href.replacen(&prefix, "", 1);
        let (dir, name) = path.rsplit_once('/').unwrap();
        let dir = dir.to_string();
        let name = name.to_string();
        Self {
            name,
            dir,
            size: list_file.content_length as u64,
            last_modified: list_file.last_modified,
            content_type: list_file.content_type.clone(),
            tag: list_file.tag.clone(),
        }
    }

    pub fn get_path(&self) -> String {
        format!("{}/{}", self.dir, self.name)
    }

    /// Get the device id from the filename
    ///
    /// The filename is in the format of {device_id}_{uuid}.json
    #[allow(dead_code)]
    pub fn get_device_id(&self) -> String {
        self.name.split("_").next().unwrap().to_string()
    }

    #[allow(dead_code)]
    pub fn is_newer_than(&self, other: &Self) -> bool {
        self.last_modified > other.last_modified
    }

    #[allow(dead_code)]
    pub fn get_content_hash(&self) -> Option<String> {
        let name_parts: Vec<&str> = self.name.split('_').collect();
        if name_parts.len() >= 2 {
            Some(name_parts[1].to_string())
        } else {
            None
        }
    }

    pub fn get_prefix(url: &str) -> Option<String> {
        url.split('/')
            .skip(3) // 跳过 "https:" 和两个空字符串
            .next()
            .map(|s| format!("/{}", s))
    }
}
