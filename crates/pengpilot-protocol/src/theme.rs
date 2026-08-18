//! Process-neutral theme preference persisted in the desktop settings file.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> String {
        match self {
            Self::System => tr!("settings.theme_system"),
            Self::Light => tr!("settings.theme_light"),
            Self::Dark => tr!("settings.theme_dark"),
        }
    }
}
