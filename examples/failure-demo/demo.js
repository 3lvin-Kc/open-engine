/**
 * FAILURE DEMO
 * 
 * This script demonstrates real agent failures and how the state engine prevents them.
 * 
 * Run with: node demo.js
 */

// ============================================
// PART 1: AGENT WITHOUT STATE ENGINE
// ============================================

console.log('╔════════════════════════════════════════════════════════════════╗');
console.log('║          PART 1: Agent WITHOUT State Engine                    ║');
console.log('╚════════════════════════════════════════════════════════════════╝\n');

// Simulated in-memory agent state (LOST on crash)
let inMemoryState = {
  emailsSent: [],
  currentTask: null,
  executionCount: 0
};

// Simulated email sending
async function sendEmailWithoutEngine(to, subject) {
  console.log(`📧 [NO ENGINE] Sending email to ${to}: "${subject}"`);
  await new Promise(r => setTimeout(r, 100)); // Simulate network delay
  inMemoryState.emailsSent.push({ to, subject, time: Date.now() });
  inMemoryState.executionCount++;
  console.log(`✅ [NO ENGINE] Email #${inMemoryState.executionCount} sent`);
  return { success: true, messageId: `msg-${Date.now()}` };
}

// Scenario 1: Network timeout + retry
console.log('┌─ Scenario 1: Network Timeout + Retry ─────────────────────────┐');

async function scenario1WithoutEngine() {
  const emailId = 'welcome-user-123';
  
  // First attempt
  console.log('\nAttempt 1:');
  await sendEmailWithoutEngine('user@example.com', 'Welcome!');
  
  // Simulate: Agent thinks it failed (timeout), retries
  console.log('\n[Agent thinks: "Did that work? Network timed out... Let me retry"]\n');
  console.log('Attempt 2 (retry):');
  await sendEmailWithoutEngine('user@example.com', 'Welcome!');
  
  console.log('\n❌ RESULT: User received 2 emails!');
  console.log(`   Emails in memory: ${inMemoryState.emailsSent.length}`);
}

await scenario1WithoutEngine();

// Scenario 2: Crash mid-execution
console.log('\n└────────────────────────────────────────────────────────────────┘\n');
console.log('┌─ Scenario 2: Crash Mid-Execution ──────────────────────────────┐');

async function scenario2WithoutEngine() {
  inMemoryState = { emailsSent: [], currentTask: null, executionCount: 0 };
  
  console.log('\n[Agent starts task: "Send 3 welcome emails"]\n');
  inMemoryState.currentTask = 'Send 3 welcome emails';
  
  console.log('Email 1:');
  await sendEmailWithoutEngine('user1@example.com', 'Welcome!');
  
  console.log('\nEmail 2:');
  await sendEmailWithoutEngine('user2@example.com', 'Welcome!');
  
  console.log('\n💥 [CRASH] Server restarts!\n');
  
  // Simulate crash - memory is wiped
  inMemoryState = { emailsSent: [], currentTask: null, executionCount: 0 };
  
  console.log('[Agent restarts with no memory of what happened]\n');
  console.log(`   Current task: ${inMemoryState.currentTask}`);
  console.log(`   Emails sent: ${inMemoryState.emailsSent.length}`);
  
  console.log('\n[Agent starts over from beginning...]\n');
  console.log('Email 1 (again):');
  await sendEmailWithoutEngine('user1@example.com', 'Welcome!');
  console.log('Email 2 (again):');
  await sendEmailWithoutEngine('user2@example.com', 'Welcome!');
  console.log('Email 3:');
  await sendEmailWithoutEngine('user3@example.com', 'Welcome!');
  
  console.log('\n❌ RESULT: user1 and user2 got duplicate emails!');
  console.log('   Agent wasted effort redoing work already done.');
}

await scenario2WithoutEngine();

console.log('\n└────────────────────────────────────────────────────────────────┘\n');

// Wait for user to see results
console.log('Press Enter to continue to PART 2 (with State Engine)...');
await new Promise(resolve => {
  process.stdin.once('data', resolve);
});

// ============================================
// PART 2: AGENT WITH STATE ENGINE
// ============================================

console.log('\n\n╔════════════════════════════════════════════════════════════════╗');
console.log('║          PART 2: Agent WITH State Engine                       ║');
console.log('╚════════════════════════════════════════════════════════════════╝\n');

// We'll simulate the state engine behavior for this demo
// In production, you'd use the actual client

const stateEngine = {
  executions: new Map(),
  
  async executeIdempotent(idempotencyKey, action) {
    if (this.executions.has(idempotencyKey)) {
      const existing = this.executions.get(idempotencyKey);
      console.log(`⚡ [ENGINE] Duplicate prevented! Key "${idempotencyKey}" already used.`);
      console.log(`   Returning existing result from ${new Date(existing.timestamp).toISOString()}`);
      return { result: existing.result, wasDuplicate: true };
    }
    
    const result = await action();
    this.executions.set(idempotencyKey, { result, timestamp: Date.now() });
    return { result, wasDuplicate: false };
  }
};

