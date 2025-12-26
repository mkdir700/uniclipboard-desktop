use crate::infrastructure::security::encryption::Encryptor;
use crate::infrastructure::security::password::{PasswordRequest, PASSWORD_SENDER};
use crate::message::Payload;
use crate::utils::helpers::string_to_32_bytes;
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use reqwest::StatusCode;
use reqwest_dav::list_cmd::{ListEntity, ListFile};
use reqwest_dav::{Auth, ClientBuilder, Depth};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

type WebDavResult<T> = std::result::Result<T, WebDavError>;

#[derive(Debug, Clone)]
pub struct WebDavConfig {
    pub host: String,
    pub username: String,
    pub base_path: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub retry_backoff: Duration,
}

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
        let prefix = Self::get_prefix(host).unwrap_or_default();
        let path = list_file.href.replacen(&prefix, "", 1);
        let (dir, name) = path.rsplit_once('/').unwrap_or(("", &path));
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
        self.name.split('_').next().unwrap_or_default().to_string()
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

#[derive(Debug, Clone)]
pub enum WebDavError {
    Timeout,
    Authentication(String),
    Permission(String),
    InsufficientStorage(String),
    AlreadyExists,
    NotFound(String),
    Network(String),
    Encryption(String),
    Serialization(String),
    Stronghold(String),
    Protocol(String),
    UnexpectedStatus(StatusCode),
}

impl Display for WebDavError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WebDavError::Timeout => write!(f, "WebDAV 请求超时"),
            WebDavError::Authentication(msg) => write!(f, "认证失败: {}", msg),
            WebDavError::Permission(msg) => write!(f, "权限不足: {}", msg),
            WebDavError::InsufficientStorage(msg) => write!(f, "存储空间不足: {}", msg),
            WebDavError::AlreadyExists => write!(f, "资源已存在"),
            WebDavError::NotFound(msg) => write!(f, "资源未找到: {}", msg),
            WebDavError::Network(msg) => write!(f, "网络错误: {}", msg),
            WebDavError::Encryption(msg) => write!(f, "加密/解密失败: {}", msg),
            WebDavError::Serialization(msg) => write!(f, "序列化失败: {}", msg),
            WebDavError::Stronghold(msg) => write!(f, "Stronghold 错误: {}", msg),
            WebDavError::Protocol(msg) => write!(f, "协议错误: {}", msg),
            WebDavError::UnexpectedStatus(code) => write!(f, "意外的状态码: {}", code),
        }
    }
}

impl std::error::Error for WebDavError {}

impl WebDavError {
    fn should_retry(&self) -> bool {
        matches!(self, WebDavError::Timeout | WebDavError::Network(_))
    }
}

pub struct WebDAVClient {
    client: reqwest_dav::Client,
    encryptor: Encryptor,
    base_path: String,
    retry_attempts: u32,
    retry_backoff: Duration,
}

impl WebDAVClient {
    #[allow(dead_code)]
    pub async fn new(config: WebDavConfig) -> WebDavResult<Self> {
        let password = fetch_password_from_stronghold().await?;
        let key = string_to_32_bytes(&password);
        let encryptor = Encryptor::from_key(&key);

        let agent = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| WebDavError::Network(format!("构建 HTTP 客户端失败: {}", e)))?;

        let client = ClientBuilder::new()
            .set_agent(agent)
            .set_host(config.host.clone())
            .set_auth(Auth::Basic(config.username.clone(), password))
            .build()
            .map_err(WebDavError::from)?;

