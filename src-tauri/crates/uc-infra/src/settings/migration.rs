use uc_core::ports::SettingsMigrationPort;
use uc_core::settings::model::{Settings, CURRENT_SCHEMA_VERSION};

pub struct SettingsMigrator {
    migrations: Vec<Box<dyn SettingsMigrationPort>>,
}

impl SettingsMigrator {
    pub fn new() -> Self {
        Self {
            migrations: vec![
                // Box::new(MigrationV1ToV2),
                // Box::new(MigrationV2ToV3),
            ],
        }
    }

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
