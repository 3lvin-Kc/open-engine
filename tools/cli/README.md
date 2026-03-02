# @openclaw/cli

Quick command-line tool for OpenClaw State Engine users.

## Install

```bash
cd tools/cli
npm install
npm link  # Makes `openclaw` command available globally
```

Or run without installing:
```bash
node bin/cli.js [command]
```

## Commands

### `openclaw quickstart`

One-command health check and setup guide.

```bash
openclaw quickstart
```

Shows:
- If state engine is running
- Next steps to get started
- Useful commands reference

### `openclaw health`

Check if the state engine is responding.

```bash
openclaw health
# or
openclaw health --url http://localhost:3030
```

### `openclaw init [project-name]`

Bootstrap a new OpenClaw project with boilerplate code.

```bash
# Create basic project
openclaw init my-project

# Create email agent template
openclaw init my-email-agent --template email

# Create web scraper template
openclaw init my-scraper --template scraper
```

**Templates:**
- `basic` - Minimal setup (default)
- `email` - Email sending with idempotency
- `scraper` - Web scraping with deduplication

### `openclaw generate-key [name]`

Generate idempotency keys for tool execution.

```bash
openclaw generate-key welcome-email
openclaw generate-key scrape-homepage --prefix crawl
```

Output:
```
🔑 Generated idempotency key:
   welcome-email-1709424000000-a3f9b2
```

**Why this matters:** Same key = same result, always. Prevents duplicate work.

### `openclaw run-example [name]`

Quick navigation to example projects.

```bash
openclaw run-example email-agent
openclaw run-example web-scraper
openclaw run-example dashboard
```

## Quick Workflow

```bash
# 1. Check that engine is running
openclaw health

# 2. Create a new project
openclaw init my-super-agent --template email
cd my-super-agent
npm install

# 3. Generate an idempotency key
openclaw generate-key welcome-user

# 4. Edit src/index.js to use your key
# 5. Run the agent
npm start
```

## Why This Exists

OpenClaw users asked for:
- "How do I start a new project?" → `openclaw init`
- "How do I check if it's working?" → `openclaw health`
- "How do I generate safe keys?" → `openclaw generate-key`
- "Show me the examples" → `openclaw run-example`

This CLI makes the state engine more approachable for new users.
