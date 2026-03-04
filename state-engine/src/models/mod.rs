//! State data models for OpenClaw
//!
//! This module defines the strongly-typed schemas for all state entities.

mod channel;
mod goal;
mod memory;
mod session;
mod tool_execution;
mod user;

pub use channel::*;
pub use goal::*;
pub use memory::*;
pub use session::*;
pub use tool_execution::*;
pub use user::*;

/// Schema version for migrations
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Entity trait for all state objects
pub trait Entity: Serialize + for<'de> Deserialize<'de> {
    fn id(&self) -> Uuid;
    fn entity_type() -> &'static str;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Base fields common to all entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBase {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EntityBase {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_id(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
        }
    }
}

impl Default for EntityBase {
    fn default() -> Self {
        Self::new()
    }
}

/// State version for tracking schema migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    pub version: u32,
    pub applied_at: DateTime<Utc>,
}

impl StateVersion {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            applied_at: Utc::now(),
        }
    }
}
