//! Session repository

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::database::{Database, DatabaseError};
use crate::models::*;

pub struct SessionRepository {
    db: Database,
}

impl SessionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, session: &Session) -> Result<(), DatabaseError> {
        self.db.execute(
            "INSERT INTO sessions (id, user_id, channel_id, status, metadata, context, 
             last_activity_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &session.base.id.to_string(),
                &session.user_id.to_string(),
                &session.channel_id.map(|id| id.to_string()),
                &format!("{:?}", session.status).to_lowercase(),
                &session.metadata.to_string(),
                &session.context.to_string(),
                &session.last_activity_at.to_rfc3339(),
                &session.base.created_at.to_rfc3339(),
                &session.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<Session, DatabaseError> {
        self.db
            .query(
                "SELECT id, user_id, channel_id, status, metadata, context, last_activity_at,
             created_at, updated_at FROM sessions WHERE id = ?",
                &[&id.to_string()],
                |row| Self::row_to_session(row),
            )?
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::NotFound(format!("Session {} not found", id)))
    }

    pub fn get_active_for_user(&self, user_id: Uuid) -> Result<Option<Session>, DatabaseError> {
        let sessions = self.db.query(
            "SELECT id, user_id, channel_id, status, metadata, context, last_activity_at,
             created_at, updated_at FROM sessions 
             WHERE user_id = ? AND status = 'active' ORDER BY last_activity_at DESC LIMIT 1",
            &[&user_id.to_string()],
            |row| Self::row_to_session(row),
        )?;

        Ok(sessions.into_iter().next())
    }

    pub fn update(&self, session: &Session) -> Result<(), DatabaseError> {
        let rows = self.db.execute(
            "UPDATE sessions SET channel_id = ?, status = ?, metadata = ?, context = ?,
             last_activity_at = ?, updated_at = ? WHERE id = ?",
            &[
                &session.channel_id.map(|id| id.to_string()),
                &format!("{:?}", session.status).to_lowercase(),
                &session.metadata.to_string(),
                &session.context.to_string(),
                &session.last_activity_at.to_rfc3339(),
                &session.base.updated_at.to_rfc3339(),
                &session.base.id.to_string(),
            ],
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound(format!(
                "Session {} not found",
                session.base.id
            )));
        }
        Ok(())
    }

    pub fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Session>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, channel_id, status, metadata, context, last_activity_at,
             created_at, updated_at FROM sessions WHERE user_id = ? ORDER BY created_at DESC",
            &[&user_id.to_string()],
            |row| Self::row_to_session(row),
        )
    }

    fn row_to_session(row: &rusqlite::Row) -> Result<Session, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let user_id_str: String = row.get(1)?;
        let channel_id_str: Option<String> = row.get(2)?;
        let status_str: String = row.get(3)?;
        let metadata_str: String = row.get(4)?;
        let context_str: String = row.get(5)?;
        let last_activity_str: String = row.get(6)?;
        let created_at_str: String = row.get(7)?;
        let updated_at_str: String = row.get(8)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Session {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            user_id: Uuid::parse_str(&user_id_str).unwrap(),
            channel_id: channel_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            status: match status_str.as_str() {
                "active" => SessionStatus::Active,
                "paused" => SessionStatus::Paused,
                "completed" => SessionStatus::Completed,
                "failed" => SessionStatus::Failed,
                "abandoned" => SessionStatus::Abandoned,
                _ => SessionStatus::Active,
            },
            metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({})),
            context: serde_json::from_str(&context_str).unwrap_or(serde_json::json!({})),
            last_activity_at: DateTime::parse_from_rfc3339(&last_activity_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}
