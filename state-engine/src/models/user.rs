//! User identity model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// User identity representing an agent or human user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(flatten)]
    pub base: EntityBase,
    pub username: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub metadata: serde_json::Value,
    pub active: bool,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            base: EntityBase::new(),
            username,
            display_name: None,
            email: None,
            metadata: serde_json::json!({}),
            active: true,
        }
    }

    pub fn with_display_name(mut self, display_name: String) -> Self {
        self.display_name = Some(display_name);
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
}

impl Entity for User {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "user"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}
