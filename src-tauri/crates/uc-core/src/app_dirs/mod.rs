use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    pub app_data_root: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn app_dirs_is_pure_fact_container() {
        let dirs = AppDirs {
            app_data_root: PathBuf::from("/tmp/uniclipboard"),
        };
        assert!(dirs.app_data_root.ends_with("uniclipboard"));
    }
}
