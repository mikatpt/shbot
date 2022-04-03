#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Film {
    pub id: Uuid,
    pub name: String,
    pub current_role: Role,
    pub priority: Priority,
    pub roles: Roles,
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Student {
    pub id: Uuid,
    pub slack_id: String,
    pub name: String,
    pub current_film: Option<String>,
    pub current_role: Role,
    pub roles: Roles,
}

impl Student {
    /// Increments role and returns it.
    pub fn increment_role(&mut self) -> Role {
        self.roles.complete_role(self.current_role);
        self.current_role = self.roles.get_next_role();
        self.current_role
    }

    pub fn get_next_role(&self) -> Role {
        self.roles.get_next_role()
    }
}

impl Default for Student {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            slack_id: "".to_string(),
            name: "".to_string(),
            current_film: None,
            current_role: Role::Ae,
            roles: Roles::default(),
        }
    }
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

    /// Increments role and returns it.
    pub fn increment_role(&mut self) -> Role {
        self.roles.complete_role(self.current_role);
        self.current_role = self.roles.get_next_role();
        self.current_role
    }

    pub fn get_next_role(&self) -> Role {
        self.roles.get_next_role()
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

#[test]
fn testing() {
    let a = Role::Ae;
    let b = Role::Editor;
    let c = a < b;
    dbg!(c);
}
