use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Role, Roles};

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