        Ok(Self {
            client,
            encryptor,
            base_path: normalize_base_path(&config.base_path),
            retry_attempts: config.retry_attempts,
            retry_backoff: config.retry_backoff,
        })
    }

    /// 检查是否连接到 WebDAV 服务器
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        self.list("", Depth::Number(0)).await.is_ok()
    }

    #[allow(dead_code)]
    pub async fn initialize_share_directory(&self, dir: String) -> WebDavResult<()> {
        let full_path = self.full_path(&dir);
        self.retry("mkcol", || {
            let path = full_path.clone();
            async move {
                match self.client.mkcol(&path).await.map_err(WebDavError::from) {
                    Ok(()) => Ok(()),
                    Err(WebDavError::AlreadyExists) => {
                        info!("目录已存在，忽略创建: {}", path);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
        })
        .await
    }

    #[allow(dead_code)]
    pub async fn is_share_code_exists(&self) -> bool {
        match self.list("uniclipboard", Depth::Number(0)).await {
            Ok(entries) => !entries.is_empty(),
            Err(err) => {
                warn!("检查共享目录失败: {}", err);
                false
            }
        }
    }

    /// Uploads a Payload to the specified directory on the WebDAV server.
    pub async fn upload(&self, dir: String, payload: Payload) -> WebDavResult<String> {
        let filename = format!("{}_{}.bin", payload.get_device_id(), payload.get_key());
        let target_dir = self.full_path(&dir);
        let path = if target_dir == "/" {
            format!("/{}", filename)
        } else {
            format!("{}/{}", target_dir.trim_end_matches('/'), filename)
        };
        let json_payload = payload.to_json();
        let encrypted_payload = self
            .encryptor
            .encrypt(&json_payload.as_bytes())
            .map_err(|e| WebDavError::Encryption(e.to_string()))?;

        info!("Uploading payload to {}", path);
        self.retry("upload", || {
            let body = encrypted_payload.clone();
            let path = path.clone();
            async move {
                self.client
                    .put(&path, body)
                    .await
                    .map_err(WebDavError::from)
            }
        })
        .await?;

        Ok(path)
    }

    /// Downloads a Payload from the specified path on the WebDAV server.
    pub async fn download(&self, path: String) -> WebDavResult<Payload> {
        let full_path = self.full_path(&path);
        info!("Downloading payload from {}", full_path);
        let response = self
            .retry("download", || {
                let path = full_path.clone();
                async move { self.client.get(&path).await.map_err(WebDavError::from) }
            })
            .await?;

        if response.status().is_success() {
            let content = response
                .bytes()
                .await
                .map_err(|e| WebDavError::Network(format!("读取响应失败: {}", e)))?;
            let decrypted_payload = self
                .encryptor
                .decrypt(&content)
                .map_err(|e| WebDavError::Encryption(e.to_string()))?;
            let payload = serde_json::from_slice(&decrypted_payload)
                .map_err(|e| WebDavError::Serialization(e.to_string()))?;
            Ok(payload)
        } else {
            Err(WebDavError::UnexpectedStatus(response.status()))
        }
    }

    /// Counts the number of files in the specified directory on the WebDAV server.
    pub async fn count_files(&self, path: String) -> WebDavResult<usize> {
        let entries = self.list(&path, Depth::Number(1)).await?;
        Ok(entries.len().saturating_sub(1))
    }

    #[allow(dead_code)]
    pub async fn fetch_latest_file(&self, dir: String) -> WebDavResult<Payload> {
        let entries = self.list(&dir, Depth::Number(1)).await?;
        let latest_file = entries
            .iter()
            .filter_map(|entity| match entity {
                ListEntity::File(file) => Some(file),
                _ => None,
            })
            .max_by_key(|file| file.last_modified);

        let target = latest_file
            .as_ref()
            .map(|f| f.href.clone())
            .ok_or_else(|| WebDavError::NotFound("No files found".to_string()))?;

        let response = self
            .retry("get_latest_file", || {
                let href = target.clone();
                async move { self.client.get(&href).await.map_err(WebDavError::from) }
            })
            .await?;

        if response.status().is_success() {
            let content = response
                .bytes()
                .await
                .map_err(|e| WebDavError::Network(format!("读取响应失败: {}", e)))?;
            let decrypted_payload = self
                .encryptor
                .decrypt(&content)
                .map_err(|e| WebDavError::Encryption(e.to_string()))?;
            let payload = serde_json::from_slice(&decrypted_payload)
                .map_err(|e| WebDavError::Serialization(e.to_string()))?;
            Ok(payload)
        } else {
            Err(WebDavError::UnexpectedStatus(response.status()))
        }
    }

    /// Fetches the metadata of the latest file from the specified directory on the WebDAV server.
    pub async fn fetch_latest_file_meta(&self, dir: String) -> WebDavResult<FileMetadata> {
        let entries = self.list(&dir, Depth::Number(1)).await?;
        let list_file = entries
            .iter()
            .filter_map(|entity| match entity {
                ListEntity::File(file) => Some(file),
                _ => None,
            })
            .max_by_key(|file| file.last_modified)
            .ok_or_else(|| WebDavError::NotFound("No files found".to_string()))?;

        let meta = FileMetadata::from_list_file(list_file, &self.client.host);
        Ok(meta)
    }

    /// Fetches the metadata of the oldest file from the specified directory on the WebDAV server.
    pub async fn fetch_oldest_file_meta(&self, dir: String) -> WebDavResult<FileMetadata> {
        let entries = self.list(&dir, Depth::Number(1)).await?;
        let list_file = entries
            .iter()
            .filter_map(|entity| match entity {
                ListEntity::File(file) => Some(file),
                _ => None,
            })
            .min_by_key(|file| file.last_modified)
            .ok_or_else(|| WebDavError::NotFound("No files found".to_string()))?;

        let meta = FileMetadata::from_list_file(list_file, &self.client.host);
        Ok(meta)
    }

    #[allow(dead_code)]
    pub async fn delete(&self, path: String) -> WebDavResult<()> {
        let full_path = self.full_path(&path);
        self.retry("delete", || {
            let path = full_path.clone();
            async move { self.client.delete(&path).await.map_err(WebDavError::from) }
        })
        .await
    }

    fn full_path(&self, relative: &str) -> String {
        let relative = relative.trim_matches('/');
        if self.base_path == "/" {
            if relative.is_empty() {
                "/".to_string()
            } else {
                format!("/{}", relative)
            }
        } else if relative.is_empty() {
            self.base_path.clone()
        } else {
            let base_trimmed = self.base_path.trim_matches('/');
            if relative == base_trimmed || relative.starts_with(&format!("{}/", base_trimmed)) {
                format!("/{}", relative)
            } else {
                format!("{}/{}", self.base_path.trim_end_matches('/'), relative)
            }
        }
    }

    async fn list(&self, path: &str, depth: Depth) -> WebDavResult<Vec<ListEntity>> {
        let target = self.full_path(path);
        debug!("Listing path: {} depth: {:?}", target, depth);
        let response = self
            .retry("list", || {
                let target = target.clone();
                async move {
                    self.client
                        .list(&target, depth)
                        .await
                        .map_err(WebDavError::from)
                }
            })
            .await?;
        Ok(response)
    }

    async fn retry<F, Fut, T>(&self, op: &str, mut action: F) -> WebDavResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = WebDavResult<T>>,
    {
        for attempt in 0..=self.retry_attempts {
            match action().await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    if attempt == self.retry_attempts || !err.should_retry() {
                        error!("{} failed after {} attempts: {}", op, attempt + 1, err);
                        return Err(err);
                    }
                    let backoff = self.retry_backoff * (attempt + 1);
                    warn!(
                        "{} failed (attempt {}): {}. retrying in {:?}",
                        op,
                        attempt + 1,
                        err,
                        backoff
                    );
                    sleep(backoff).await;
                }
            }
        }

        Err(WebDavError::Network(format!("{} 未知错误：重试耗尽", op)))
    }
}

