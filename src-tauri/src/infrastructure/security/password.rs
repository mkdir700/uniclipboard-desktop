use crate::config::get_config_dir;
use anyhow::{anyhow, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use iota_stronghold::types::Client;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri_plugin_stronghold::stronghold::Stronghold;
use tokio::sync::mpsc;

/// 密码管理器，用于处理密码的哈希和验证
pub struct PasswordManager {
    stronghold: Stronghold,
    client: Client,
}

// 全局单例实例
static INSTANCE: Lazy<Mutex<PasswordManager>> =
    Lazy::new(|| Mutex::new(PasswordManager::new_internal()));

// 异步操作请求类型
pub enum PasswordRequest {
    GetRecord(String, mpsc::Sender<Result<Option<String>>>),
    InsertRecord(String, String, mpsc::Sender<Result<()>>),
    DeleteRecord(String, mpsc::Sender<Result<()>>),
    GetEncryptionPassword(mpsc::Sender<Result<Option<String>>>),
    SetEncryptionPassword(String, mpsc::Sender<Result<()>>),
    DeleteEncryptionPassword(mpsc::Sender<Result<()>>),
}

// 全局异步操作通道
pub static PASSWORD_SENDER: Lazy<Mutex<Option<mpsc::Sender<PasswordRequest>>>> = Lazy::new(|| {
    // 创建通道
    let (tx, mut rx) = mpsc::channel::<PasswordRequest>(100);

    // 启动后台工作线程
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            while let Some(request) = rx.recv().await {
                // 获取密码管理器实例
                let pm = PasswordManager::get_instance().lock().unwrap();

                match request {
                    PasswordRequest::GetRecord(key, responder) => {
                        let result = pm.get_record_internal(key);
                        let _ = responder.send(result).await;
                    }
                    PasswordRequest::InsertRecord(key, value, responder) => {
                        let result = pm.insert_record_internal(key, value);
                        let _ = responder.send(result).await;
                    }
                    PasswordRequest::DeleteRecord(key, responder) => {
                        let result = pm.delete_record_internal(key);
                        let _ = responder.send(result).await;
                    }
                    PasswordRequest::GetEncryptionPassword(responder) => {
                        let key = "encryption_password".to_string();
                        let result = pm.get_record_internal(key);
                        let _ = responder.send(result).await;
                    }
                    PasswordRequest::SetEncryptionPassword(password, responder) => {
                        let key = "encryption_password".to_string();
                        let result = pm.insert_record_internal(key, password);
                        let _ = responder.send(result).await;
                    }
                    PasswordRequest::DeleteEncryptionPassword(responder) => {
                        let key = "encryption_password".to_string();
                        let result = pm.delete_record_internal(key);
                        let _ = responder.send(result).await;
                    }
                }
            }
        });
    });

    Mutex::new(Some(tx))
});

