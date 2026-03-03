# OpenClaw State Engine

**Persistent State & Session Management for AI Agents**

[![Status](https://img.shields.io/badge/status-working-brightgreen)]()
[![API](https://img.shields.io/badge/API-v0.1-blue)](docs/API_VERSION.md)
[![Idempotency](https://img.shields.io/badge/idempotency-tested_100_concurrent-green)]()

## Why This Exists

AI agents fail in production because they forget everything on restart. This engine gives agents durable memory that survives crashes.

**What it prevents:**
- Duplicate tool executions (any tool, not just emails)
- Lost progress on restart (agent forgets what it was doing)
- Untraceable actions ("why did the agent do that?")
- Corrupted state from partial failures

→ [Read the full story](docs/WHY_THIS_EXISTS.md)

## What This Is (and Is NOT)

### This IS:

- **A durability layer** — State survives crashes, restarts, and failures
- **A correctness layer** — Idempotency guarantees, atomic transactions
- **A recovery layer** — Resume interrupted work, replay sessions
- **An audit layer** — Every action tracked, traceable, explainable

### This is NOT:

- ❌ A vector database — No embeddings, no similarity search
- ❌ An agent framework — No orchestration, no reasoning, no planning
- ❌ A memory summarizer — No compression, no forgetting, no prioritization
- ❌ A message queue — No pub/sub, no async delivery
- ❌ A workflow engine — No DAGs, no branching logic

**If you need those things, use them alongside this engine.**

This engine does ONE thing well: **ensures agent state is durable and correct.**

## Quick Start

### 1. Start the Engine

```bash
cd state-engine
cargo run -- serve --database openclaw_state.db
```

Server runs on `http://127.0.0.1:3030`

### 2. Use the Client

```typescript
import { StateEngineClient } from '@openclaw/ts-client';

const client = new StateEngineClient({ url: 'http://127.0.0.1:3030' });

// Create user and session
const user = await client.createUser('my-agent');
const session = await client.createSession(user.id);

// Create a goal
const goal = await client.createGoal(user.id, session.id, 'Send email');

// Execute with idempotency - prevents duplicates!
const exec = await client.executeToolIdempotent(
  user.id, session.id, goal.id,
  'send_email',
  { to: 'user@example.com', subject: 'Hello' },
  'email-user123-welcome'  // Same key = same result, always
);
```

## Killer Feature: Automatic Idempotency

The `execute_tool_idempotent` method guarantees no duplicate executions - **without needing to provide a key**:

```typescript
// Auto-generated idempotency key (server creates one for you)
const exec = await client.executeToolIdempotent(
  user.id, session.id, goal.id,
  'any_tool',
  { input: 'value' }
  // No idempotency_key needed - server generates:
  // "auto:any_tool:20240315:abc123" from hash of inputs
);

// Same inputs = same auto-generated key = no duplicates!
```

Or provide your own key:
```typescript
await client.executeToolIdempotent(..., 'my-custom-key');
```

This is atomic and race-condition safe. **Tested with 100 concurrent calls — all returned the same execution ID.**

## CLI Commands

### Start Server

```bash
openclaw-state-engine serve --database state.db --addr 127.0.0.1:3030
```

### Replay Session (Audit/Debug)

```bash
openclaw-state-engine replay --database state.db --session <session-id>
```

Output:
```
╔════════════════════════════════════════════════════════════════╗
║                    SESSION REPLAY                              ║
╚════════════════════════════════════════════════════════════════╝

Session ID:     eedf0c5c-83bb-4674-940d-25918c64fb93
User ID:        adac540e-1edb-4b23-9a7a-a525394c69c5
Status:         Active
Created:        2026-02-23 12:48:34 UTC

┌─────────────────────────────────────────────────────────────────┐
│ GOALS (1)                                                        │
└─────────────────────────────────────────────────────────────────┘

  Goal #1: Send welcome email
  Status:      Pending
  Priority:    Medium

  ┌── TOOL EXECUTIONS (1) ──
  │
  │  Tool:      send_email
  │  Status:    Pending
  │  Input:     {"subject":"Welcome!","to":"user@example.com"}
  │  Idempotency Key: welcome-email-001
  │  Time:      12:48:34
  └───────────────────────────

══════════════════════════════════════════════════════════════════
Replay complete. 1 goal(s), 1 execution(s) total.
```

## Failure Contract

If you use this engine, these failures will not happen:

| Guarantee | What It Prevents |
|-----------|------------------|
| Tool executions are idempotent | Duplicate emails, double charges |
| Goals survive restart | Lost work, repeated effort |
| Duplicate actions detectable | Accidental retries |
| State transitions auditable | "Why did this happen?" debugging |
| Atomic writes | Corrupted state, orphan records |

→ [Read the full contract](docs/FAILURE_CONTRACT.md)

## Examples & Demos

### Failure Demo

Shows before/after comparison of agent failures:

```bash
cd examples/failure-demo
node demo.js
```

**What you'll see:**
- WITHOUT engine: Duplicate emails sent on retry
- WITH engine: Same key returns same result, no duplicates

### Email Agent Example

Complete integration example:

```bash
cd examples/email-agent
npm install
npm start
```

## Integrations

### OpenClaw

Drop-in adapter for OpenClaw agents — adds durability to any agent:

```bash
cd integrations/openclaw
npm install
```

```typescript
import { OpenClawAgent } from './adapter';

const agent = new OpenClawAgent({
  userId: 'my-agent',
  engineUrl: 'http://127.0.0.1:3030'
});

await agent.init();

// Crash-safe, duplicate-safe tool execution - auto-tracked!
const result = await agent.execute('send_email', {
  to: 'user@example.com',
  subject: 'Hello'
});

// Every tool automatically tracked - no manual key needed
// Server auto-generates idempotency key from tool+inputs
```

→ [Read full docs](integrations/openclaw/README.md)

## Testing

### Stress Test Idempotency

```bash
# Start server
cd state-engine && cargo run -- serve --database test.db

# Run stress test (100 concurrent calls)
cd tests/idempotency-stress
npm test
```

**Results:**
```
✅ Test 1 (Concurrent Calls):    PASS - 100 calls → 1 execution ID
✅ Test 2 (Different Keys):      PASS - 10 keys → 10 executions
✅ Test 3 (Persist After Restart): PASS - Same result after restart
```

## Dashboard

Visualize tool executions, sessions, and statistics with the web dashboard:

```bash
# 1. Start the engine
cd state-engine && cargo run -- serve --database openclaw_state.db

# 2. Open dashboard in browser
open dashboard/index.html
```

**Features:**
- Session overview with status
- Tool execution timeline
- Execution details (input, output, errors)
- Auto-generated idempotency keys visible

![Dashboard](docs/dashboard.png)

→ [Full dashboard docs](docs/DASHBOARD.md)

## Documentation

| Document | Description |
|----------|-------------|
| [API Version](docs/API_VERSION.md) | Frozen v0.1 API surface |
| [Failure Contract](docs/FAILURE_CONTRACT.md) | What failures this prevents |
| [Dashboard](docs/DASHBOARD.md) | Web dashboard for visualization |
| [Transactions](docs/TRANSACTIONS.md) | Atomic guarantees explained |
| [Why This Exists](docs/WHY_THIS_EXISTS.md) | Non-technical explanation |

## API Methods

| Method | Description |
|--------|-------------|
| `create_user` | Create a new user |
| `get_user` | Get user by ID |
| `create_session` | Start a new session |
| `get_session` | Get session by ID |
| `get_active_session` | Get active session for user |
| `update_session` | Update session |
| `create_goal` | Create a goal |
| `get_goal` | Get goal by ID |
| `update_goal` | Update goal |
| `list_pending_goals` | List pending goals |
| `create_tool_execution` | Log tool execution |
| `execute_tool_idempotent` | ⭐ Idempotent execution |
| `get_tool_execution` | Get execution by ID |
| `find_by_idempotency_key` | Find by idempotency key |
| `update_tool_execution` | Update execution |
| `list_tool_executions` | List executions for goal |
| `create_memory` | Create a memory |
| `get_memory` | Get memory by ID |
| `update_memory` | Update memory |
| `list_memories` | List memories for user |
| `create_channel` | Create a channel |
| `get_channel` | Get channel by ID |
| `update_channel` | Update channel |
| `list_channels` | List channels for user |

## Architecture

```
┌─────────────┐
│ AI Agent    │ ─────► ┌──────────────┐ ─────► ┌────────────────┐
└─────────────┘        │ ts-client    │        │ State Engine  │
                       │ (TypeScript) │        │ (Rust + SQLite)│
                       └──────────────┘        └────────────────┘
                                                       │
                                               ┌───────▼───────┐
                                               │  SQLite WAL   │
                                               │  (durable)    │
                                               └───────────────┘
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| Database | SQLite with WAL mode |
| API | JSON-RPC 2.0 (jsonrpsee) |
| Client | TypeScript |
| Idempotency | UNIQUE constraint + atomic transactions |

## Project Structure

```
open-engine/
├── state-engine/           # Rust backend
│   ├── src/
│   │   ├── models/         # Data models
│   │   ├── persistence/    # SQLite repositories
│   │   ├── api/            # JSON-RPC endpoints
│   │   └── main.rs         # CLI entry point
│   └── Cargo.toml
├── ts-client/              # TypeScript client
│   ├── src/
│   │   ├── client.ts       # API client
│   │   └── types.ts        # TypeScript types
│   └── package.json
├── dashboard/               # Web dashboard (HTML/CSS/JS)
├── examples/
│   ├── email-agent/        # Complete integration example
│   └── failure-demo/       # Before/after failure demo
├── tests/
│   └── idempotency-stress/ # Stress test suite
├── docs/
│   ├── API_VERSION.md      # Frozen API surface
│   ├── FAILURE_CONTRACT.md # Guarantees
│   ├── TRANSACTIONS.md     # Atomic guarantees
│   ├── DASHBOARD.md       # Dashboard docs
│   └── WHY_THIS_EXISTS.md  # Non-technical explanation
└── README.md
```

## License

MIT
