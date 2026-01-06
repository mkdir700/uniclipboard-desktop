use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json;
use std::path::{Path, PathBuf};
use tokio::fs;
use uc_core::{
    ports::SettingsPort,
    settings::model::{Settings, CURRENT_SCHEMA_VERSION},
};

use crate::settings::migration::SettingsMigrator;

pub struct FileSettingsRepository {
    path: PathBuf,
}

impl FileSettingsRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn dir(&self) -> Option<&Path> {
        self.path.parent()
    }

    async fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(dir) = self.dir() {
            fs::create_dir_all(dir)
                .await
                .with_context(|| format!("create settings dir failed: {}", dir.display()))?;
        }
        Ok(())
    }

    async fn atomic_write(&self, content: &str) -> Result<()> {
        self.ensure_parent_dir().await?;

        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, content)
            .await
            .with_context(|| format!("write temp settings failed: {}", tmp_path.display()))?;

        // TODO: Windows 上 rename 覆盖可能不一致；macOS/Linux OK。
        fs::rename(&tmp_path, &self.path).await.with_context(|| {
            format!(
                "rename temp settings to target failed: {} -> {}",
                tmp_path.display(),
                self.path.display()
            )
        })?;

        Ok(())
    }
}

#[async_trait]
impl SettingsPort for FileSettingsRepository {
    async fn load(&self) -> Result<Settings> {
        let content = match fs::read_to_string(&self.path).await {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Settings::default());
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("read settings failed: {}", self.path.display()))
            }
        };

        let settings: Settings = serde_json::from_str(&content)?;
        let migrator = SettingsMigrator::new();
        let migrated = migrator.migrate_to_latest(settings);

        if migrated.schema_version < CURRENT_SCHEMA_VERSION {
            self.save(&migrated).await?;
        }

        Ok(migrated)
    }

    async fn save(&self, settings: &Settings) -> Result<()> {
        let content =
            serde_json::to_string_pretty(settings).context("serialize settings failed")?;

        self.atomic_write(&content).await
    }
}