// Simulated email sending WITH state engine
async function sendEmailWithEngine(to, subject, emailId) {
  const idempotencyKey = `email-${emailId}`;
  
  return stateEngine.executeIdempotent(idempotencyKey, async () => {
    console.log(`📧 [ENGINE] Sending email to ${to}: "${subject}"`);
    await new Promise(r => setTimeout(r, 100));
    console.log(`✅ [ENGINE] Email sent`);
    return { success: true, messageId: `msg-${Date.now()}` };
  });
}

// Scenario 1: Network timeout + retry (WITH ENGINE)
console.log('┌─ Scenario 1: Network Timeout + Retry (WITH ENGINE) ───────────┐');

async function scenario1WithEngine() {
  const emailId = 'welcome-user-456';
  
  console.log('\nAttempt 1:');
  const result1 = await sendEmailWithEngine('user@example.com', 'Welcome!', emailId);
  console.log(`   Was duplicate: ${result1.wasDuplicate}`);
  
  console.log('\n[Agent thinks: "Did that work? Network timed out... Let me retry"]\n');
  console.log('Attempt 2 (retry):');
  const result2 = await sendEmailWithEngine('user@example.com', 'Welcome!', emailId);
  console.log(`   Was duplicate: ${result2.wasDuplicate}`);
  
  console.log('\n✅ RESULT: User received exactly 1 email!');
  console.log('   Second call returned existing result without re-executing.');
}

await scenario1WithEngine();

// Scenario 2: Crash mid-execution (WITH ENGINE)
console.log('\n└────────────────────────────────────────────────────────────────┘\n');
console.log('┌─ Scenario 2: Crash Mid-Execution (WITH ENGINE) ────────────────┐');

async function scenario2WithEngine() {
  // Reset state engine for clean demo
  stateEngine.executions.clear();
  
  console.log('\n[Agent starts task: "Send 3 welcome emails"]\n');
  
  console.log('Email 1:');
  await sendEmailWithEngine('user1@example.com', 'Welcome!', 'email-user1-welcome');
  
  console.log('\nEmail 2:');
  await sendEmailWithEngine('user2@example.com', 'Welcome!', 'email-user2-welcome');
  
  console.log('\n💥 [CRASH] Server restarts!\n');
  
  console.log('[Agent restarts and queries state engine...]\n');
  
  // Check what was already done
  const user1Done = stateEngine.executions.has('email-email-user1-welcome');
  const user2Done = stateEngine.executions.has('email-email-user2-welcome');
  
  console.log(`   Email 1 already sent: ${user1Done}`);
  console.log(`   Email 2 already sent: ${user2Done}`);
  
  console.log('\n[Agent resumes from where it left off...]\n');
  
  // Retry user1 and user2 (will be blocked as duplicates)
  console.log('Email 1 (check):');
  await sendEmailWithEngine('user1@example.com', 'Welcome!', 'email-user1-welcome');
  
  console.log('\nEmail 2 (check):');
  await sendEmailWithEngine('user2@example.com', 'Welcome!', 'email-user2-welcome');
  
  // Only send user3
  console.log('\nEmail 3:');
  await sendEmailWithEngine('user3@example.com', 'Welcome!', 'email-user3-welcome');
  
  console.log('\n✅ RESULT: No duplicates! All 3 users got exactly 1 email.');
  console.log('   Agent correctly resumed from crash point.');
}

await scenario2WithEngine();

console.log('\n└────────────────────────────────────────────────────────────────┘\n');

// ============================================
// SUMMARY
// ============================================

console.log('\n╔════════════════════════════════════════════════════════════════╗');
console.log('║                        SUMMARY                                 ║');
console.log('╚════════════════════════════════════════════════════════════════╝\n');

console.log('┌─────────────────────────────────────────────────────────────────┐');
console.log('│ WITHOUT State Engine                                            │');
console.log('├─────────────────────────────────────────────────────────────────┤');
console.log('│ ❌ Duplicate emails sent on retry                               │');
console.log('│ ❌ Crash = amnesia, work lost                                   │');
console.log('│ ❌ Restart = start over from beginning                          │');
console.log('│ ❌ No way to know what already happened                         │');
console.log('└─────────────────────────────────────────────────────────────────┘\n');

console.log('┌─────────────────────────────────────────────────────────────────┐');
console.log('│ WITH State Engine                                               │');
console.log('├─────────────────────────────────────────────────────────────────┤');
console.log('│ ✅ Retries return existing result (no duplicates)               │');
console.log('│ ✅ Crash = pause, not reset                                     │');
console.log('│ ✅ Resume from exact point of failure                           │');
console.log('│ ✅ Full visibility into what happened before                    │');
console.log('└─────────────────────────────────────────────────────────────────┘\n');

console.log('The difference is STATE that survives failures.\n');
