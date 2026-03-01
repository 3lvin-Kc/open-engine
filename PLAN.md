# Open-Engine v1 Architecture Plan

## Features to Implement

### 1. Tool Schema Registry
- `register_tool_schema` - Register new tool with parameters
- `get_tool_schema` - Retrieve tool by name  
- `validate_tool_input` - Validate input against schema
- `list_tool_schemas` - List all registered tools

### 2. Auto-Retry with Backoff
- `RetryPolicy` struct with max_attempts, initial_delay_ms, backoff_multiplier
- `mark_for_retry` - Mark failed execution for retry
- `get_retryable_executions` - Get ready-to-retry items
- Automatic delay calculation: delay = initial * (multiplier ^ attempt)

### 3. Working Memory (Scratchpad)
- `create_scratchpad` - Create session temp storage
- `get_scratchpad` - Retrieve scratchpad content
- `update_scratchpad` - Update temp data
- Auto-cleanup on session end

### 4. Checkpoints
- `save_checkpoint` - Save snapshot of current state
- `restore_checkpoint` - Restore from snapshot
- `list_checkpoints` - List available checkpoints
- Checkpoint = {id, session_id, name, data, timestamp}

### 5. Multi-Agent Coordination
- `assign_goal_to_agent` - Assign goal to specific agent
- `release_goal` - Release goal assignment
- `list_agent_goals` - List goals assigned to agent
- Shared goal state across agents

### 6. Context Window Tracker
- `get_session_context` - What's loaded in context
- `update_context_window` - Track loaded items
- `evict_from_context` - Mark items as evicted
- Track context_size vs storage_size

### 7. TypeScript Client
- Full SDK matching Rust API
- Type-safe methods for all endpoints
- Error handling, retries
- Example usage

## Files to Modify

### New Files
- `src/models/tool_schema.rs` - Tool schema models
- `src/persistence/tool_schema_repo.rs` - Schema DB operations
- `src/models/scratchpad.rs` - Working memory models
- `src/persistence/scratchpad_repo.rs` - Scratchpad DB operations
- `src/models/checkpoint.rs` - Checkpoint models
- `src/persistence/checkpoint_repo.rs` - Checkpoint DB operations

### Modified Files
- `src/models/tool_execution.rs` - Add RetryState
- `src/models/mod.rs` - Export new modules
- `src/persistence/mod.rs` - Export new repos
- `src/persistence/migrations.rs` - Add v3 migration
- `src/api/state_rpc.rs` - Add all new endpoints
- `ts-client/` - Complete SDK

## Migration v3 Schema

```sql
-- Tool schemas
CREATE TABLE tool_schemas (
    id TEXT PRIMARY KEY,
    tool_name TEXT UNIQUE NOT NULL,
    description TEXT,
    version TEXT,
    parameters TEXT, -- JSON
    return_type TEXT,
    category TEXT,
    tags TEXT, -- JSON array
    enabled BOOLEAN DEFAULT 1,
    deprecated BOOLEAN DEFAULT 0,
    deprecation_message TEXT,
    created_at TEXT,
    updated_at TEXT
);

-- Scratchpad
CREATE TABLE scratchpads (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    name TEXT,
    content TEXT, -- JSON
    expires_at TEXT,
    created_at TEXT,
    updated_at TEXT
);

-- Checkpoints  
CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    name TEXT,
    snapshot TEXT, -- JSON state snapshot
    created_at TEXT
);

-- Context tracking
CREATE TABLE context_entries (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    entry_type TEXT, -- 'goal', 'memory', 'tool_exec'
    entry_id TEXT,
    in_context BOOLEAN,
    loaded_at TEXT,
    last_accessed TEXT
);

-- Add retry_state to tool_executions
ALTER TABLE tool_executions ADD COLUMN retry_state TEXT;
ALTER TABLE tool_executions ADD COLUMN retry_next_at TEXT;
```

## API Endpoints

### Tool Schema
- `register_tool_schema(schema)` → ToolSchema
- `get_tool_schema(tool_name)` → ToolSchema | null
- `validate_tool_input(tool_name, input)` → boolean
- `list_tool_schemas(category?, enabled?)` → ToolSchema[]

### Retry
- `mark_for_retry(execution_id, error, policy)` → ToolExecution
- `get_retryable_executions(session_id, limit?)` → ToolExecution[]

### Scratchpad
- `create_scratchpad(session_id, name, content)` → Scratchpad
- `get_scratchpad(session_id, name)` → Scratchpad | null
- `update_scratchpad(id, content)` → Scratchpad
- `delete_scratchpad(id)` → void

### Checkpoints
- `save_checkpoint(session_id, name, snapshot)` → Checkpoint
- `restore_checkpoint(checkpoint_id)` → Snapshot
- `list_checkpoints(session_id)` → Checkpoint[]
- `delete_checkpoint(id)` → void

### Multi-Agent
- `assign_goal_to_agent(goal_id, agent_id)` → Goal
- `release_goal(goal_id)` → Goal
- `list_agent_goals(agent_id)` → Goal[]

### Context
- `get_session_context(session_id)` → ContextEntry[]
- `update_context_window(session_id, entries)` → void
- `evict_from_context(session_id, entries)` → void

## Implementation Order
1. Migration v3 (add tables/columns)
2. Tool Schema models + repo
3. Tool Schema API endpoints
4. RetryState + retry methods
5. Retry API endpoints
6. Scratchpad models + repo + API
7. Checkpoint models + repo + API
8. Context tracker models + repo + API
9. Multi-agent coordination
10. TypeScript client
11. Write tests for all features
12. Build and verify
13. Push to GitHub

## Success Criteria
- `cargo build` passes with no warnings
- `cargo test` passes 100%
- All 7 features working
- Examples demonstrate usage
- README updated
- Ready for v1 release
