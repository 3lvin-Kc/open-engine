//! Goal repository

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::*;
use super::database::{Database, DatabaseError};

pub struct GoalRepository {
    db: Database,
}

impl GoalRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create(&self, goal: &Goal) -> Result<(), DatabaseError> {
        self.db.execute(
            "INSERT INTO goals (id, user_id, session_id, parent_goal_id, title, description,
             status, priority, context, result, started_at, completed_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &goal.base.id.to_string(),
                &goal.user_id.to_string(),
                &goal.session_id.to_string(),
                &goal.parent_goal_id.map(|id| id.to_string()),
                &goal.title,
                &goal.description,
                &format!("{:?}", goal.status).to_lowercase(),
                &format!("{:?}", goal.priority).to_lowercase(),
                &goal.context.to_string(),
                &goal.result.as_ref().map(|v| v.to_string()),
                &goal.started_at.map(|dt| dt.to_rfc3339()),
                &goal.completed_at.map(|dt| dt.to_rfc3339()),
                &goal.base.created_at.to_rfc3339(),
                &goal.base.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: Uuid) -> Result<Goal, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, session_id, parent_goal_id, title, description, status, priority,
             context, result, started_at, completed_at, created_at, updated_at 
             FROM goals WHERE id = ?",
            &[&id.to_string()],
            |row| Self::row_to_goal(row),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| DatabaseError::NotFound(format!("Goal {} not found", id)))
    }

    pub fn update(&self, goal: &Goal) -> Result<(), DatabaseError> {
        let rows = self.db.execute(
            "UPDATE goals SET parent_goal_id = ?, title = ?, description = ?, status = ?,
             priority = ?, context = ?, result = ?, started_at = ?, completed_at = ?,
             updated_at = ? WHERE id = ?",
            &[
                &goal.parent_goal_id.map(|id| id.to_string()),
                &goal.title,
                &goal.description,
                &format!("{:?}", goal.status).to_lowercase(),
                &format!("{:?}", goal.priority).to_lowercase(),
                &goal.context.to_string(),
                &goal.result.as_ref().map(|v| v.to_string()),
                &goal.started_at.map(|dt| dt.to_rfc3339()),
                &goal.completed_at.map(|dt| dt.to_rfc3339()),
                &goal.base.updated_at.to_rfc3339(),
                &goal.base.id.to_string(),
            ],
        )?;
        
        if rows == 0 {
            return Err(DatabaseError::NotFound(format!("Goal {} not found", goal.base.id)));
        }
        Ok(())
    }

    pub fn list_for_session(&self, session_id: Uuid) -> Result<Vec<Goal>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, session_id, parent_goal_id, title, description, status, priority,
             context, result, started_at, completed_at, created_at, updated_at 
             FROM goals WHERE session_id = ? ORDER BY created_at ASC",
            &[&session_id.to_string()],
            |row| Self::row_to_goal(row),
        )
    }

    pub fn list_pending_for_session(&self, session_id: Uuid) -> Result<Vec<Goal>, DatabaseError> {
        self.db.query(
            "SELECT id, user_id, session_id, parent_goal_id, title, description, status, priority,
             context, result, started_at, completed_at, created_at, updated_at 
             FROM goals WHERE session_id = ? AND status IN ('pending', 'in_progress') 
             ORDER BY 
                CASE priority 
                    WHEN 'critical' THEN 1 
                    WHEN 'high' THEN 2 
                    WHEN 'medium' THEN 3 
                    WHEN 'low' THEN 4 
                END,
                created_at ASC",
            &[&session_id.to_string()],
            |row| Self::row_to_goal(row),
        )
    }

    fn row_to_goal(row: &rusqlite::Row) -> Result<Goal, rusqlite::Error> {
        let id_str: String = row.get(0)?;
        let user_id_str: String = row.get(1)?;
        let session_id_str: String = row.get(2)?;
        let parent_goal_id_str: Option<String> = row.get(3)?;
        let title: String = row.get(4)?;
        let description: Option<String> = row.get(5)?;
        let status_str: String = row.get(6)?;
        let priority_str: String = row.get(7)?;
        let context_str: String = row.get(8)?;
        let result_str: Option<String> = row.get(9)?;
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

        Ok(Goal {
            base: EntityBase {
                id: Uuid::parse_str(&id_str).unwrap(),
                created_at,
                updated_at,
            },
            user_id: Uuid::parse_str(&user_id_str).unwrap(),
            session_id: Uuid::parse_str(&session_id_str).unwrap(),
            parent_goal_id: parent_goal_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
            title,
            description,
            status: match status_str.as_str() {
                "pending" => GoalStatus::Pending,
                "in_progress" => GoalStatus::InProgress,
                "completed" => GoalStatus::Completed,
                "failed" => GoalStatus::Failed,
                "cancelled" => GoalStatus::Cancelled,
                _ => GoalStatus::Pending,
            },
            priority: match priority_str.as_str() {
                "low" => GoalPriority::Low,
                "medium" => GoalPriority::Medium,
                "high" => GoalPriority::High,
                "critical" => GoalPriority::Critical,
                _ => GoalPriority::Medium,
            },
            context: serde_json::from_str(&context_str).unwrap_or(serde_json::json!({})),
            result: result_str.and_then(|s| serde_json::from_str(&s).ok()),
            started_at: started_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
            completed_at: completed_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
        })
    }
}
