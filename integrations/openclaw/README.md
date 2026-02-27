# OpenClaw Integration for Open Engine

Makes OpenClaw agents durable and reliable.

## What This Does

- **Survives crashes**: Session state persists, resume where you left off
- **Prevents duplicates**: Idempotent tool execution — same action = same result
- **Full audit trail**: Every action tracked and replayable
- **Goal tracking**: Break work into trackable goals with progress

## Quick Start

```typescript
import { OpenClawAgent } from './adapter';

// 1. Start Open Engine server
// cargo run -- serve --database openclaw.db

// 2. Create agent
const agent = new OpenClawAgent({
  userId: 'my-agent',
  engineUrl: 'http://127.0.0.1:3030'
});

await agent.init('my-session');

// 3. Execute tools with durability
const result = await agent.execute('web_search', {
  query: 'rust programming'
}, {
  goalName: 'Research Rust',
  idempotencyKey: 'search-rust-20250227'
});

// Crash? Restart? Same key = same result, no duplicate search
```

## Integration Architecture

```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│ OpenClaw   │────>│   Adapter   │────>│  Open Engine   │
│  Agent     │     │  (this)      │     │  (Rust core)   │
└─────────────┘     └──────────────┘     └────────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │   SQLite    │
                     │   (durable) │
                     └──────────────┘
```

## TODO

- [ ] Implement actual OpenClaw tool invocation
- [ ] Add session auto-resume on startup
- [ ] Create dashboard view for active sessions
- [ ] Add multi-agent coordination support

## License

MIT