fn normalize_base_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else {
        format!("/{}", trimmed.trim_matches('/'))
    }
}

async fn fetch_password_from_stronghold() -> WebDavResult<String> {
    let sender = PASSWORD_SENDER
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| WebDavError::Stronghold("密码通道未初始化".to_string()))?;

    let (tx, mut rx) = mpsc::channel(1);
    sender
        .send(PasswordRequest::GetEncryptionPassword(tx))
        .await
        .map_err(|e| WebDavError::Stronghold(format!("发送 Stronghold 请求失败: {}", e)))?;

    let result = rx
        .recv()
        .await
        .ok_or_else(|| WebDavError::Stronghold("Stronghold 工作线程未响应".to_string()))?;

    match result {
        Ok(Some(password)) => Ok(password),
        Ok(None) => Err(WebDavError::Stronghold(
            "Stronghold 中未找到加密口令".to_string(),
        )),
        Err(e) => Err(WebDavError::Stronghold(format!(
            "读取 Stronghold 失败: {}",
            e
        ))),
    }
}

impl From<reqwest::Error> for WebDavError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            WebDavError::Timeout
        } else if let Some(status) = error.status() {
            map_status_code(status)
        } else {
            WebDavError::Network(error.to_string())
        }
    }
}

impl From<reqwest_dav::types::Error> for WebDavError {
    fn from(error: reqwest_dav::types::Error) -> Self {
        use reqwest_dav::types::{DecodeError, Error as DavError, ReqwestDecodeError};
        match error {
            DavError::Reqwest(err) => WebDavError::from(err),
            DavError::ReqwestDecode(ReqwestDecodeError::InvalidMethod(err)) => {
                WebDavError::Protocol(format!("无效方法: {}", err))
            }
            DavError::ReqwestDecode(ReqwestDecodeError::Url(err)) => {
                WebDavError::Protocol(format!("URL 解析失败: {}", err))
            }
            DavError::ReqwestDecode(err) => WebDavError::Protocol(err.to_string()),
            DavError::Decode(DecodeError::StatusMismatched(err)) => map_status_code(
                StatusCode::from_u16(err.response_code).unwrap_or(StatusCode::BAD_REQUEST),
            ),
            DavError::Decode(DecodeError::Server(err)) => map_status_code(
                StatusCode::from_u16(err.response_code).unwrap_or(StatusCode::BAD_REQUEST),
            ),
            DavError::Decode(DecodeError::FieldNotFound(err)) => {
                WebDavError::Protocol(format!("字段未找到: {}", err.field))
            }
            DavError::Decode(DecodeError::FieldNotSupported(err)) => {
                WebDavError::Protocol(format!("字段不支持: {}", err.field))
            }
            DavError::Decode(DecodeError::DigestAuth(err)) => {
                WebDavError::Authentication(format!("摘要认证失败: {}", err))
            }
            DavError::Decode(DecodeError::SerdeXml(err)) => {
                WebDavError::Protocol(format!("解析 XML 失败: {}", err))
            }
            DavError::Decode(DecodeError::NoAuthHeaderInResponse) => {
                WebDavError::Authentication("认证头缺失".to_string())
            }
            DavError::MissingAuthContext => {
                WebDavError::Authentication("缺少认证上下文".to_string())
            }
        }
    }
}

