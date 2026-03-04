//! Repository layer for CRUD operations on state entities

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::database::{Database, DatabaseError};
use crate::models::*;

/// Repository for User entities
pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, user: &User) -> Result<(), DatabaseError> {
        self.db.execute(
            "INSERT INTO users (id, username, display_name, email, metadata, active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &user.base.id.to_string(),
                &user.username,
                &user.display_name,
                &user.email,
                &user.metadata.to_string(),
                &(user.active as i32).to_string(),
                &user.base.created_at.to_rfc3339(),
                &user.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<User, DatabaseError> {
        self.db.query(
            "SELECT id, username, display_name, email, metadata, active, created_at, updated_at 
             FROM users WHERE id = ?",
            &[&id.to_string()],
            |row| Self::row_to_user(row),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::NotFound(format!("User {} not found", id)))
    }

    pub fn get_by_username(&self, username: &str) -> Result<User, DatabaseError> {
        self.db.query(
            "SELECT id, username, display_name, email, metadata, active, created_at, updated_at 
             FROM users WHERE username = ?",
            &[&username.to_string()],
            |row| Self::row_to_user(row),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::NotFound(format!("User {} not found", username)))
    }

    pub fn update(&self, user: &User) -> Result<(), DatabaseError> {
        let rows = self.db.execute(
            "UPDATE users SET username = ?, display_name = ?, email = ?, metadata = ?, 
             active = ?, updated_at = ? WHERE id = ?",
            &[
                &user.username,
                &user.display_name,
                &user.email,
                &user.metadata.to_string(),
                &(user.active as i32).to_string(),
                &user.base.updated_at.to_rfc3339(),
                &user.base.id.to_string(),
            ],
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound(format!(
                "User {} not found",
                user.base.id
            )));
        }
        Ok(())
    }

    pub fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let rows = self
            .db
            .execute("DELETE FROM users WHERE id = ?", &[&id.to_string()])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("User {} not found", id)));
        }
        Ok(())
    }

    pub fn list(&self, active_only: bool) -> Result<Vec<User>, DatabaseError> {
        let sql = if active_only {
            "SELECT id, username, display_name, email, metadata, active, created_at, updated_at 
             FROM users WHERE active = 1"
        } else {
            "SELECT id, username, display_name, email, metadata, active, created_at, updated_at FROM users"
        };

        self.db.query(sql, &[], |row| Self::row_to_user(row))
    }

    fn row_to_user(row: &rusqlite::Row) -> Result<User, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let username: String = row.get(1)?;
        let display_name: Option<String> = row.get(2)?;
        let email: Option<String> = row.get(3)?;
        let metadata_str: String = row.get(4)?;
        let active: i32 = row.get(5)?;
        let created_at_str: String = row.get(6)?;
        let updated_at_str: String = row.get(7)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(User {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            username,
            display_name,
            email,
            metadata: serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({})),
            active: active != 0,
        })
    }
}
