//! Tool Schema Repository
//!
//! Persist tool schemas for discovery and validation.

use rusqlite::{params, OptionalExtension};
use serde_json;
use uuid::Uuid;

use crate::models::{ToolSchema, ToolParameter, RetryPolicy};
use super::{Database, Repository, PersistResult};

pub struct ToolSchemaRepository {
    db: Database,
}

impl Repository<ToolSchema> for ToolSchemaRepository {
    fn new(db: Database) -> Self {
        Self { db }
    }

    fn create(&self, schema: &ToolSchema) -> PersistResult<()> {
        let conn = self.db.connection.lock().unwrap();
        
        let params_json = serde_json::to_string(&schema.parameters)
            .map_err(|e| super::PersistenceError::SerializationError(e.to_string()))?;
        let tags_json = serde_json::to_string(&schema.tags)
            .map_err(|e| super::PersistenceError::SerializationError(e.to_string()))?;
        
        conn.execute(
            "INSERT INTO tool_schemas (
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
            )",
            params![
                schema.base.id.to_string(),
                &schema.tool_name,
                &schema.description,
                &schema.version,
                params_json,
                schema.return_type.as_ref().map(|t| format!("{:?}", t)),
                schema.return_description.as_deref(),
                schema.category.as_deref(),
                tags_json,
                schema.enabled,
                schema.deprecated,
                schema.deprecation_message.as_deref(),
                schema.base.created_at.to_rfc3339(),
                schema.base.updated_at.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }

    fn get(&self, id: Uuid) -> PersistResult<Option<ToolSchema>> {
        let conn = self.db.connection.lock().unwrap();
        
        let result = conn.query_row(
            "SELECT 
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            FROM tool_schemas WHERE id = ?1",
            params![id.to_string()],
            |row| self.row_to_schema(row),
        ).optional()?;
        
        Ok(result)
    }

    fn update(&self, schema: &ToolSchema) -> PersistResult<()> {
        let conn = self.db.connection.lock().unwrap();
        
        let params_json = serde_json::to_string(&schema.parameters)
            .map_err(|e| super::PersistenceError::SerializationError(e.to_string()))?;
        let tags_json = serde_json::to_string(&schema.tags)
            .map_err(|e| super::PersistenceError::SerializationError(e.to_string()))?;
        
        conn.execute(
            "UPDATE tool_schemas SET
                tool_name = ?2,
                description = ?3,
                version = ?4,
                parameters = ?5,
                return_type = ?6,
                return_description = ?7,
                category = ?8,
                tags = ?9,
                enabled = ?10,
                deprecated = ?11,
                deprecation_message = ?12,
                updated_at = ?13
            WHERE id = ?1",
            params![
                schema.base.id.to_string(),
                &schema.tool_name,
                &schema.description,
                &schema.version,
                params_json,
                schema.return_type.as_ref().map(|t| format!("{:?}", t)),
                schema.return_description.as_deref(),
                schema.category.as_deref(),
                tags_json,
                schema.enabled,
                schema.deprecated,
                schema.deprecation_message.as_deref(),
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }

    fn delete(&self, id: Uuid) -> PersistResult<()> {
        let conn = self.db.connection.lock().unwrap();
        
        conn.execute(
            "DELETE FROM tool_schemas WHERE id = ?1",
            params![id.to_string()],
        )?;
        
        Ok(())
    }
}

impl ToolSchemaRepository {
    pub fn get_by_name(&self, tool_name: &str) -> PersistResult<Option<ToolSchema>> {
        let conn = self.db.connection.lock().unwrap();
        
        let result = conn.query_row(
            "SELECT 
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            FROM tool_schemas WHERE tool_name = ?1",
            params![tool_name],
            |row| self.row_to_schema(row),
        ).optional()?;
        
        Ok(result)
    }

    pub fn list_enabled(&self) -> PersistResult<Vec<ToolSchema>> {
        let conn = self.db.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            FROM tool_schemas WHERE enabled = 1"
        )?;
        
        let schemas = stmt.query_map([], |row| self.row_to_schema(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(schemas)
    }

    pub fn list_by_category(&self, category: &str) -> PersistResult<Vec<ToolSchema>> {
        let conn = self.db.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            FROM tool_schemas WHERE category = ?1 AND enabled = 1"
        )?;
        
        let schemas = stmt.query_map(params![category], |row| self.row_to_schema(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(schemas)
    }

    pub fn search_by_tag(&self, tag: &str) -> PersistResult<Vec<ToolSchema>> {
        let conn = self.db.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT 
                id, tool_name, description, version, parameters, return_type,
                return_description, category, tags, enabled, deprecated,
                deprecation_message, created_at, updated_at
            FROM tool_schemas WHERE tags LIKE ?1 AND enabled = 1"
        )?;
        
        let search_pattern = format!("%\"{}\"%", tag);
        let schemas = stmt.query_map(params![search_pattern], |row| self.row_to_schema(row))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(schemas)
    }

    fn row_to_schema(&self, row: &rusqlite::Row) -> Result<ToolSchema, rusqlite::Error> {
        use chrono::DateTime;
        
        let params_json: String = row.get(4)?;
        let parameters: Vec<ToolParameter> = serde_json::from_str(&params_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string()
                ))
            ))?;
        
        let tags_json: String = row.get(8)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                8,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string()
                ))
            ))?;
        
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string()
                ))
            ))?;
        
        let created_at_str: String = row.get(12)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string()
                ))
            ))?.with_timezone(&chrono::Utc);
        
        let updated_at_str: String = row.get(13)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                13,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string()
                ))
            ))?.with_timezone(&chrono::Utc);
        
        Ok(ToolSchema {
            base: crate::models::EntityBase {
                id,
                created_at,
                updated_at,
            },
            tool_name: row.get(1)?,
            description: row.get(2)?,
            version: row.get(3)?,
            parameters,
            return_type: None, // TODO: Parse from string
            return_description: row.get(6)?,
            category: row.get(7)?,
            tags,
            enabled: row.get(9)?,
            deprecated: row.get(10)?,
            deprecation_message: row.get(11)?,
        })
    }
}
