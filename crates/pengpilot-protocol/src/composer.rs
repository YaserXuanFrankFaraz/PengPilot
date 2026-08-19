use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CommandScope {
    Project,
    User,
    Skill,
    Builtin,
}

impl CommandScope {
    pub const fn display_rank(self) -> u8 {
        match self {
            Self::Builtin => 0,
            Self::Project => 1,
            Self::User => 2,
            Self::Skill => 3,
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Project => tr!("command_scope.project"),
            Self::User => tr!("command_scope.user"),
            Self::Skill => tr!("command_scope.skill"),
            Self::Builtin => tr!("command_scope.builtin"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub scope: CommandScope,
    pub argument_hint: Option<String>,
    pub template: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub is_dir: bool,
}
