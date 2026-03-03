# OpenClaw Integration for Open Engine

Makes OpenClaw agents durable and reliable with idempotency guarantees.

## What This Does

- **Idempotent tool execution** — Same action = same result, prevents duplicates
- **Crash recovery** — Resume interrupted work after restart
- **Full audit trail** — Track every action your agent performs
- **Session management** — Track goals and tool executions per session

## Quick Start

### 1. Start Open Engine Server

```bash
cd state-engine
cargo run -- serve --database openclaw.db
# Runs on http://127.0.0.1:3030
```

### 2. Start OpenClaw Gateway

```bash
openclaw gateway start
# Runs on http://127.0.0.1:18789
```

### 3. Use the Adapter

```typescript
import { OpenClawAgent } from './adapter';

// Create agent with both Open Engine and OpenClaw configs
const agent = new OpenClawAgent({
  userId: 'my-user',
  engineUrl: 'http://127.0.0.1:3030',      // Open Engine server
  openclawUrl: 'http://127.0.0.1:18789',    // OpenClaw Gateway
  openclawToken: 'YOUR_OPENCLAW_TOKEN'       // Your auth token
});

// Initialize - creates user and session in Open Engine
await agent.init();

// Execute tools - idempotency key is AUTO-GENERATED!
const result = await agent.execute('exec', {
  command: 'npm install'
}, {
  goalName: 'Setup project'
  // No idempotencyKey needed - server auto-generates it!
});

console.log(result);
// { success: true, data: {...}, executionId: '...' }
```

## Key Features

### Auto-Generated Idempotency Keys (NEW!)

Every tool execution is automatically tracked with an auto-generated idempotency key. No manual key needed!

```typescript
// Just execute - server auto-generates key from tool+inputs
await agent.execute('send_email', { to: 'user@example.com' });
// Key format: "auto:send_email:20240315:abc123"

// Same tool + same inputs = same execution (no duplicates!)
await agent.execute('send_email', { to: 'user@example.com' });
// Returns cached result!
```

You can still provide a custom key if needed:
```typescript
await agent.execute('send_email', { to: 'user@example.com' }, {
  idempotencyKey: 'my-custom-key'
});
```

### Session Resume

After crash/restart, your agent resumes where it left off:

```typescript
// On startup - auto-resumes last active session
await agent.init();

// Or resume a specific session
await agent.resumeSession('session-id-from-database');
```

### Audit Trail

Get full history of what your agent did:

```typescript
const audit = await agent.getSessionAudit();
console.log(audit);
// {
//   session: {...},
//   goals: [...],
//   executions: [...]
// }
```

## API

### OpenClawAgent

```typescript
new OpenClawAgent(config: {
  userId: string;           // Your user identifier
  engineUrl: string;        // Open Engine server URL
  openclawUrl?: string;     // OpenClaw Gateway URL (default: http://127.0.0.1:18789)
  openclawToken?: string;   // OpenClaw auth token
})
```

### Methods

| Method | Description |
|--------|-------------|
| `init()` | Initialize agent, create user/session |
| `execute(tool, params, options)` | Execute tool with idempotency |
| `resumeSession(sessionId)` | Resume a specific session |
| `getSessionAudit()` | Get full audit trail |
| `listSessions()` | List all sessions for user |
| `listPendingGoals()` | Get pending goals in current session |

## Testing

```bash
cd integrations/openclaw
npm install
npm test
```

All 12 tests pass:
- OpenClaw Tools Invoke API integration
- Error handling (HTTP errors, network errors)
- Idempotency key generation
- Configuration defaults

## Architecture

```
┌──────────────┐     ┌─────────────────┐     ┌────────────────┐
│   You/       │────▶│   OpenClaw     │────▶│  Open Engine  │
│   Channels   │     │  (Gateway)     │     │  (This tool)  │
│  (WA/TG/..)  │◀────│  Tools + AI    │◀────│  Idempotency  │
└──────────────┘     └─────────────────┘     └────────────────┘
                                                      │
                                                      ▼
                                               ┌───────────┐
                                               │  SQLite   │
                                               │  (durable)│
                                               └───────────┘
```

## Requirements

- OpenClaw Gateway running (for actual tool execution)
- Open Engine server running (for state tracking)
- Auth token for OpenClaw Gateway

## License

MIT
