use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Error;
pub mod slack;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Film {
    pub id: Uuid,
    pub name: String,
    pub priority: Priority,
    pub roles: Roles,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Priority {
    HIGH,
    LOW,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
/// A post-production role. Students must work each role sequentially.
pub struct Roles {
    pub ae: Option<DateTime<Utc>>,
    pub editor: Option<DateTime<Utc>>,
    pub sound: Option<DateTime<Utc>>,
    pub color: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Role {
    Ae,
    Editor,
    Sound,
    Color,
}

impl Film {
    pub fn new(id: Uuid, name: String, priority: Priority, roles: Roles) -> Self {
        Film {
            id,
            name,
            priority,
            roles,
        }
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
            priority: Priority::HIGH,
            roles: Roles::default(),
        }
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Priority::LOW => write!(f, "LOW"),
            &Self::HIGH => write!(f, "HIGH"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HIGH" => Ok(Priority::HIGH),
            "LOW" => Ok(Priority::LOW),
            _ => Err(Error::InvalidArg("not a valid priority".into())),
        }
    }
}

impl Roles {
    pub fn new(
        ae: Option<DateTime<Utc>>,
        editor: Option<DateTime<Utc>>,
        sound: Option<DateTime<Utc>>,
        color: Option<DateTime<Utc>>,
    ) -> Roles {
        Roles {
            ae,
            editor,
            sound,
            color,
        }
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
