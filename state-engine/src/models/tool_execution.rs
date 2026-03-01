//! Tool execution tracking model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    pub retry_state: Option<RetryState>,
}

/// Retry state tracking for automatic retries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub retry_policy: Option<RetryPolicy>,
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            attempt_count: 0,
            last_error: None,
            next_retry_at: None,
            retry_policy: None,
        }
    }
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
            retry_state: None,
        }
    }

    pub fn with_idempotency_key(mut self, key: String) -> Self {
        self.idempotency_key = Some(key);
        self
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

    pub fn mark_for_retry(&mut self, error: String, retry_policy: &RetryPolicy) {
        self.status = ToolStatus::Pending;
        self.error = Some(error.clone());
        
        let attempt_count = self.retry_state.as_ref().map(|r| r.attempt_count).unwrap_or(0) + 1;
        let delay_ms = retry_policy.calculate_delay(attempt_count);
        let next_retry = Utc::now() + chrono::Duration::milliseconds(delay_ms as i64);
        
        self.retry_state = Some(RetryState {
            attempt_count,
            last_error: Some(error),
            next_retry_at: Some(next_retry),
            retry_policy: Some(retry_policy.clone()),
        });
        
        self.base.updated_at = Utc::now();
    }

    pub fn can_retry(&self, retry_policy: &RetryPolicy) -> bool {
        let attempt_count = self.retry_state.as_ref().map(|r| r.attempt_count).unwrap_or(0);
        attempt_count < retry_policy.max_attempts
    }

    pub fn is_ready_for_retry(&self) -> bool {
        if self.status != ToolStatus::Pending {
            return false;
        }
        
        if let Some(ref retry_state) = self.retry_state {
            if let Some(next_retry) = retry_state.next_retry_at {
                return Utc::now() >= next_retry;
            }
        }
        
        false
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
