use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, Deserialize, Serialize)]
#[strum(serialize_all = "UPPERCASE")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum Priority {
    Low,
    High,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
/// A post-production role. Students must work each role sequentially.
pub struct Roles {
    pub ae: Option<String>,
    pub editor: Option<String>,
    pub sound: Option<String>,
    pub finish: Option<String>,
}

#[derive(AsRefStr, EnumString, Debug, Clone, Copy)]
#[strum(serialize_all = "UPPERCASE")]
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Ae,
    Editor,
    Sound,
    Finish,
    Done,
}

impl Default for Priority {
    fn default() -> Self {
        Self::High
    }
}

impl Default for Role {
    fn default() -> Self {
        Self::Ae
    }
}

impl Roles {
    #[rustfmt::skip]
    pub fn new(
        ae: Option<String>,
        editor: Option<String>,
        sound: Option<String>,
        finish: Option<String>,
    ) -> Roles {
        Roles { ae, editor, sound, finish }
    }

    pub fn get_next_role(&self) -> Role {
        if self.ae.is_none() {
            Role::Ae
        } else if self.editor.is_none() {
            Role::Editor
        } else if self.sound.is_none() {
            Role::Sound
        } else if self.finish.is_none() {
            Role::Finish
        } else {
            Role::Done
        }
    }

    pub fn complete_role(&mut self, role: Role, film: String) {
        match role {
            Role::Ae => self.ae = Some(film),
            Role::Editor => self.editor = Some(film),
            Role::Sound => self.sound = Some(film),
            Role::Finish => self.finish = Some(film),
            Role::Done => {}
        }
    }
}
