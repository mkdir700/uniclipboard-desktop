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
    /// Creates a new DirsAppDirsAdapter with no base data directory override.
    ///
    /// # Examples
    ///
    /// ```
    /// let _ = DirsAppDirsAdapter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            base_data_local_dir_override: None,
        }
    }

    /// Creates a test-only adapter that overrides the base local data directory.
    ///
    /// The provided `base` path will be used instead of the system data local directory
    /// when resolving application directories for this adapter.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use crate::app_dirs::DirsAppDirsAdapter;
    ///
    /// let adapter = DirsAppDirsAdapter::with_base_data_local_dir(PathBuf::from("/tmp"));
    /// ```
    #[cfg(test)]
    pub fn with_base_data_local_dir(base: PathBuf) -> Self {
        Self {
            base_data_local_dir_override: Some(base),
        }
    }

    /// Resolve the base local data directory used for application data.
    ///
    /// Returns `Some(PathBuf)` containing the overridden base directory if one was set when the
    /// adapter was constructed; otherwise returns the system data-local directory from `dirs::data_local_dir()`.
    /// Returns `None` if no override is set and the system data-local directory is unavailable.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// // For tests you can construct an adapter with an explicit base directory:
    /// let adapter = DirsAppDirsAdapter::with_base_data_local_dir(PathBuf::from("/tmp"));
    /// assert_eq!(adapter.base_data_local_dir(), Some(PathBuf::from("/tmp")));
    /// ```
    fn base_data_local_dir(&self) -> Option<PathBuf> {
        if let Some(base) = &self.base_data_local_dir_override {
            return Some(base.clone());
        }
        dirs::data_local_dir()
    }
}

impl AppDirsPort for DirsAppDirsAdapter {
    /// Constructs the application's directories using the system (or overridden) local data directory.
    ///
    /// # Returns
    ///
    /// `AppDirs` with `app_data_root` set to the base local data directory joined with `"uniclipboard"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// let adapter = DirsAppDirsAdapter::with_base_data_local_dir(PathBuf::from("/tmp"));
    /// let dirs = adapter.get_app_dirs().unwrap();
    /// assert_eq!(dirs.app_data_root, PathBuf::from("/tmp/uniclipboard"));
    /// ```
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

    /// Verifies that the adapter appends the `uniclipboard` directory name to the base data directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let adapter = DirsAppDirsAdapter::with_base_data_local_dir(std::path::PathBuf::from("/tmp"));
    /// let dirs = adapter.get_app_dirs().unwrap();
    /// assert_eq!(dirs.app_data_root, std::path::PathBuf::from("/tmp/uniclipboard"));
    /// ```
    #[test]
    fn adapter_appends_uniclipboard_dir_name() {
        let adapter =
            DirsAppDirsAdapter::with_base_data_local_dir(std::path::PathBuf::from("/tmp"));
        let dirs = adapter.get_app_dirs().unwrap();
        assert_eq!(
            dirs.app_data_root,
            std::path::PathBuf::from("/tmp/uniclipboard")
        );
    }
}
