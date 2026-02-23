//! Memory model for agent short-term and long-term memory

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// Memory type - short-term (working) or long-term (persistent)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    ShortTerm,
    LongTerm,
}

impl Default for MemoryType {
    fn default() -> Self {
        Self::ShortTerm
    }
}

/// Memory importance level for retention decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryImportance {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for MemoryImportance {
    fn default() -> Self {
        Self::Medium
    }
}

/// Memory entry for agent context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    #[serde(flatten)]
    pub base: EntityBase,
    pub user_id: Uuid,
    pub memory_type: MemoryType,
    pub importance: MemoryImportance,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub source_session_id: Option<Uuid>,
    pub source_tool_execution_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Memory {
    pub fn new_short_term(user_id: Uuid, content: String) -> Self {
        Self {
            base: EntityBase::new(),
            user_id,
            memory_type: MemoryType::ShortTerm,
            importance: MemoryImportance::Medium,
            content,
            embedding: None,
            source_session_id: None,
            source_tool_execution_id: None,
            tags: vec![],
            expires_at: None,
        }
    }

    pub fn new_long_term(user_id: Uuid, content: String) -> Self {
        Self {
            base: EntityBase::new(),
            user_id,
            memory_type: MemoryType::LongTerm,
            importance: MemoryImportance::Medium,
            content,
            embedding: None,
            source_session_id: None,
            source_tool_execution_id: None,
            tags: vec![],
            expires_at: None,
        }
    }

    pub fn with_importance(mut self, importance: MemoryImportance) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn promote_to_long_term(&mut self) {
        self.memory_type = MemoryType::LongTerm;
        self.base.updated_at = Utc::now();
    }
}

impl Entity for Memory {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "memory"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}

/// Query for searching memories
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryQuery {
    pub user_id: Option<Uuid>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
