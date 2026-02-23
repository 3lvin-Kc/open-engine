//! SQLite database management

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use rusqlite::Connection;
use tracing::info;

use super::migrations::{run_migrations, MigrationError};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub wal_mode: bool,
    pub busy_timeout_ms: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("openclaw_state.db"),
            wal_mode: true,
            busy_timeout_ms: 5000,
        }
    }
}

/// Database wrapper for SQLite
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create a database at the specified path
    pub fn open(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("Opening database at {:?}", config.path);
        
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::IoError(e.to_string()))?;
        }
        
        let conn = Connection::open(&config.path)
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        // Configure connection
        conn.busy_timeout(std::time::Duration::from_millis(config.busy_timeout_ms))
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        if config.wal_mode {
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        }
        
        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DatabaseError::ConnectionError(e.to_string()))?;
        
        // Run migrations
        run_migrations(&conn)?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
        };
        
        info!("Database opened successfully");
        Ok(db)
    }
    
    /// Get a reference to the database connection
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
    
    /// Execute a transaction with the given closure
    pub fn transaction<T, F>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&Connection) -> Result<T, DatabaseError>,
    {
        let conn = self.conn.lock();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
        
        let result = f(&tx);
        
        match result {
            Ok(value) => {
                tx.commit()
                    .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback()
                    .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;
                Err(e)
            }
        }
    }
    
    /// Execute a query that returns rows
    pub fn query<T, F>(&self, sql: &str, params: &[&dyn rusqlite::ToSql], f: F) -> Result<Vec<T>, DatabaseError>
    where
        F: FnMut(&rusqlite::Row<'_>) -> Result<T, rusqlite::Error>,
    {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        let rows = stmt
            .query_map(params, f)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| DatabaseError::QueryError(e.to_string()))?);
        }
        
        Ok(results)
    }
    
    /// Execute an update/insert/delete query
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<usize, DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(sql, params)
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }
    
    /// Get the current schema version
    pub fn get_schema_version(&self) -> Result<u32, DatabaseError> {
        let conn = self.conn.lock();
        let result: Result<i64, _> = conn.query_row(
            "SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        );
        
        match result {
            Ok(version) => Ok(version as u32),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(DatabaseError::QueryError(e.to_string())),
        }
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
            config: self.config.clone(),
        }
    }
}

/// Database-specific errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),
    
    #[error("Database query error: {0}")]
    QueryError(String),
    
    #[error("Database transaction error: {0}")]
    TransactionError(String),
    
    #[error("Migration error: {0}")]
    MigrationError(#[from] MigrationError),
    
    #[error("I/O error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintError(String),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DatabaseError::NotFound("No rows returned".to_string())
            }
            _ => DatabaseError::QueryError(err.to_string()),
        }
    }
}

impl serde::Serialize for DatabaseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
