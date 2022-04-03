#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Priority, Role, Roles};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Film {
    pub id: Uuid,
    pub name: String,
    pub current_role: Role,
    pub priority: Priority,
    pub roles: Roles,
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
