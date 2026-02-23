# Agent Failure Contract

If you use this engine, these failures will not happen.

---

## Guarantee 1: Tool Executions Are Idempotent

**What fails without this:**
An agent crashes after calling `send_email`. On restart, it calls `send_email` again. User receives duplicate emails.

**What the engine guarantees:**
When you provide an `idempotency_key`, the engine will:
- Return the existing execution if that key was already used
- Never execute the same tool twice for the same key

**How to use:**
```
create_tool_execution(..., idempotency_key="email-user-123-welcome")
```

---

## Guarantee 2: Agent Does Not Forget Goals on Restart

**What fails without this:**
Agent is working on goal "Book flight to NYC". Server crashes. On restart, agent has no memory of the goal and starts over.

**What the engine guarantees:**
- All goals are persisted immediately
- Goals can be queried by session or status
- Goal state (pending, in_progress, completed) survives any crash

**How to use:**
```
goal = create_goal(user_id, session_id, "Book flight to NYC")
# ... agent crashes ...
goals = list_pending_goals(session_id)  # Goal is still there
```

---

## Guarantee 3: Duplicate Actions Are Detectable

**What fails without this:**
Agent calls `create_calendar_event` three times because of network retries. User sees three duplicate events.

**What the engine guarantees:**
- Every tool execution is logged with unique ID
- You can check if an action was already taken before executing
- Full history of all attempted actions

**How to use:**
```
existing = find_by_idempotency_key("calendar-event-2024-03-15")
if existing:
    return existing.output  # Don't execute again
```

---

## Guarantee 4: State Transitions Are Auditable

**What fails without this:**
Agent says it completed a task, but you have no proof. User asks "why did you do X?" and you can't answer.

**What the engine guarantees:**
- Every state change is timestamped
- Tool executions link to goals, goals link to sessions
- Full causal chain: User → Session → Goal → Tool Execution

**How to use:**
```
executions = list_tool_executions(goal_id)
for e in executions:
    print(f"{e.tool_name} at {e.started_at} -> {e.status}")
```

---

## Guarantee 5: Partial Failures Don't Corrupt State

**What fails without this:**
Agent creates a goal, then crashes before linking it to a session. You have an orphan goal with no context.

**What the engine guarantees:**
- All writes are atomic transactions
- Foreign key constraints prevent orphans
- Crash during write leaves database in consistent state

**How this works:**
- Write-ahead logging (WAL) ensures durability
- Transactions are all-or-nothing
- Database is never in partially-updated state

---

## Summary

| # | Guarantee | Real Bug This Prevents |
|---|-----------|----------------------|
| 1 | Idempotent tool executions | Duplicate emails, double charges |
| 2 | Goals survive restart | Lost work, repeated effort |
| 3 | Duplicate detection | Accidental retries, wasted API calls |
| 4 | Audit trail | "Why did this happen?" debugging |
| 5 | Atomic writes | Corrupted state, orphan records |

---

**This contract is versioned with the API.** Any breaking change to these guarantees requires a major version bump.
