use std::time::Duration;

use super::model::*;

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            silent_start: false,
            auto_check_update: true,
            theme: Theme::System,
            theme_color: None,
            device_name: None,
            language: None,
        }
    }
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_frequency: SyncFrequency::Realtime,
            content_types: ContentTypes::default(),
            max_file_size_mb: 100,
        }
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            skip_pinned: true,
            evaluation: RuleEvaluation::AnyMatch,
            rules: vec![
                RetentionRule::ByAge {
                    max_age: Duration::from_secs(60 * 60 * 24 * 30), // 30 days
                },
                RetentionRule::ByCount { max_items: 500 },
            ],
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            encryption_enabled: false,
            passphrase_configured: false,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralSettings::default(),
            sync: SyncSettings::default(),
            retention_policy: RetentionPolicy::default(),
            security: SecuritySettings::default(),
        }
    }
}