impl PasswordManager {
    /// 获取PasswordManager的单例实例
    pub fn get_instance() -> &'static Mutex<PasswordManager> {
        &INSTANCE
    }

    // 内部构造函数，只在初始化单例时使用
    fn new_internal() -> Self {
        let vault_password = Self::init_vault_password()
            .unwrap_or_else(|e| {
                log::error!("Failed to initialize vault password: {}", e);
                // 降级：生成临时密码（避免应用崩溃）
                Self::generate_vault_password()
            });

        let stronghold = Stronghold::new(
            Self::get_snapshot_path(),
            vault_password.into()
        ).unwrap_or_else(|e| {
            panic!("Failed to initialize Stronghold: {}", e);
        });

        // 如果加载失败，捕获错误并创建 client
        let client = stronghold
            .load_client("uniclipboard")
            .unwrap_or_else(|_| stronghold.create_client("uniclipboard").unwrap());
        Self { stronghold, client }
    }

    /// 获取Stronghold数据文件路径
    ///
    /// Returns:
    ///
    /// - 如果获取到配置目录，则返回配置目录下的 `uniclipboard.stronghold` 文件路径
    /// - 如果获取不到配置目录，则返回错误
    pub fn get_snapshot_path() -> PathBuf {
        get_config_dir()
            .expect("Could not find config directory")
            .join("uniclipboard.stronghold")
    }

    /// 获取密码盐文件路径
    ///
    /// Returns:
    ///
    /// - 如果获取到配置目录，则返回配置目录下的 `.salt` 文件路径
    /// - 如果获取不到配置目录，则返回错误
    pub fn get_salt_file_path() -> PathBuf {
        get_config_dir()
            .expect("Could not find config directory")
            .join(".salt")
    }

    /// 初始化密码盐文件
    ///
    /// 如果密码盐文件不存在，则生成并写入
    pub fn init_salt_file_if_not_exists() -> Result<()> {
        let salt_file_path = Self::get_salt_file_path();
        if !salt_file_path.exists() {
            let salt = Self::generate_salt();
            std::fs::write(salt_file_path, salt)?;
        }
        Ok(())
    }

    /// 生成盐值用于密钥派生
    ///
    /// # 返回
    /// * `String` - 生成的盐值字符串表示
    pub fn generate_salt() -> String {
        SaltString::generate(&mut OsRng).to_string()
    }

    /// 获取 vault 密钥文件路径
    pub fn get_vault_key_path() -> PathBuf {
        get_config_dir()
            .expect("Could not find config directory")
            .join(".vault_key")
    }

    /// 检查加密密码是否存在（不触发单例初始化）
    ///
    /// 此方法仅检查文件系统状态，不会访问 INSTANCE 单例
    /// 用于 onboarding 流程中的轻量级状态检查
    pub fn has_encryption_password() -> bool {
        let snapshot_path = Self::get_snapshot_path();
        let vault_key_path = Self::get_vault_key_path();

        // 两个文件都必须存在才表示 vault 已正确初始化
        snapshot_path.exists() && vault_key_path.exists()
    }

    /// 生成随机 vault 密码
    fn generate_vault_password() -> String {
        use rand::Rng;
        const VAULT_PASSWORD_LENGTH: usize = 32;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";
        let mut rng = rand::rng();
        (0..VAULT_PASSWORD_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// 初始化或加载 vault 密码
    fn init_vault_password() -> Result<String> {
        let vault_key_path = Self::get_vault_key_path();
        let snapshot_path = Self::get_snapshot_path();

        // 检查状态一致性
        let vault_exists = vault_key_path.exists();
        let snapshot_exists = snapshot_path.exists();

        if vault_exists && snapshot_exists {
            // 两个文件都存在 - 读取已存在的 vault 密码
            return std::fs::read_to_string(&vault_key_path)
                .map_err(|e| anyhow!("Failed to read vault key: {}", e));
        }

        if !vault_exists && !snapshot_exists {
            // 两个文件都不存在 - 生成新的 vault 密码
            let vault_password = Self::generate_vault_password();
            std::fs::write(&vault_key_path, &vault_password)
                .map_err(|e| anyhow!("Failed to write vault key: {}", e))?;

            // 设置文件权限（Unix: 仅用户可读写）
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&vault_key_path)?.permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&vault_key_path, perms)?;
            }

            return Ok(vault_password);
        }

        // 状态不一致 - 一个存在另一个不存在
        Err(anyhow!(
            "Vault state inconsistent: vault_key={}, snapshot={}. \
             Please delete both files to reset: {:?} and {:?}",
            vault_exists,
            snapshot_exists,
            vault_key_path,
            snapshot_path
        ))
    }

    /// 检查 vault 密钥是否已存在
    pub fn vault_key_exists() -> bool {
        Self::get_vault_key_path().exists()
    }

    /// 获取Stronghold数据文件路径
    pub fn get_stronghold_path() -> PathBuf {
        get_config_dir()
            .expect("Could not find config directory")
            .join("uniclipboard.stronghold")
    }

    // 内部同步方法，只在后台线程中使用
    fn insert_record_internal(&self, key: String, value: String) -> Result<()> {
        self.client.store().insert(key.into(), value.into(), None)?;
        self.stronghold.save()?;
        Ok(())
    }

    // 内部同步方法，只在后台线程中使用
    fn get_record_internal(&self, key: String) -> Result<Option<String>> {
        let client = self.client.clone();
        let value = client.store().get(key.as_bytes())?;

        match value {
            Some(bytes) => {
                let string =
                    String::from_utf8(bytes).map_err(|e| anyhow!("无法解析UTF-8字符串: {}", e))?;
                Ok(Some(string))
            }
            None => Ok(None),
        }
    }

    // 内部同步方法，只在后台线程中使用
    fn delete_record_internal(&self, key: String) -> Result<()> {
        self.client.store().delete(key.as_bytes())?;
        self.stronghold.save()?;
        Ok(())
    }

    /// 插入记录（异步版本）
    pub async fn insert_record(&self, key: String, value: String) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::InsertRecord(key, value, tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }

    /// 获取记录（异步版本）
    pub async fn get_record(&self, key: String) -> Result<Option<String>> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::GetRecord(key, tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }

    /// 删除记录（异步版本）
    pub async fn delete_record(&self, key: String) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::DeleteRecord(key, tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }

    /// 获取加密口令（异步版本）
    pub async fn get_encryption_password(&self) -> Result<Option<String>> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::GetEncryptionPassword(tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }

    /// 设置加密口令（异步版本）
    pub async fn set_encryption_password(&self, password: String) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::SetEncryptionPassword(password, tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }

    /// 清除加密口令（异步版本）
    pub async fn delete_encryption_password(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(1);

        // 发送请求到工作线程
        if let Some(sender) = PASSWORD_SENDER.lock().unwrap().as_ref() {
            sender
                .send(PasswordRequest::DeleteEncryptionPassword(tx))
                .await
                .map_err(|_| anyhow!("无法发送密码操作请求"))?;

            // 等待结果
            rx.recv().await.ok_or_else(|| anyhow!("工作线程已关闭"))?
        } else {
            Err(anyhow!("密码通道未初始化"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_salt_file_if_not_exists() {
        PasswordManager::init_salt_file_if_not_exists().unwrap();
        assert!(PasswordManager::get_salt_file_path().exists());
        std::fs::remove_file(PasswordManager::get_salt_file_path()).unwrap();
    }

    #[test]
    fn test_generate_salt() {
        // 生成的盐值不应为空
        let salt = PasswordManager::generate_salt();
        assert!(!salt.is_empty());

        // 多次生成的盐值应该不同
        let salt2 = PasswordManager::generate_salt();
        assert_ne!(salt, salt2, "两次生成的盐值不应相同");
    }

    // TODO: running for over 60s
    // #[tokio::test]
    // async fn test_set_and_get_encryption_password() {
    //     // 使用单例模式获取实例
    //     let manager = PasswordManager::get_instance().lock().unwrap();
    //     let test_password = "test_password".to_string();

    //     // 测试设置加密口令
    //     manager
    //         .set_encryption_password(test_password.clone())
    //         .await
    //         .unwrap();

    //     // 测试获取加密口令
    //     let result = manager.get_encryption_password().await.unwrap();
    //     assert_eq!(result, Some(test_password));

    //     // 测试清除加密口令
    //     manager.delete_encryption_password().await.unwrap();
    //     let result = manager.get_encryption_password().await.unwrap();
    //     assert_eq!(result, None);
    // }
}
