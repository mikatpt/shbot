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
    pub group_number: i32,
}

impl Film {
    pub fn new(name: &str, priority: Priority, group_number: i32) -> Self {
        Film {
            name: name.to_string(),
            priority,
            group_number,
            ..Default::default()
        }
    }

    /// Increments role and returns it.
    pub fn increment_role(&mut self) -> Role {
        self.roles
            .complete_role(self.current_role, self.name.clone());
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
            group_number: 0,
        }
    }
}
