use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, Deserialize, Serialize)]
#[strum(serialize_all = "mixed_case", ascii_case_insensitive)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Low,
    High,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
/// A post-production role. Students must work each role sequentially.
/// These mark completion time!
pub struct Roles {
    pub ae: Option<DateTime<Utc>>,
    pub editor: Option<DateTime<Utc>>,
    pub sound: Option<DateTime<Utc>>,
    pub color: Option<DateTime<Utc>>,
}

impl Roles {
    #[rustfmt::skip]
    pub fn new(
        ae: Option<DateTime<Utc>>,
        editor: Option<DateTime<Utc>>,
        sound: Option<DateTime<Utc>>,
        color: Option<DateTime<Utc>>,
    ) -> Roles {
        Roles { ae, editor, sound, color }
    }

    pub fn get_next_role(&self) -> Role {
        if self.ae.is_none() {
            Role::Ae
        } else if self.editor.is_none() {
            Role::Editor
        } else if self.sound.is_none() {
            Role::Sound
        } else if self.color.is_none() {
            Role::Color
        } else {
            Role::Done
        }
    }

    pub fn complete_role(&mut self, role: Role) {
        let now = Some(Utc::now());
        match role {
            Role::Ae => self.ae = now,
            Role::Editor => self.editor = now,
            Role::Sound => self.sound = now,
            Role::Color => self.color = now,
            Role::Done => {}
        }
    }
}

#[derive(AsRefStr, EnumString, Debug, Clone, Copy)]
#[strum(serialize_all = "mixed_case", ascii_case_insensitive)]
#[derive(Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    Ae,
    Editor,
    Sound,
    Color,
    Done,
}
