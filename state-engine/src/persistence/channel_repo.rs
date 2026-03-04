//! Channel repository

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::database::{Database, DatabaseError};
use crate::models::*;

pub struct ChannelRepository {
    db: Database,
}

impl ChannelRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, channel: &Channel) -> Result<(), DatabaseError> {
        self.db.execute(
            "INSERT INTO channels (id, user_id, channel_type, channel_name, channel_identifier,
             status, metadata, config, last_message_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &channel.base.id.to_string(),
                &channel.user_id.to_string(),
                &format!("{:?}", channel.channel_type).to_lowercase(),
                &channel.channel_name,
                &channel.channel_identifier,
                &format!("{:?}", channel.status).to_lowercase(),
                &channel.metadata.to_string(),
                &channel.config.to_string(),
                &channel.last_message_at.map(|dt| dt.to_rfc3339()),
                &channel.base.created_at.to_rfc3339(),
                &channel.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<Channel, DatabaseError> {
        self.db
            .query(
                "SELECT id, user_id, channel_type, channel_name, channel_identifier, status,
             metadata, config, last_message_at, created_at, updated_at 
             FROM channels WHERE id = ?",
                &[&id.to_string()],
                |row| Self::row_to_channel(row),
            )?
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::NotFound(format!("Channel {} not found", id)))
    }

    pub fn get_by_identifier(
        &self,
        user_id: Uuid,
        identifier: &str,
    ) -> Result<Option<Channel>, DatabaseError> {
        let channels = self.db.query(
            "SELECT id, user_id, channel_type, channel_name, channel_identifier, status,
             metadata, config, last_message_at, created_at, updated_at 
             FROM channels WHERE user_id = ? AND channel_identifier = ?",
            &[&user_id.to_string(), &identifier.to_string()],
            |row| Self::row_to_channel(row),
        )?;

        Ok(channels.into_iter().next())
    }

    pub fn update(&self, channel: &Channel) -> Result<(), DatabaseError> {
        let rows = self.db.execute(
            "UPDATE channels SET channel_type = ?, channel_name = ?, channel_identifier = ?,
             status = ?, metadata = ?, config = ?, last_message_at = ?, updated_at = ? 
             WHERE id = ?",
            &[
                &format!("{:?}", channel.channel_type).to_lowercase(),
                &channel.channel_name,
                &channel.channel_identifier,
                &format!("{:?}", channel.status).to_lowercase(),
                &channel.metadata.to_string(),
                &channel.config.to_string(),
                &channel.last_message_at.map(|dt| dt.to_rfc3339()),
                &channel.base.updated_at.to_rfc3339(),
                &channel.base.id.to_string(),
            ],
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound(format!(
                "Channel {} not found",
                channel.base.id
            )));
        }
        Ok(())
    }

    pub fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Channel>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, channel_type, channel_name, channel_identifier, status,
             metadata, config, last_message_at, created_at, updated_at 
             FROM channels WHERE user_id = ? ORDER BY created_at DESC",
            &[&user_id.to_string()],
            |row| Self::row_to_channel(row),
        )
    }

    pub fn list_by_type(
        &self,
        user_id: Uuid,
        channel_type: ChannelType,
    ) -> Result<Vec<Channel>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, channel_type, channel_name, channel_identifier, status,
             metadata, config, last_message_at, created_at, updated_at 
             FROM channels WHERE user_id = ? AND channel_type = ? ORDER BY created_at DESC",
            &[
                &user_id.to_string(),
                &format!("{:?}", channel_type).to_lowercase(),
            ],
            |row| Self::row_to_channel(row),
        )
    }

    pub fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let rows = self
            .db
            .execute("DELETE FROM channels WHERE id = ?", &[&id.to_string()])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Channel {} not found", id)));
        }
        Ok(())
    }

    fn row_to_channel(row: &rusqlite::Row) -> Result<Channel, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let user_id_str: String = row.get(1)?;
        let channel_type_str: String = row.get(2)?;
        let channel_name: String = row.get(3)?;
        let channel_identifier: String = row.get(4)?;
        let status_str: String = row.get(5)?;
        let metadata_str: String = row.get(6)?;
        let config_str: String = row.get(7)?;
        let last_message_at_str: Option<String> = row.get(8)?;
        let created_at_str: String = row.get(9)?;
        let updated_at_str: String = row.get(10)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Channel {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            user_id: Uuid::parse_str(&user_id_str).unwrap(),
            channel_type: match channel_type_str.as_str() {
                "chat" => ChannelType::Chat,
                "email" => ChannelType::Email,
                "calendar" => ChannelType::Calendar,
                "voice" => ChannelType::Voice,
                "webhook" => ChannelType::Webhook,
                "custom" => ChannelType::Custom,
                _ => ChannelType::Chat,
            },
            channel_name,
            channel_identifier,
            status: match status_str.as_str() {
                "active" => ChannelStatus::Active,
                "paused" => ChannelStatus::Paused,
                "inactive" => ChannelStatus::Inactive,
                _ => ChannelStatus::Active,
            },
            metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({})),
            config: serde_json::from_str(&config_str).unwrap_or(serde_json::json!({})),
            last_message_at: last_message_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        })
    }
}
