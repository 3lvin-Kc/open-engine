# Dashboard

OpenEngine includes a web-based dashboard to visualize tool executions, sessions, and statistics.

## Running the Dashboard

1. Start the OpenEngine server:
```bash
cd state-engine
cargo run -- serve --database openclaw_state.db
```

2. Open the dashboard in your browser:
```
dashboard/index.html
```

3. Enter your engine URL (default: `http://127.0.0.1:3030`) and click "Load Data"

## Features

### Stats Overview
- Total sessions
- Total tool executions
- Status breakdown (completed, failed, running, pending)

### Sessions List
- View all sessions
- See session status (active, paused, completed, failed)
- Click to view executions for a session

### Execution Timeline
- See all tool executions for a session
- View execution status (color-coded)
- Click any execution to see details:
  - Tool name
  - Status
  - Idempotency key (auto-generated if not provided)
  - Input/output
  - Error messages (if any)

## Key Features

### Auto-Generated Idempotency Keys
Every tool execution gets an automatic idempotency key:
```
auto:{tool_name}:{timestamp}:{hash}
```

This ensures:
- No duplicate tool executions
- Crash recovery works seamlessly
- Every tool call is tracked

### Real-Time Refresh
Click "Refresh" to reload data from the engine.
