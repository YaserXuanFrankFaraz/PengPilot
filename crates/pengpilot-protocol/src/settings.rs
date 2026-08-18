use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::computer_use::ComputerAppGrant;
use crate::model::ProviderKind;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct DaemonSettings {
    pub computer_use_enabled: bool,
    pub computer_use_allowed_apps: Vec<ComputerAppGrant>,
    pub disabled_providers: Vec<ProviderKind>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub provider_binary_overrides: HashMap<ProviderKind, String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            computer_use_enabled: false,
            computer_use_allowed_apps: Vec::new(),
            disabled_providers: Vec::new(),
            provider_binary_overrides: HashMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

impl DaemonSettings {
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join(".pengpilot")
            .join("settings.json")
    }

    pub fn discard_legacy_app_keys(&mut self) {
        for key in ["analytics_enabled", "favorite_models", "theme", "language"] {
            self.extra.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_settings_keep_snake_case_file_keys() {
        let json = serde_json::to_value(DaemonSettings {
            computer_use_enabled: true,
            ..DaemonSettings::default()
        })
        .unwrap();
        assert_eq!(json["computer_use_enabled"], true);
        assert!(json.get("computerUseEnabled").is_none());
    }
}
