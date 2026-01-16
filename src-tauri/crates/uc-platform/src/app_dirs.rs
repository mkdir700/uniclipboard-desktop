use std::path::PathBuf;

use uc_core::{
    app_dirs::AppDirs,
    ports::{AppDirsError, AppDirsPort},
};

const APP_DIR_NAME: &str = "uniclipboard";

pub struct DirsAppDirsAdapter {
    base_data_local_dir_override: Option<PathBuf>,
}

impl DirsAppDirsAdapter {
    pub fn new() -> Self {
        Self {
            base_data_local_dir_override: None,
        }
    }

    #[cfg(test)]
    pub fn with_base_data_local_dir(base: PathBuf) -> Self {
        Self {
            base_data_local_dir_override: Some(base),
        }
    }

    fn base_data_local_dir(&self) -> Option<PathBuf> {
        if let Some(base) = &self.base_data_local_dir_override {
            return Some(base.clone());
        }
        dirs::data_local_dir()
    }
}

impl AppDirsPort for DirsAppDirsAdapter {
    fn get_app_dirs(&self) -> Result<AppDirs, AppDirsError> {
        let base = self
            .base_data_local_dir()
            .ok_or(AppDirsError::DataLocalDirUnavailable)?;

        Ok(AppDirs {
            app_data_root: base.join(APP_DIR_NAME),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ports::AppDirsPort;

    #[test]
    fn adapter_appends_uniclipboard_dir_name() {
        let adapter = DirsAppDirsAdapter::with_base_data_local_dir(std::path::PathBuf::from("/tmp"));
        let dirs = adapter.get_app_dirs().unwrap();
        assert_eq!(
            dirs.app_data_root,
            std::path::PathBuf::from("/tmp/uniclipboard")
        );
    }
}
