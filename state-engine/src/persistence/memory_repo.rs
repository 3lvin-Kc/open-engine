//! Memory repository

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::database::{Database, DatabaseError};
use crate::models::*;

pub struct MemoryRepository {
    db: Database,
}

impl MemoryRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, memory: &Memory) -> Result<(), DatabaseError> {
        let tags_json = serde_json::to_string(&memory.tags).unwrap_or_else(|_| "[]".to_string());

        self.db.execute(
            "INSERT INTO memories (id, user_id, memory_type, importance, content, embedding,
             source_session_id, source_tool_execution_id, tags, expires_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &memory.base.id.to_string(),
                &memory.user_id.to_string(),
                &format!("{:?}", memory.memory_type).to_lowercase(),
                &format!("{:?}", memory.importance).to_lowercase(),
                &memory.content,
                &memory
                    .embedding
                    .as_ref()
                    .map(|e| bincode::serialize(e).unwrap_or_default()),
                &memory.source_session_id.map(|id| id.to_string()),
                &memory.source_tool_execution_id.map(|id| id.to_string()),
                &tags_json,
                &memory.expires_at.map(|dt| dt.to_rfc3339()),
                &memory.base.created_at.to_rfc3339(),
                &memory.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<Memory, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, memory_type, importance, content, embedding, source_session_id,
             source_tool_execution_id, tags, expires_at, created_at, updated_at 
             FROM memories WHERE id = ?",
            &[&id.to_string()],
            |row| Self::row_to_memory(row),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::NotFound(format!("Memory {} not found", id)))
    }

    pub fn update(&self, memory: &Memory) -> Result<(), DatabaseError> {
        let tags_json = serde_json::to_string(&memory.tags).unwrap_or_else(|_| "[]".to_string());

        let rows = self.db.execute(
            "UPDATE memories SET memory_type = ?, importance = ?, content = ?, embedding = ?,
             source_session_id = ?, source_tool_execution_id = ?, tags = ?, expires_at = ?,
             updated_at = ? WHERE id = ?",
            &[
                &format!("{:?}", memory.memory_type).to_lowercase(),
                &format!("{:?}", memory.importance).to_lowercase(),
                &memory.content,
                &memory
                    .embedding
                    .as_ref()
                    .map(|e| bincode::serialize(e).unwrap_or_default()),
                &memory.source_session_id.map(|id| id.to_string()),
                &memory.source_tool_execution_id.map(|id| id.to_string()),
                &tags_json,
                &memory.expires_at.map(|dt| dt.to_rfc3339()),
                &memory.base.updated_at.to_rfc3339(),
                &memory.base.id.to_string(),
            ],
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound(format!(
                "Memory {} not found",
                memory.base.id
            )));
        }
        Ok(())
    }

    pub fn list_for_user(&self, user_id: Uuid, limit: usize) -> Result<Vec<Memory>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, memory_type, importance, content, embedding, source_session_id,
             source_tool_execution_id, tags, expires_at, created_at, updated_at 
             FROM memories WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
            &[&user_id.to_string(), &limit.to_string()],
            |row| Self::row_to_memory(row),
        )
    }

    pub fn list_by_type(
        &self,
        user_id: Uuid,
        memory_type: MemoryType,
        limit: usize,
    ) -> Result<Vec<Memory>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, memory_type, importance, content, embedding, source_session_id,
             source_tool_execution_id, tags, expires_at, created_at, updated_at 
             FROM memories WHERE user_id = ? AND memory_type = ? ORDER BY created_at DESC LIMIT ?",
            &[
                &user_id.to_string(),
                &format!("{:?}", memory_type).to_lowercase(),
                &limit.to_string(),
            ],
            |row| Self::row_to_memory(row),
        )
    }

    pub fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let rows = self
            .db
            .execute("DELETE FROM memories WHERE id = ?", &[&id.to_string()])?;
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Memory {} not found", id)));
        }
        Ok(())
    }

    pub fn delete_expired(&self) -> Result<usize, DatabaseError> {
        let now = Utc::now().to_rfc3339();
        self.db.execute(
            "DELETE FROM memories WHERE expires_at IS NOT NULL AND expires_at < ?",
            &[&now],
        )
    }

    fn row_to_memory(row: &rusqlite::Row) -> Result<Memory, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let user_id_str: String = row.get(1)?;
        let memory_type_str: String = row.get(2)?;
        let importance_str: String = row.get(3)?;
        let content: String = row.get(4)?;
        let embedding_bytes: Option<Vec<u8>> = row.get(5)?;
        let source_session_id_str: Option<String> = row.get(6)?;
        let source_tool_execution_id_str: Option<String> = row.get(7)?;
        let tags_str: String = row.get(8)?;
        let expires_at_str: Option<String> = row.get(9)?;
        let created_at_str: String = row.get(10)?;
        let updated_at_str: String = row.get(11)?;

        let embedding: Option<Vec<f32>> =
            embedding_bytes.and_then(|bytes| bincode::deserialize(&bytes).ok());

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Memory {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            user_id: Uuid::parse_str(&user_id_str).unwrap(),
            memory_type: match memory_type_str.as_str() {
                "short_term" => MemoryType::ShortTerm,
                "long_term" => MemoryType::LongTerm,
                _ => MemoryType::ShortTerm,
            },
            importance: match importance_str.as_str() {
                "low" => MemoryImportance::Low,
                "medium" => MemoryImportance::Medium,
                "high" => MemoryImportance::High,
                "critical" => MemoryImportance::Critical,
                _ => MemoryImportance::Medium,
            },
            content,
            embedding,
            source_session_id: source_session_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            source_tool_execution_id: source_tool_execution_id_str
                .map(|s| Uuid::parse_str(&s).unwrap()),
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            expires_at: expires_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        })
    }
}
