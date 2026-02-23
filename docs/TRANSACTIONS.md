# Transaction Guarantees

## Atomic Operations

All state changes in the engine are atomic. This means:

### Single Operations
Every `create_*`, `update_*` method is a single transaction:
- Either fully succeeds
- Or fully fails
- No partial state possible

### Multi-Step Operations

The `execute_tool_idempotent` endpoint is atomic:

```
1. Check if idempotency_key exists
2. If yes → return existing execution
3. If no → create new execution
```

This is a single database transaction. Two agents calling simultaneously with the same key will NOT create duplicates - one will get the existing record.

## Write-Ahead Logging (WAL)

The database uses SQLite WAL mode:

- Writes are appended to a log first
- Log is checkpointed to main database
- Crash during write = log replay on restart
- No corrupted state possible

## Foreign Key Constraints

All relationships are enforced:

| Table | Constraint |
|-------|-----------|
| sessions | Must reference valid user_id |
| goals | Must reference valid session_id, user_id |
| tool_executions | Must reference valid goal_id, session_id, user_id |
| memories | Must reference valid user_id |
| channels | Must reference valid user_id |

This prevents orphan records.

## Idempotency Guarantee

When you call `execute_tool_idempotent` with an idempotency_key:

1. Database enforces UNIQUE constraint on the key
2. Transaction checks for existing key before insert
3. If key exists, existing record is returned
4. If key doesn't exist, new record is created
5. This is atomic - no race conditions possible

### What This Prevents

| Scenario | Without Engine | With Engine |
|----------|---------------|-------------|
| Network timeout, retry | Duplicate execution | Same record returned |
| Agent crash, restart | Lost progress, re-execute | Resume from saved state |
| Concurrent requests | Race condition, duplicates | One wins, others get same result |

## Crash Safety

If the server crashes:

1. On restart, WAL is replayed
2. Any incomplete transactions are rolled back
3. Committed transactions are preserved
4. Database is in consistent state

No manual recovery needed.