fn map_status_code(code: StatusCode) -> WebDavError {
    match code {
        StatusCode::UNAUTHORIZED => WebDavError::Authentication("未授权".to_string()),
        StatusCode::FORBIDDEN => WebDavError::Permission("禁止访问".to_string()),
        StatusCode::NOT_FOUND => WebDavError::NotFound("资源不存在".to_string()),
        StatusCode::CONFLICT => WebDavError::AlreadyExists,
        StatusCode::INSUFFICIENT_STORAGE => {
            WebDavError::InsufficientStorage("服务器存储不足".to_string())
        }
        StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => WebDavError::Timeout,
        _ if code.is_server_error() => WebDavError::Network(format!("服务器错误: {}", code)),
        _ => WebDavError::UnexpectedStatus(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use mockito::Server;

    fn build_client(host: String, base_path: &str) -> WebDAVClient {
        let agent = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let client = ClientBuilder::new()
            .set_agent(agent)
            .set_host(host)
            .set_auth(Auth::Anonymous)
            .build()
            .unwrap();
        WebDAVClient {
            client,
            encryptor: Encryptor::from_key(&[0u8; 32]),
            base_path: base_path.to_string(),
            retry_attempts: 0,
            retry_backoff: Duration::from_millis(1),
        }
    }

    fn sample_propfind_response() -> String {
        r#"<?xml version="1.0" encoding="utf-8"?>
        <D:multistatus xmlns:D="DAV:">
            <D:response>
                <D:href>/folder</D:href>
                <D:propstat>
                    <D:status>HTTP/1.1 200 OK</D:status>
                    <D:prop>
                        <D:getlastmodified>Wed, 10 Apr 2019 14:00:00 GMT</D:getlastmodified>
                        <D:resourcetype>
                            <D:collection/>
                        </D:resourcetype>
                        <D:getetag>"root-etag"</D:getetag>
                        <D:getcontenttype>httpd/unix-directory</D:getcontenttype>
                    </D:prop>
                </D:propstat>
            </D:response>
            <D:response>
                <D:href>/folder/latest.bin</D:href>
                <D:propstat>
                    <D:status>HTTP/1.1 200 OK</D:status>
                    <D:prop>
                        <D:getlastmodified>Thu, 11 Apr 2019 14:00:00 GMT</D:getlastmodified>
                        <D:resourcetype/>
                        <D:getetag>"latest-etag"</D:getetag>
                        <D:getcontenttype>application/octet-stream</D:getcontenttype>
                        <D:getcontentlength>5</D:getcontentlength>
                    </D:prop>
                </D:propstat>
            </D:response>
        </D:multistatus>
        "#
        .to_string()
    }

    #[tokio::test]
    async fn fetch_latest_file_uses_depth_one_and_decrypts_response() {
        let mut server = Server::new_async().await;
        let propfind_body = sample_propfind_response();
        let list_mock = server
            .mock("PROPFIND", "/folder")
            .match_header("depth", "1")
            .with_status(207)
            .with_header("content-type", "application/xml; charset=utf-8")
            .with_body(propfind_body)
            .create_async()
            .await;

        let key = [42u8; 32];
        let encryptor = Encryptor::from_key(&key);
        let payload = Payload::new_text(
            Bytes::from_static(b"hello"),
            "device".to_string(),
            Utc::now(),
        );
        let encrypted_body = encryptor.encrypt(payload.to_json().as_bytes()).unwrap();
        let get_mock = server
            .mock("GET", "/folder/latest.bin")
            .with_status(200)
            .with_body(encrypted_body)
            .create_async()
            .await;

        let mut client = build_client(server.url(), "/");
        client.encryptor = Encryptor::from_key(&key);

        let result = client
            .fetch_latest_file("folder".to_string())
            .await
            .expect("latest file should be fetched");

        list_mock.assert_async().await;
        get_mock.assert_async().await;
        assert_eq!(payload, result);
    }

    #[test]
    fn full_path_respects_segment_boundaries() {
        let client = build_client("http://example.com".to_string(), "/foo");
        let unrelated = client.full_path("foobar/file.bin");
        assert_eq!(unrelated, "/foo/foobar/file.bin");

        let already_prefixed = client.full_path("foo/bar.bin");
        assert_eq!(already_prefixed, "/foo/bar.bin");
    }
}
