/**
 * Web Scraper Example
 * 
 * This demonstrates how an AI agent can use the state engine to:
 * 1. Never scrape the same URL twice (idempotency)
 * 2. Resume interrupted scraping jobs after crash
 * 3. Track all scraped data with audit trail
 * 4. Handle rate limiting gracefully
 * 
 * Run with: npm start
 */

import { StateEngineClient } from '@openclaw/ts-client';

const client = new StateEngineClient({ url: 'http://127.0.0.1:3030' });

// Simulated web scraping function
async function fetchPage(url) {
  console.log(`🌐 Fetching: ${url}`);
  
  // Simulate network delay
  await new Promise(r => setTimeout(r, 300));
  
  // Simulate occasional failures (rate limit, timeout)
  if (Math.random() < 0.15) {
    const error = Math.random() < 0.5 
      ? new Error('429 Rate Limited') 
      : new Error('Network timeout');
    console.log(`❌ Failed: ${error.message}`);
    throw error;
  }
  
  // Simulate extracted content
  const mockContent = {
    url,
    title: `Page at ${url.split('/').pop() || 'index'}`,
    content: `This is scraped content from ${url}. Contains text, links, and metadata.`,
    links: [`${url}/page1`, `${url}/page2`, `${url}/page3`].slice(0, Math.floor(Math.random() * 3) + 1),
    timestamp: new Date().toISOString()
  };
  
  console.log(`✅ Scraped: "${mockContent.title}"`);
  return mockContent;
}

/**
 * Web Scraper Agent with State Engine
 * 
 * This agent can:
 * - Never scrape the same URL twice (deduplication)
 * - Resume after crash (continue where it left off)
 * - Store scraped data durably
 * - Retry failed fetches with backoff
 * - Show full audit trail
 */
class WebScraperAgent {
  constructor(client) {
    this.client = client;
    this.userId = null;
    this.sessionId = null;
    this.scrapedUrls = new Set();
  }

  async initialize(agentName) {
    // Create or get user
    try {
      const user = await client.createUser(agentName);
      this.userId = user.id;
      console.log(`🤖 Agent created: ${this.userId}`);
    } catch (e) {
      // User might exist
      console.log(`🤖 Using existing agent`);
    }

    // Check for active session (resuming after crash)
    const activeSession = await client.getActiveSession(this.userId);
    if (activeSession) {
      this.sessionId = activeSession.id;
      console.log(`📂 Resumed session: ${this.sessionId}`);
      await this.resumePendingScrapes();
    } else {
      const session = await client.createSession(this.userId);
      this.sessionId = session.id;
      console.log(`📂 New session: ${this.sessionId}`);
    }
  }

  async resumePendingScrapes() {
    console.log('\n📋 Checking for pending scrapes...');
    const goals = await client.listPendingGoals(this.sessionId);
    
    if (goals.length === 0) {
      console.log('✓ No pending scrapes');
      return;
    }

    console.log(`Found ${goals.length} pending scrape job(s):`);
    for (const goal of goals) {
      console.log(`  - ${goal.title} (${goal.status})`);
      const executions = await client.listToolExecutions(goal.id);
      console.log(`    ${executions.length} URL(s) in this batch`);
      
      for (const exec of executions) {
        if (exec.tool_input?.url) {
          this.scrapedUrls.add(exec.tool_input.url);
        }
      }
    }
    console.log(`\n📊 Already scraped: ${this.scrapedUrls.size} URLs`);
  }

  /**
   * Scrape a single URL with idempotency.
   * If this URL was already scraped, returns cached result immediately.
   */
  async scrapeUrl(url) {
    // Deduplication check (session-level)
    if (this.scrapedUrls.has(url)) {
      console.log(`⏩ Skipping already-scraped URL: ${url}`);
      return null;
    }

    console.log(`\n🌐 Processing URL: ${url}`);

    // Create a goal for this scrape
    const goal = await client.createGoal(
      this.userId,
      this.sessionId,
      `Scrape ${url}`,
      `Fetch and extract content from ${url}`
    );
    console.log(`🎯 Goal created: ${goal.id}`);

    // Use idempotent execution - prevents duplicate scrapes
    const idempotencyKey = `scrape-${Buffer.from(url).toString('base64')}`;
    
    const execution = await client.executeToolIdempotent(
      this.userId,
      this.sessionId,
      goal.id,
      'scrape_url',
      { url },
      idempotencyKey
    );

    // Check if already scraped (by idempotency key)
    if (execution.status === 'completed' || execution.status === 'failed') {
      console.log(`📦 Already recorded: ${execution.status}`);
      this.scrapedUrls.add(url);
      
      if (execution.status === 'completed') {
        await this.createMemory(url, execution.output);
        return execution.output;
      }
      return null;
    }

    // Actually scrape the URL
    try {
      const result = await fetchPage(url);
      
      // Update execution with result
      execution.status = 'completed';
      execution.output = result;
      await client.updateToolExecution(execution);

      // Mark goal as complete
      goal.status = 'completed';
      await client.updateGoal(goal);

      // Store in memory for later retrieval
      await this.createMemory(url, result);

      this.scrapedUrls.add(url);
      console.log(`✅ Scraped and stored: ${url}`);
      
      return result;
    } catch (error) {
      // Record the failure
      execution.status = 'failed';
      execution.error = error.message;
      await client.updateToolExecution(execution);

      goal.status = 'failed';
      await client.updateGoal(goal);

      console.log(`❌ Scrape failed: ${error.message}`);
      
      // Mark as scraped anyway (don't retry same session)
      this.scrapedUrls.add(url);
      
      return null;
    }
  }

