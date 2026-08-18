use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::ProviderKind;

pub const SKILL_FILE: &str = "SKILL.md";
pub const DISABLED_SKILL_FILE: &str = "SKILL.md.disabled";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillSource {
    Shared,
    Provider(ProviderKind),
}

impl SkillSource {
    pub fn label(self) -> String {
        match self {
            Self::Shared => tr!("skills.source_shared"),
            Self::Provider(provider) => provider.short_name().to_owned(),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Shared => "icons/package.svg",
            Self::Provider(provider) => provider_icon_path(provider),
        }
    }
}

fn provider_icon_path(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Amp => "icons/provider-amp.svg",
        ProviderKind::Claude => "icons/provider-claude.png",
        ProviderKind::Codex => "icons/provider-openai.png",
        ProviderKind::Cursor => "icons/provider-cursor.svg",
        ProviderKind::DeepSeek => "icons/provider-deepseek.png",
        ProviderKind::OpenCode => "icons/provider-opencode.png",
        ProviderKind::Grok => "icons/provider-grok.png",
        ProviderKind::Pi => "icons/provider-pi.svg",
        ProviderKind::Omp => "icons/provider-omp-color.svg",
        ProviderKind::Kiro => "icons/provider-kiro.png",
        ProviderKind::Hermes => "icons/provider-hermes.png",
        ProviderKind::Trae => "icons/provider-trae.png",
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillScope {
    User,
    Project,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillLocation {
    pub source: SkillSource,
    pub scope: SkillScope,
    pub root: PathBuf,
    pub project: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstall {
    pub source: SkillSource,
    pub dir: PathBuf,
    pub skill_file: PathBuf,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub scope: SkillScope,
    pub project: Option<String>,
    pub installs: Vec<SkillInstall>,
    pub enabled: bool,
    pub allowed_tools: Option<String>,
    pub body: String,
    pub supporting_files: usize,
    pub total_bytes: u64,
    pub modified_at: Option<u64>,
    pub duplicates: usize,
    pub row_key: u64,
}

impl SkillEntry {
    pub fn primary(&self) -> &SkillInstall {
        &self.installs[0]
    }

    pub fn icon(&self) -> &'static str {
        if self.installs.len() > 1 {
            "icons/package.svg"
        } else {
            self.primary().source.icon()
        }
    }

    pub fn sources_label(&self) -> String {
        self.installs
            .iter()
            .map(|install| install.source.label())
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsCatalog {
    pub skills: Vec<SkillEntry>,
}

impl SkillsCatalog {
    pub fn disabled_count(&self) -> usize {
        self.skills.iter().filter(|skill| !skill.enabled).count()
    }
}
