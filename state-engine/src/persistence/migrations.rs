//! Database migrations

use rusqlite::{Connection, params};
use tracing::info;

/// Migration error type
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Migration failed: {0}")]
    Failed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Run all pending migrations
pub fn run_migrations(conn: &Connection) -> Result<(), MigrationError> {
    // Create schema_version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    let current_version: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| {
            row.get(0)
        })
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    if current_version < 1 {
        info!("Running migration 1: Initial schema");
        migration_v1(conn)?;
    }
    
    if current_version < 2 {
        info!("Running migration 2: Hard idempotency enforcement");
        migration_v2(conn)?;
    }
    
    Ok(())
}

/// Migration v1: Create initial schema
fn migration_v1(conn: &Connection) -> Result<(), MigrationError> {
    // Users table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            display_name TEXT,
            email TEXT,
            metadata TEXT NOT NULL DEFAULT '{}',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Channels table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            channel_name TEXT NOT NULL,
            channel_identifier TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            metadata TEXT NOT NULL DEFAULT '{}',
            config TEXT NOT NULL DEFAULT '{}',
            last_message_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Sessions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            channel_id TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            metadata TEXT NOT NULL DEFAULT '{}',
            context TEXT NOT NULL DEFAULT '{}',
            last_activity_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (channel_id) REFERENCES channels(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Goals table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            parent_goal_id TEXT,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            priority TEXT NOT NULL DEFAULT 'medium',
            context TEXT NOT NULL DEFAULT '{}',
            result TEXT,
            started_at TEXT,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (session_id) REFERENCES sessions(id),
            FOREIGN KEY (parent_goal_id) REFERENCES goals(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Tool executions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_executions (
            id TEXT PRIMARY KEY,
            goal_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            tool_input TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            output TEXT,
            error TEXT,
            idempotency_key TEXT,
            started_at TEXT,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (goal_id) REFERENCES goals(id),
            FOREIGN KEY (session_id) REFERENCES sessions(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Create index on idempotency_key for duplicate detection
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tool_executions_idempotency 
         ON tool_executions(idempotency_key) WHERE idempotency_key IS NOT NULL",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Memories table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            memory_type TEXT NOT NULL,
            importance TEXT NOT NULL DEFAULT 'medium',
            content TEXT NOT NULL,
            embedding BLOB,
            source_session_id TEXT,
            source_tool_execution_id TEXT,
            tags TEXT NOT NULL DEFAULT '[]',
            expires_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id),
            FOREIGN KEY (source_session_id) REFERENCES sessions(id),
            FOREIGN KEY (source_tool_execution_id) REFERENCES tool_executions(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id)",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_goals_session ON goals(session_id)",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_user ON memories(user_id)",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Record migration
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (1, ?)",
        params![now],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    info!("Migration v1 completed successfully");
    Ok(())
}

/// Migration v2: Add UNIQUE constraint on idempotency_key for hard enforcement
fn migration_v2(conn: &Connection) -> Result<(), MigrationError> {
    // Create a new table with UNIQUE constraint on idempotency_key
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_executions_v2 (
            id TEXT PRIMARY KEY,
            goal_id TEXT NOT NULL,
            session_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            tool_input TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            output TEXT,
            error TEXT,
            idempotency_key TEXT UNIQUE,
            started_at TEXT,
            completed_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (goal_id) REFERENCES goals(id),
            FOREIGN KEY (session_id) REFERENCES sessions(id),
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Copy data from old table
    conn.execute(
        "INSERT OR IGNORE INTO tool_executions_v2 
         SELECT id, goal_id, session_id, user_id, tool_name, tool_input, status, output, error, 
                idempotency_key, started_at, completed_at, created_at, updated_at 
         FROM tool_executions",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Drop old table
    conn.execute("DROP TABLE tool_executions", [])
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Rename new table
    conn.execute("ALTER TABLE tool_executions_v2 RENAME TO tool_executions", [])
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Recreate index
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tool_executions_idempotency 
         ON tool_executions(idempotency_key) WHERE idempotency_key IS NOT NULL",
        [],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    // Record migration
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (2, ?)",
        params![now],
    ).map_err(|e| MigrationError::DatabaseError(e.to_string()))?;
    
    info!("Migration v2 completed successfully");
    Ok(())
}