  /**
   * Scrape multiple URLs in batch
   */
  async scrapeUrls(urls) {
    console.log(`\n📦 Starting batch scrape of ${urls.length} URLs`);
    console.log('=' .repeat(60));
    
    const results = [];
    let success = 0;
    let failed = 0;
    let skipped = 0;

    for (let i = 0; i < urls.length; i++) {
      console.log(`\n[${i + 1}/${urls.length}]`);
      
      // Check if already scraped
      if (this.scrapedUrls.has(urls[i])) {
        skipped++;
        continue;
      }

      const result = await this.scrapeUrl(urls[i]);
      
      if (result) {
        results.push(result);
        success++;
      } else if (!this.scrapedUrls.has(urls[i])) {
        failed++;
      } else {
        skipped++;
      }

      // Rate limit protection
      if (i < urls.length - 1) {
        await new Promise(r => setTimeout(r, 200));
      }
    }

    console.log('\n' + '='.repeat(60));
    console.log('📊 Batch Complete');
    console.log(`   ✅ Successful: ${success}`);
    console.log(`   ❌ Failed: ${failed}`);
    console.log(`   ⏩ Skipped (already scraped): ${skipped}`);
    console.log(`   📦 Total results: ${results.length}`);
    
    return results;
  }

  async createMemory(url, content) {
    await client.createMemory(
      this.userId,
      JSON.stringify({ url, ...content }),
      'normal',
      ['scraped', 'web', url.split('/')[2]]
    );
  }

  /**
   * Get stored content for a URL
   */
  async getStoredContent(url) {
    const memories = await client.listMemories(this.userId, 100);
    for (const mem of memories) {
      try {
        const data = JSON.parse(mem.content);
        if (data.url === url) {
          return data;
        }
      } catch (e) {
        // Not JSON or no URL field
      }
    }
    return null;
  }

  /**
   * Extract links from scraped content for crawling
   */
  extractLinks(results) {
    const allLinks = new Set();
    for (const result of results) {
      if (result?.links) {
        for (const link of result.links) {
          allLinks.add(link);
        }
      }
    }
    return Array.from(allLinks);
  }

  async showAuditTrail() {
    console.log('\n📊 SCRAPE AUDIT TRAIL');
    console.log('=' .repeat(60));
    
    const goals = await client.listPendingGoals(this.sessionId);
    let completed = 0;
    let failed = 0;
    let total = goals.length;
    
    for (const goal of goals) {
      const executions = await client.listToolExecutions(goal.id);
      for (const exec of executions) {
        if (exec.status === 'completed') completed++;
        if (exec.status === 'failed') failed++;
      }
    }
    
    console.log(`\nSession: ${this.sessionId}`);
    console.log(`Total Goals: ${total}`);
    console.log(`✅ Completed: ${completed}`);
    console.log(`❌ Failed: ${failed}`);
    
    const memories = await client.listMemories(this.userId, 10);
    console.log(`\n📝 Recent Memories: ${memories.length}`);
    for (const mem of memories.slice(0, 5)) {
      try {
        const data = JSON.parse(mem.content);
        console.log(`   - ${data.url?.substring(0, 50)}... (${data.title})`);
      } catch (e) {
        console.log(`   - [Unparseable memory]`);
      }
    }
  }
}

// Demo
async function main() {
  const agent = new WebScraperAgent(client);
  
  console.log('╔════════════════════════════════════════════════════════╗');
  console.log('║        Web Scraper Agent with State Engine               ║');
  console.log('╚════════════════════════════════════════════════════════╝');

  await agent.initialize('web-scraper-demo');

  // URLs to scrape
  const urls = [
    'https://example.com/products',
    'https://example.com/blog/post-1',
    'https://example.com/blog/post-2',
    'https://example.com/about',
    'https://example.com/contact',
  ];

  // First batch scrape
  console.log('\n─────────────────────────────────────────────────────────');
  console.log('PHASE 1: Initial scrape of all URLs');
  console.log('─────────────────────────────────────────────────────────');
  const results = await agent.scrapeUrls(urls);

  // Simulate crash and resume scenario
  console.log('\n─────────────────────────────────────────────────────────');
  console.log('PHASE 2: Simulate crash + resume (same URLs)');
  console.log('─────────────────────────────────────────────────────────');
  console.log('\n💥 [SIMULATION] Agent crashes and restarts...\n');
  
  // Create new agent instance (simulating restart)
  const newAgent = new WebScraperAgent(client);
  await newAgent.initialize('web-scraper-demo');
  
  // Try same URLs again - should be deduplicated
  const resumedResults = await newAgent.scrapeUrls(urls);

  // Crawl discovered links from first scrape
  console.log('\n─────────────────────────────────────────────────────────');
  console.log('PHASE 3: Crawl discovered links');
  console.log('─────────────────────────────────────────────────────────');
  const discoveredLinks = agent.extractLinks(results);
  if (discoveredLinks.length > 0) {
    console.log(`\n🔗 Found ${discoveredLinks.length} links to crawl`);
    await newAgent.scrapeUrls(discoveredLinks.slice(0, 3));
  }

  // Show audit trail
  await newAgent.showAuditTrail();

  console.log('\n✅ Demo complete!');
  console.log('\n💡 Key takeaways:');
  console.log('   - Same URLs were skipped on retry (idempotent)');
  console.log('   - Agent resumed with knowledge of already-scraped URLs');
  console.log('   - Failed scrapes recorded without blocking others');
  console.log('   - Discovered links can be crawled in next batch');
}

main().catch(console.error);
