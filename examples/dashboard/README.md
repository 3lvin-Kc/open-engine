# OpenClaw State Engine Dashboard

A web-based dashboard for monitoring sessions, goals, and tool executions in real-time.

## Quick Start

```bash
# Make sure state engine is running
cd ../state-engine && cargo run -- serve --database dashboard.db

# In another terminal, run the dashboard
cd examples/dashboard
npm install
npm start

# Open browser to http://localhost:8080
```

## Features

- **Real-time Metrics**: View users, sessions, goals, and executions at a glance
- **Active Sessions List**: See all active sessions with their status
- **Pending Goals**: Monitor goals waiting to be completed
- **Quick Actions**: Create users, sessions, and run demo scenarios
- **API Health**: Connection status to the state engine

## Dashboard Overview

```
┌─────────────────────────────────────────────────────────┐
│  OpenClaw State Engine                                  │
├─────────────────────────────────────────────────────────┤
│  🟢 Connected to http://127.0.0.1:3030                  │
├─────────────────────────────────────────────────────────┤
│  👤 Users    │  📂 Sessions  │  🎯 Goals   │  ⚡ Execs │
│     5        │      3        │      2      │     12   │
├─────────────────────────────────────────────────────────┤
│  Quick Actions                                          │
│  [Create User] [Create Session] [Run Demo] [Refresh]   │
├─────────────────────────────────────────────────────────┤
│  Active Sessions        │  Pending Goals               │
│  ┌─────────────────┐  │  ┌─────────────────┐         │
│  │ session-abc...  │  │  │ Send email      │         │
│  │ active          │  │  │ completed       │         │
│  └─────────────────┘  │  └─────────────────┘         │
└─────────────────────────────────────────────────────────┘
```

## Configuration

Set environment variables:

```bash
# Dashboard port (default: 8080)
export PORT=3000

# State engine URL (default: http://127.0.0.1:3030)
export API_URL=http://localhost:3030

npm start
```

## API Proxy

The dashboard includes a built-in API proxy to avoid CORS issues:

```javascript
// Frontend calls this
fetch('/api/proxy', {
  method: 'POST',
  body: JSON.stringify({
    method: 'create_user',
    params: { username: 'test' }
  })
});

// Backend proxies to state engine
http.request('http://127.0.0.1:3030', ...);
```

## Development

The dashboard is a simple Express server serving static HTML with vanilla JavaScript. No build step required.

To extend:
1. Edit `public/index.html` for UI changes
2. Edit `server.js` for API changes
3. Refresh browser to see changes
