# Web Scraper Example

Demonstrates how to use the OpenClaw State Engine to build a robust web scraper that:

- **Never scrapes the same URL twice** (idempotency)
- **Resumes after crashes** (continue where left off)
- **Tracks all scraped data** (full audit trail)
- **Handles rate limiting gracefully** (automatic retry)

## Quick Start

```bash
# Make sure state engine is running
cd ../state-engine && cargo run -- serve --database scraper.db

# In another terminal, run the example
cd examples/web-scraper
npm install
npm start
```

## What It Demonstrates

1. **URL Deduplication**: Each URL gets a unique idempotency key
2. **Crash Recovery**: Agent resumes knowing what was already scraped
3. **Batch Processing**: Scrape multiple URLs with progress tracking
4. **Link Discovery**: Extract links and crawl them automatically
5. **Memory Storage**: Store scraped content for later retrieval

## Output Example

```
╔════════════════════════════════════════════════════════╗
║        Web Scraper Agent with State Engine             ║
╚════════════════════════════════════════════════════════╝

PHASE 1: Initial scrape of all URLs
══════════════════════════════════════════════════════════

[1/5] Processing URL: https://example.com/products
🎯 Goal created
✅ Scraped: "products"

...

PHASE 2: Simulate crash + resume (same URLs)
══════════════════════════════════════════════════════════

💥 [SIMULATION] Agent crashes and restarts...
📂 Resumed session
📋 Checking for pending scrapes...
Already scraped: 5 URLs

[1/5] Processing URL: https://example.com/products
⏩ Skipping already-scraped URL

...

✅ Demo complete!
💡 Key takeaways:
   - Same URLs were skipped on retry (idempotent)
   - Agent resumed with knowledge of already-scraped URLs
```

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────┐
│   Web Scraper   │────>│  State Engine    │────>│   SQLite    │
│    Agent        │     │  (Rust backend)  │     │   Database  │
└─────────────────┘     └──────────────────┘     └─────────────┘
         │
         └────> fetchPage() (simulated HTTP)
```

## Production Use

Replace `fetchPage()` with actual HTTP library:

```typescript
import axios from 'axios';
import * as cheerio from 'cheerio';

async function fetchPage(url: string) {
  const response = await axios.get(url);
  const $ = cheerio.load(response.data);
  
  return {
    url,
    title: $('title').text(),
    content: $('body').text(),
    links: $('a').map((_, el) => $(el).attr('href')).get()
  };
}
```
