//! Tool execution tracking model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Default for ToolStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Record of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    #[serde(flatten)]
    pub base: EntityBase,
    pub goal_id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub status: ToolStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub idempotency_key: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ToolExecution {
    pub fn new(
        user_id: Uuid,
        session_id: Uuid,
        goal_id: Uuid,
        tool_name: String,
        tool_input: serde_json::Value,
    ) -> Self {
        Self {
            base: EntityBase::new(),
            goal_id,
            session_id,
            user_id,
            tool_name,
            tool_input,
            status: ToolStatus::Pending,
            output: None,
            error: None,
            idempotency_key: None,
            started_at: None,
            completed_at: None,
        }
    }

    pub fn with_idempotency_key(mut self, key: String) -> Self {
        self.idempotency_key = Some(key);
        self
    }

    pub fn generate_idempotency_key(
        user_id: Uuid,
        session_id: Uuid,
        goal_id: Uuid,
        tool_name: &str,
        tool_input: &serde_json::Value,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        session_id.hash(&mut hasher);
        goal_id.hash(&mut hasher);
        tool_name.hash(&mut hasher);
        tool_input.hash(&mut hasher);
        let hash = hasher.finish();
        format!(
            "auto:{}-{}:{:x}",
            tool_name,
            chrono::Utc::now().format("%Y%m%d%H%M%S"),
            hash
        )
    }

    pub fn start(&mut self) {
        self.status = ToolStatus::Running;
        self.started_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn complete(&mut self, output: serde_json::Value) {
        self.status = ToolStatus::Completed;
        self.output = Some(output);
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn fail(&mut self, error: String) {
        self.status = ToolStatus::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn cancel(&mut self) {
        self.status = ToolStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }
}

impl Entity for ToolExecution {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "tool_execution"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}

/// Query for finding duplicate tool executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionQuery {
    pub goal_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub tool_name: Option<String>,
    pub idempotency_key: Option<String>,
    pub status: Option<ToolStatus>,
}
