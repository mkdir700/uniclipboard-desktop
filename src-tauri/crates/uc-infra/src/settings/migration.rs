use uc_core::ports::SettingsMigrationPort;
use uc_core::settings::model::{Settings, CURRENT_SCHEMA_VERSION};

pub struct SettingsMigrator {
    migrations: Vec<Box<dyn SettingsMigrationPort>>,
}

impl SettingsMigrator {
    /// Creates a new `SettingsMigrator` with its migrations list initialized.
    ///
    /// The migrations vector is currently empty with placeholder comments where future
    /// migrations can be added.
    ///
    /// # Examples
    ///
    /// ```
    /// let migrator = SettingsMigrator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            migrations: vec![
                // Box::new(MigrationV1ToV2),
                // Box::new(MigrationV2ToV3),
            ],
        }
    }

    /// Migrates a Settings instance forward until its schema_version equals CURRENT_SCHEMA_VERSION.
    ///
    /// Applies successive migrations from the migrator's migration list starting at the settings'
    /// current `schema_version`. Panics if no migration is available for the next required version.
    ///
    /// # Returns
    ///
    /// The `Settings` value after applying all required migrations, updated to `CURRENT_SCHEMA_VERSION`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let migrator = SettingsMigrator::new();
    /// // `settings` should be obtained from storage or created by the caller.
    /// let settings = /* obtain Settings */;
    /// let migrated = migrator.migrate_to_latest(settings);
    /// ```
    pub fn migrate_to_latest(&self, mut settings: Settings) -> Settings {
        loop {
            let current = settings.schema_version;

            if current >= CURRENT_SCHEMA_VERSION {
                break;
            }

            let migration = self
                .migrations
                .iter()
                .find(|m| m.from_version() == current)
                .unwrap_or_else(|| panic!("no migration found from version {}", current));

            settings = migration.migrate(settings);
        }

        settings
    }
}