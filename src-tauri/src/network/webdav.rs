use crate::encrypt::Encryptor;
use crate::file_metadata::FileMetadata;
use crate::message::Payload;
use crate::utils::string_to_32_bytes;
use anyhow::Result;
use reqwest_dav::{list_cmd::ListEntity, Auth, ClientBuilder, Depth};

pub struct WebDAVClient {
    client: reqwest_dav::Client,
    encryptor: Encryptor,
}

impl WebDAVClient {
    #[allow(dead_code)]
    pub async fn new(webdav_url: String, username: String, password: String) -> Result<Self> {
        let key = string_to_32_bytes(&password);
        let encryptor = Encryptor::from_key(&key);
        let client = ClientBuilder::new()
            .set_host(webdav_url)
            .set_auth(Auth::Basic(username, password))
            .build()?;
        Ok(Self { client, encryptor })
    }

    /// 检查是否连接到 WebDAV 服务器
    #[allow(dead_code)]
    pub async fn is_connected(&self) -> bool {
        self.client.list("/", Depth::Number(0)).await.is_ok()
    }

    /// Initializes the share directory on the WebDAV server.
    ///
    /// This method attempts to create a new directory at the specified base path
    /// on the WebDAV server. This directory will be used for storing shared files.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is `Ok(())` if the directory is successfully created,
    /// or an `Error` if the operation fails.
    #[allow(dead_code)]
    async fn initialize_share_directory(&self, dir: String) -> anyhow::Result<()> {
        self.client.mkcol(&dir).await?;
        Ok(())
    }

    /// Creates a new directory on the WebDAV server for sharing.
    ///
    /// This method attempts to create a new directory at the specified base path
    /// on the WebDAV server. This directory will be used for storing shared files.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is `Ok(())` if the directory is successfully created,
    /// or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There's a failure in communicating with the WebDAV server
    /// - The directory already exists
    /// - The client doesn't have sufficient permissions to create the directory
    #[allow(dead_code)]
    pub async fn is_share_code_exists(&self) -> bool {
        match self.client.list("/uniclipboard", Depth::Number(0)).await {
            Ok(entries) => !entries.is_empty(),
            Err(_) => false,
        }
    }

    /// Uploads a Payload to the specified directory on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `dir` - A String representing the directory path to upload the Payload to.
    /// * `payload` - A Payload to be uploaded.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the path of the uploaded file.
    pub async fn upload(&self, dir: String, payload: Payload) -> Result<String> {
        let filename = format!("{}_{}.bin", payload.get_device_id(), payload.get_key());
        let path: String;
        if dir == "/" {
            path = format!("/{}", filename);
        } else {
            path = format!("{}/{}", dir, filename);
        }
        let json_payload = payload.to_json();
        let encrypted_payload = self.encryptor.encrypt(&json_payload.as_bytes())?;
        self.client.put(&path, encrypted_payload).await?;
        Ok(path)
    }

    /// Downloads a Payload from the specified path on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `path` - A String representing the path to download the Payload from.
    ///
    /// # Returns   
    ///
    /// Returns a Result containing the Payload.
    pub async fn download(&self, path: String) -> Result<Payload> {
        let response = self.client.get(&path).await?;
        if response.status().is_success() {
            let content = response.bytes().await?;
            let decrypted_payload = self.encryptor.decrypt(&content)?;
            let payload = serde_json::from_slice(&decrypted_payload)?;
            Ok(payload)
        } else {
            Err(anyhow::anyhow!("Failed to download file"))
        }
    }

    /// Counts the number of files in the specified directory on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `path` - A String representing the directory path to count files in.
    ///
    /// # Returns
    pub async fn count_files(&self, path: String) -> Result<usize> {
        let entries = self.client.list(&path, Depth::Number(1)).await?;
        Ok(entries.len().saturating_sub(1))
    }

    /// Fetches the latest file from the specified directory on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `dir` - A String representing the directory path to search for files.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Payload of the latest file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The WebDAV list operation fails.
    /// * No files are found in the specified directory.
    /// * The latest file cannot be retrieved or deserialized.
    #[allow(dead_code)]
    pub async fn fetch_latest_file(&self, dir: String) -> Result<Payload> {
        let entries = self.client.list(&dir, Depth::Number(0)).await?;
        let latest_file = entries
            .iter()
            .map(|entity| match entity {
                ListEntity::File(file) => file,
                _ => panic!("Not a file"),
            })
            .max_by_key(|file| file.last_modified);

        let response = self.client.get(&latest_file.unwrap().href).await?;
        if response.status().is_success() {
            let content = response.bytes().await?;
            let payload = serde_json::from_slice(&content)?;
            Ok(payload)
        } else {
            Err(anyhow::anyhow!("Failed to fetch latest file"))
        }
    }

    /// Fetches the metadata of the latest file from the specified directory on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `dir` - A String representing the directory path to search for files.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the metadata of the latest file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The WebDAV list operation fails.
    /// * No files are found in the specified directory.
    pub async fn fetch_latest_file_meta(&self, dir: String) -> Result<FileMetadata> {
        let entries = self.client.list(&dir, Depth::Number(1)).await?;
        let list_file = entries
            .iter()
            .filter_map(|entity| match entity {
                ListEntity::File(file) => Some(file),
                _ => None,
            })
            .max_by_key(|file| file.last_modified)
            .ok_or_else(|| anyhow::anyhow!("No files found"))?;

        let meta = FileMetadata::from_list_file(&list_file, &self.client.host);
        Ok(meta)
    }

    /// Fetches the metadata of the oldest file from the specified directory on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `dir` - A String representing the directory path to search for files.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the metadata of the oldest file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The WebDAV list operation fails.
    /// * No files are found in the specified directory.
    pub async fn fetch_oldest_file_meta(&self, dir: String) -> Result<FileMetadata> {
        let entries = self.client.list(&dir, Depth::Number(1)).await?;
        let list_file = entries
            .iter()
            .filter_map(|entity| match entity {
                ListEntity::File(file) => Some(file),
                _ => None,
            })
            .min_by_key(|file| file.last_modified)
            .ok_or_else(|| anyhow::anyhow!("No files found"))?;

        let meta = FileMetadata::from_list_file(&list_file, &self.client.host);
        Ok(meta)
    }

    /// Deletes a file from the specified path on the WebDAV server.
    ///
    /// # Arguments
    ///
    /// * `path` - A String representing the path to delete the file from.
    ///
    /// # Returns
    ///
    /// Returns a Result containing `Ok(())` if the file is successfully deleted,
    /// or an `Error` if the operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The WebDAV delete operation fails.
    #[allow(dead_code)]
    pub async fn delete(&self, path: String) -> Result<()> {
        self.client.delete(&path).await?;
        Ok(())
    }
}
