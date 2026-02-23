//! Session model for tracking agent sessions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Failed,
    Abandoned,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// A session represents a single agent interaction or task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(flatten)]
    pub base: EntityBase,
    pub user_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub status: SessionStatus,
    pub metadata: serde_json::Value,
    pub context: serde_json::Value,
    pub last_activity_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            base: EntityBase::new(),
            user_id,
            channel_id: None,
            status: SessionStatus::Active,
            metadata: serde_json::json!({}),
            context: serde_json::json!({}),
            last_activity_at: now,
        }
    }

    pub fn with_channel(mut self, channel_id: Uuid) -> Self {
        self.channel_id = Some(channel_id);
        self
    }

    pub fn update_activity(&mut self) {
        self.last_activity_at = Utc::now();
        self.base.updated_at = Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.base.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
        self.base.updated_at = Utc::now();
    }

    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
        self.base.updated_at = Utc::now();
    }
}

impl Entity for Session {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "session"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}
