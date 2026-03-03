# Agent Failure Contract

If you use this engine, these failures will not happen.

---

## Guarantee 1: ALL Tool Executions Are Idempotent

**What fails without this:**
An agent crashes after calling a tool (any tool). On restart, it calls the same tool again. Results in duplicate API calls, wasted resources, or corrupted state.

**What the engine guarantees:**
Every tool execution is automatically tracked with an idempotency key. The engine will:
- Return the existing execution if that tool+inputs already ran
- Never execute the same tool twice for the same inputs
- Works for ANY tool: API calls, database writes, file operations, etc.

**How to use:**
```typescript
// Just call execute() - no manual key needed
const result = await agent.execute("any_tool", { input: "value" });

// Server auto-generates idempotency key from:
// user_id + session_id + goal_id + tool_name + tool_input
```

---

## Guarantee 2: Agent Does Not Forget Goals on Restart

**What fails without this:**
Agent is working on goal "Process user data". Server crashes. On restart, agent has no memory of the goal and starts over.

**What the engine guarantees:**
- All goals are persisted immediately
- Goals can be queried by session or status
- Goal state (pending, in_progress, completed) survives any crash

**How to use:**
```
goal = create_goal(user_id, session_id, "Process user data")
# ... agent crashes ...
goals = list_pending_goals(session_id)  # Goal is still there
```

---

## Guarantee 3: All Actions Are Trackable

**What fails without this:**
Agent calls a tool multiple times because of network retries or bugs. No way to know what actually ran.

**What the engine guarantees:**
- Every tool execution is logged with unique ID
- You can check if an action was already taken before executing
- Full history of all attempted actions

**How to use:**
```
# Find if this specific tool call already happened
existing = find_by_idempotency_key("auto:web_search:20240315:abc123")
if existing:
    return existing.output  # Don't execute again
```

---

## Guarantee 4: State Transitions Are Auditable

**What fails without this:**
Agent says it completed a task, but you have no proof. User asks "what did you do?" and you can't answer.

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
| 1 | ALL tool executions idempotent | Any duplicate tool call |
| 2 | Goals survive restart | Lost work, repeated effort |
| 3 | All actions trackable | Unknown what ran |
| 4 | Audit trail | "Why did this happen?" debugging |
| 5 | Atomic writes | Corrupted state, orphan records |

---

**This contract is versioned with the API.** Any breaking change to these guarantees requires a major version bump.
