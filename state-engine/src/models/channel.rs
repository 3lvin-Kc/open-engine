//! Channel model for cross-channel context management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Entity, EntityBase};

/// Channel type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Chat,
    Email,
    Calendar,
    Voice,
    Webhook,
    Custom,
}

impl Default for ChannelType {
    fn default() -> Self {
        Self::Chat
    }
}

/// Channel status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelStatus {
    Active,
    Paused,
    Inactive,
}

impl Default for ChannelStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Channel represents a communication channel (e.g., Slack, Email, Calendar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(flatten)]
    pub base: EntityBase,
    pub user_id: Uuid,
    pub channel_type: ChannelType,
    pub channel_name: String,
    pub channel_identifier: String,
    pub status: ChannelStatus,
    pub metadata: serde_json::Value,
    pub config: serde_json::Value,
    pub last_message_at: Option<DateTime<Utc>>,
}

impl Channel {
    pub fn new(user_id: Uuid, channel_type: ChannelType, channel_name: String, channel_identifier: String) -> Self {
        Self {
            base: EntityBase::new(),
            user_id,
            channel_type,
            channel_name,
            channel_identifier,
            status: ChannelStatus::Active,
            metadata: serde_json::json!({}),
            config: serde_json::json!({}),
            last_message_at: None,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }

    pub fn update_last_message(&mut self) {
        self.last_message_at = Some(Utc::now());
        self.base.updated_at = Utc::now();
    }

    pub fn pause(&mut self) {
        self.status = ChannelStatus::Paused;
        self.base.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.status = ChannelStatus::Inactive;
        self.base.updated_at = Utc::now();
    }
}

impl Entity for Channel {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn entity_type() -> &'static str {
        "channel"
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }
}

/// Query for finding channels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelQuery {
    pub user_id: Option<Uuid>,
    pub channel_type: Option<ChannelType>,
    pub status: Option<ChannelStatus>,
}
