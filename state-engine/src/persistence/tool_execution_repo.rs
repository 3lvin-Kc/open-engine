//! Tool execution repository

use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use super::database::{Database, DatabaseError};
use crate::models::*;

/// Result of idempotent create operation
#[derive(Debug)]
pub enum IdempotentCreateResult {
    /// New execution was created
    Created(ToolExecution),
    /// Existing execution with same idempotency_key was found
    Duplicate(ToolExecution),
}

pub struct ToolExecutionRepository {
    db: Database,
}

impl ToolExecutionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, execution: &ToolExecution) -> Result<(), DatabaseError> {
        self.db.execute(
            "INSERT INTO tool_executions (id, goal_id, session_id, user_id, tool_name, tool_input,
             status, output, error, idempotency_key, started_at, completed_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &execution.base.id.to_string(),
                &execution.goal_id.to_string(),
                &execution.session_id.to_string(),
                &execution.user_id.to_string(),
                &execution.tool_name,
                &execution.tool_input.to_string(),
                &format!("{:?}", execution.status).to_lowercase(),
                &execution.output.as_ref().map(|v| v.to_string()),
                &execution.error,
                &execution.idempotency_key,
                &execution.started_at.map(|dt| dt.to_rfc3339()),
                &execution.completed_at.map(|dt| dt.to_rfc3339()),
                &execution.base.created_at.to_rfc3339(),
                &execution.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<ToolExecution, DatabaseError> {
        self.db
            .query(
                "SELECT id, goal_id, session_id, user_id, tool_name, tool_input, status, output,
             error, idempotency_key, started_at, completed_at, created_at, updated_at
             FROM tool_executions WHERE id = ?",
                &[&id.to_string()],
                |row| Self::row_to_execution(row),
            )?
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::NotFound(format!("ToolExecution {} not found", id)))
    }

    pub fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<ToolExecution>, DatabaseError> {
        let executions = self.db.query(
            "SELECT id, goal_id, session_id, user_id, tool_name, tool_input, status, output,
             error, idempotency_key, started_at, completed_at, created_at, updated_at
             FROM tool_executions WHERE idempotency_key = ?",
            &[&key.to_string()],
            |row| Self::row_to_execution(row),
        )?;

        Ok(executions.into_iter().next())
    }

    /// Create a tool execution with hard idempotency enforcement.
    ///
    /// If an execution with the same idempotency_key already exists,
    /// returns that existing execution instead of creating a new one.
    /// This is atomic and race-condition safe.
    pub fn create_idempotent(
        &self,
        execution: &ToolExecution,
    ) -> Result<IdempotentCreateResult, DatabaseError> {
        // If no idempotency key, just create normally
        if execution.idempotency_key.is_none() {
            self.create(execution)?;
            return Ok(IdempotentCreateResult::Created(execution.clone()));
        }

        let key = execution.idempotency_key.as_ref().unwrap();

        // Use a transaction for atomic check-and-insert
        self.db.transaction(|conn| {
            // First check if exists
            let existing: Option<ToolExecution> = conn.query_row(
                "SELECT id, goal_id, session_id, user_id, tool_name, tool_input, status, output,
                 error, idempotency_key, started_at, completed_at, created_at, updated_at
                 FROM tool_executions WHERE idempotency_key = ?",
                [key],
                |row| Self::row_to_execution(row),
            ).ok();

            if let Some(existing) = existing {
                return Ok(IdempotentCreateResult::Duplicate(existing));
            }

            // Insert new execution
            conn.execute(
                "INSERT INTO tool_executions (id, goal_id, session_id, user_id, tool_name, tool_input,
                 status, output, error, idempotency_key, started_at, completed_at, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    execution.base.id.to_string(),
                    execution.goal_id.to_string(),
                    execution.session_id.to_string(),
                    execution.user_id.to_string(),
                    &execution.tool_name,
                    execution.tool_input.to_string(),
                    format!("{:?}", execution.status).to_lowercase(),
                    execution.output.as_ref().map(|v| v.to_string()),
                    &execution.error,
                    &execution.idempotency_key,
                    execution.started_at.map(|dt| dt.to_rfc3339()),
                    execution.completed_at.map(|dt| dt.to_rfc3339()),
                    execution.base.created_at.to_rfc3339(),
                    execution.base.updated_at.to_rfc3339(),
                ],
            ).map_err(|e| DatabaseError::QueryError(e.to_string()))?;

            Ok(IdempotentCreateResult::Created(execution.clone()))
        })
    }

    pub fn update(&self, execution: &ToolExecution) -> Result<(), DatabaseError> {
        let rows = self.db.execute(
            "UPDATE tool_executions SET status = ?, output = ?, error = ?, started_at = ?,
             completed_at = ?, updated_at = ? WHERE id = ?",
            &[
                &format!("{:?}", execution.status).to_lowercase(),
                &execution.output.as_ref().map(|v| v.to_string()),
                &execution.error,
                &execution.started_at.map(|dt| dt.to_rfc3339()),
                &execution.completed_at.map(|dt| dt.to_rfc3339()),
                &execution.base.updated_at.to_rfc3339(),
                &execution.base.id.to_string(),
            ],
        )?;

        if rows == 0 {
            return Err(DatabaseError::NotFound(format!(
                "ToolExecution {} not found",
                execution.base.id
            )));
        }
        Ok(())
    }

    pub fn list_for_goal(&self, goal_id: Uuid) -> Result<Vec<ToolExecution>, DatabaseError> {
        self.db.query(
            "SELECT id, goal_id, session_id, user_id, tool_name, tool_input, status, output,
             error, idempotency_key, started_at, completed_at, created_at, updated_at
             FROM tool_executions WHERE goal_id = ? ORDER BY created_at ASC",
            &[&goal_id.to_string()],
            |row| Self::row_to_execution(row),
        )
    }

    fn row_to_execution(row: &rusqlite::Row) -> Result<ToolExecution, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let goal_id_str: String = row.get(1)?;
        let session_id_str: String = row.get(2)?;
        let user_id_str: String = row.get(3)?;
        let tool_name: String = row.get(4)?;
        let tool_input_str: String = row.get(5)?;
        let status_str: String = row.get(6)?;
        let output_str: Option<String> = row.get(7)?;
        let error: Option<String> = row.get(8)?;
        let idempotency_key: Option<String> = row.get(9)?;
        let started_at_str: Option<String> = row.get(10)?;
        let completed_at_str: Option<String> = row.get(11)?;
        let created_at_str: String = row.get(12)?;
        let updated_at_str: String = row.get(13)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(ToolExecution {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            goal_id: Uuid::parse_str(&goal_id_str).unwrap(),
            session_id: Uuid::parse_str(&session_id_str).unwrap(),
            user_id: Uuid::parse_str(&user_id_str).unwrap(),
            tool_name,
            tool_input: serde_json::from_str(&tool_input_str).unwrap_or(serde_json::json!({})),
            status: match status_str.as_str() {
                "pending" => ToolStatus::Pending,
                "running" => ToolStatus::Running,
                "completed" => ToolStatus::Completed,
                "failed" => ToolStatus::Failed,
                "cancelled" => ToolStatus::Cancelled,
                _ => ToolStatus::Pending,
            },
            output: output_str.and_then(|s| serde_json::from_str(&s).ok()),
            error,
            idempotency_key,
            started_at: started_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
            completed_at: completed_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            }),
        })
    }
}
