//! Goal and intent model for agent planning

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// Goal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl Default for GoalStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Goal priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for GoalPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// A goal represents an agent's objective or task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    #[serde(flatten)]
    pub base: EntityBase,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub parent_goal_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: GoalStatus,
    pub priority: GoalPriority,
    pub context: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Goal {
    pub fn new(user_id: Uuid, session_id: Uuid, title: String) -> Self {
        Self {
            base: EntityBase::new(),
            user_id,
            session_id,
            parent_goal_id: None,
            title,
            description: None,
            status: GoalStatus::Pending,
            priority: GoalPriority::Medium,
            context: serde_json::json!({}),
            result: None,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_goal_id = Some(parent_id);
        self
    }

    pub fn start(&mut self) {
        self.status = GoalStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn complete(&mut self, result: serde_json::Value) {
        self.status = GoalStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = GoalStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.status = GoalStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }
}

impl Entity for Goal {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "goal"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}
