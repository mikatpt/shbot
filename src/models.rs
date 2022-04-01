#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Film {
    pub id: Uuid,
    pub name: String,
    pub current_role: Role,
    pub priority: Priority,
    pub roles: Roles,
}

#[derive(Debug, Clone, Copy, AsRefStr, EnumString, Deserialize, Serialize)]
#[strum(serialize_all = "mixed_case")]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    High,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
/// A post-production role. Students must work each role sequentially.
pub struct Roles {
    pub ae: Option<DateTime<Utc>>,
    pub editor: Option<DateTime<Utc>>,
    pub sound: Option<DateTime<Utc>>,
    pub color: Option<DateTime<Utc>>,
}

#[derive(AsRefStr, EnumString, Deserialize, Serialize, Debug, Clone, Copy)]
#[strum(serialize_all = "mixed_case")]
#[derive(PartialEq, Eq)]
pub enum Role {
    Ae,
    Editor,
    Sound,
    Color,
}

impl Film {
    #[rustfmt::skip]
    pub fn new(
        id: Uuid,
        name: String,
        current_role: Role,
        priority: Priority,
        roles: Roles,
    ) -> Self {
        Film { id, name, current_role, priority, roles }
    }

    pub fn add_next_role(&mut self) -> bool {
        self.roles.add_next_role()
    }
}

impl Default for Film {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".to_string(),
            priority: Priority::High,
            current_role: Role::Ae,
            roles: Roles::default(),
        }
    }
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

    /// Returns false if all roles have been worked.
    pub fn add_next_role(&mut self) -> bool {
        let next_role = if self.ae.is_none() {
            Role::Ae
        } else if self.editor.is_none() {
            Role::Editor
        } else if self.sound.is_none() {
            Role::Sound
        } else if self.color.is_none() {
            Role::Color
        } else {
            return false;
        };

        self.add_role(next_role);
        true
    }

    fn add_role(&mut self, role: Role) {
        let now = Some(Utc::now());
        match role {
            Role::Ae => self.ae = now,
            Role::Editor => self.editor = now,
            Role::Sound => self.sound = now,
            Role::Color => self.color = now,
        }
    }
}
