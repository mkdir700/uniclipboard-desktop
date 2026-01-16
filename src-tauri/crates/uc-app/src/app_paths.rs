use std::path::PathBuf;

use uc_core::app_dirs::AppDirs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub db_path: PathBuf,
    pub vault_dir: PathBuf,
    pub settings_path: PathBuf,
    pub keyring_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn from_app_dirs(dirs: &AppDirs) -> Self {
        Self {
            db_path: dirs.app_data_root.join("uniclipboard.db"),
            vault_dir: dirs.app_data_root.join("vault"),
            settings_path: dirs.app_data_root.join("settings.json"),
            keyring_dir: dirs.app_data_root.join("keyring"),
            logs_dir: dirs.app_data_root.join("logs"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uc_core::app_dirs::AppDirs;

    #[test]
    fn app_paths_derives_concrete_locations_from_app_data_root() {
        let dirs = AppDirs {
            app_data_root: PathBuf::from("/tmp/uniclipboard"),
        };

        let paths = AppPaths::from_app_dirs(&dirs);

        assert_eq!(paths.db_path, PathBuf::from("/tmp/uniclipboard/uniclipboard.db"));
        assert_eq!(paths.vault_dir, PathBuf::from("/tmp/uniclipboard/vault"));
        assert_eq!(paths.settings_path, PathBuf::from("/tmp/uniclipboard/settings.json"));
        assert_eq!(paths.keyring_dir, PathBuf::from("/tmp/uniclipboard/keyring"));
        assert_eq!(paths.logs_dir, PathBuf::from("/tmp/uniclipboard/logs"));
    }
}
