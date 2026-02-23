# API Version 0.1 (Stable)

**Status: FROZEN** — No breaking changes will be made to this API surface.

## Stable Models

The following models are frozen and will not change in v0.1:

| Model | Fields | Status |
|-------|--------|--------|
| User | id, username, display_name, email, metadata, active | FROZEN |
| Session | id, user_id, channel_id, status, metadata, context, last_activity_at | FROZEN |
| Goal | id, user_id, session_id, parent_goal_id, title, description, status, priority, context, result | FROZEN |
| ToolExecution | id, goal_id, session_id, user_id, tool_name, tool_input, status, output, error, idempotency_key | FROZEN |
| Memory | id, user_id, memory_type, importance, content, tags, expires_at | FROZEN |
| Channel | id, user_id, channel_type, channel_name, channel_identifier, status | FROZEN |

## Stable Endpoints

These endpoints are guaranteed to work in v0.1:

### User
- `create_user` — Create a new user
- `get_user` — Retrieve user by ID

### Session
- `create_session` — Start a new session
- `get_session` — Retrieve session by ID
- `get_active_session` — Get active session for user
- `update_session` — Update session state

### Goal
- `create_goal` — Create a goal
- `get_goal` — Retrieve goal by ID
- `update_goal` — Update goal state
- `list_pending_goals` — List pending goals for session

### Tool Execution
- `create_tool_execution` — Log tool execution
- `get_tool_execution` — Retrieve execution by ID
- `find_by_idempotency_key` — Find by idempotency key
- `update_tool_execution` — Update execution state
- `list_tool_executions` — List executions for goal

### Memory
- `create_memory` — Store a memory
- `get_memory` — Retrieve memory by ID
- `update_memory` — Update memory
- `list_memories` — List memories for user

### Channel
- `create_channel` — Create a channel
- `get_channel` — Retrieve channel by ID
- `update_channel` — Update channel
- `list_channels` — List channels for user

## What Agent Failures Does This Prevent?

| Failure | How Engine Prevents It |
|---------|----------------------|
| Lost context on restart | All state persisted to SQLite with WAL |
| Duplicate tool executions | Idempotency key lookup before execution |
| Orphaned goals | Goals linked to sessions, foreign key constraints |
| Forgotten objectives | Goals persist until explicitly completed |
| No audit trail | Every tool execution logged with timestamps |

## Versioning Policy

- **v0.1**: Current stable release
- **v0.2**: May add new endpoints, will not break existing ones
- **v1.0**: First production release

## Adding New Endpoints

New endpoints will only be added if they:
1. Fix a real agent failure case
2. Cannot be composed from existing endpoints
3. Have a clear use case documented

**Last Updated**: 2026-02-23
