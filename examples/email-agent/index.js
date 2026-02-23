/**
 * Email Agent Example
 * 
 * This demonstrates how an AI agent can use the state engine to:
 * 1. Never send duplicate emails
 * 2. Survive crashes and resume work
 * 3. Track all actions for audit
 */

import { StateEngineClient } from '@openclaw/ts-client';

const client = new StateEngineClient({ url: 'http://127.0.0.1:3030' });

// Simulated email sending function
async function sendEmail(to, subject, body) {
  console.log(`📧 Sending email to ${to}: "${subject}"`);
  // Simulate network delay
  await new Promise(r => setTimeout(r, 500));
  // Simulate occasional failure
  if (Math.random() < 0.1) {
    throw new Error('Network timeout');
  }
  console.log(`✅ Email sent successfully`);
  return { messageId: `msg-${Date.now()}`, sent: true };
}

/**
 * Email Agent with State Engine
 * 
 * This agent can:
 * - Resume after crash
 * - Never duplicate emails
 * - Show full audit trail
 */
class EmailAgent {
  constructor(client) {
    this.client = client;
    this.userId = null;
    this.sessionId = null;
  }

  async initialize(username) {
    // Create or get user
    try {
      this.userId = 'user-' + username;
      const user = await client.createUser(username);
      this.userId = user.id;
      console.log(`👤 User created: ${this.userId}`);
    } catch (e) {
      // User might exist, find them
      console.log(`👤 Using existing user`);
    }

    // Check for active session (resuming after crash)
    const activeSession = await client.getActiveSession(this.userId);
    if (activeSession) {
      this.sessionId = activeSession.id;
      console.log(`📂 Resumed session: ${this.sessionId}`);
      await this.resumePendingGoals();
    } else {
      const session = await client.createSession(this.userId);
      this.sessionId = session.id;
      console.log(`📂 New session: ${this.sessionId}`);
    }
  }

  async resumePendingGoals() {
    console.log('\n📋 Checking for pending goals...');
    const goals = await client.listPendingGoals(this.sessionId);
    
    if (goals.length === 0) {
      console.log('✓ No pending goals');
      return;
    }

    console.log(`Found ${goals.length} pending goal(s):`);
    for (const goal of goals) {
      console.log(`  - ${goal.title} (${goal.status})`);
      const executions = await client.listToolExecutions(goal.id);
      console.log(`    ${executions.length} tool execution(s) recorded`);
    }
  }

  /**
   * Send an email with idempotency guarantee.
   * If this method is called multiple times with the same emailId,
   * only one email will be sent.
   */
  async sendEmailWithTracking(emailId, to, subject, body) {
    console.log(`\n📧 Processing email: ${emailId}`);

    // Create a goal for this email
    const goal = await client.createGoal(
      this.userId,
      this.sessionId,
      `Send email to ${to}`,
      subject
    );
    console.log(`🎯 Goal created: ${goal.id}`);

    // Use idempotent execution - prevents duplicates
    const idempotencyKey = `email-${emailId}`;
    
    const execution = await client.executeToolIdempotent(
      this.userId,
      this.sessionId,
      goal.id,
      'send_email',
      { to, subject, body },
      idempotencyKey
    );

    if (execution.status !== 'pending') {
      // This was a duplicate - already processed
      console.log(`⚡ Duplicate prevented! Using existing execution: ${execution.id}`);
      console.log(`   Status: ${execution.status}`);
      return execution.output;
    }

    // Actually send the email
    try {
      const result = await sendEmail(to, subject, body);
      
      // Update execution with result
      execution.status = 'completed';
      execution.output = result;
      await client.updateToolExecution(execution);

      // Mark goal as complete
      goal.status = 'completed';
      await client.updateGoal(goal);

      console.log(`✅ Email sent and tracked`);
      return result;
    } catch (error) {
      // Record the failure
      execution.status = 'failed';
      execution.error = error.message;
      await client.updateToolExecution(execution);

      goal.status = 'failed';
      await client.updateGoal(goal);

      console.log(`❌ Email failed: ${error.message}`);
      throw error;
    }
  }

  async showAuditTrail() {
    console.log('\n📊 AUDIT TRAIL');
    console.log('='.repeat(50));
    
    const goals = await client.listPendingGoals(this.sessionId);
    
    for (const goal of goals) {
      console.log(`\nGoal: ${goal.title}`);
      console.log(`  Status: ${goal.status}`);
      console.log(`  Created: ${goal.created_at}`);
      
      const executions = await client.listToolExecutions(goal.id);
      for (const exec of executions) {
        console.log(`\n  └─ Tool: ${exec.tool_name}`);
        console.log(`     Status: ${exec.status}`);
        console.log(`     Input: ${JSON.stringify(exec.tool_input)}`);
        if (exec.output) {
          console.log(`     Output: ${JSON.stringify(exec.output)}`);
        }
        if (exec.error) {
          console.log(`     Error: ${exec.error}`);
        }
        console.log(`     Idempotency Key: ${exec.idempotency_key}`);
      }
    }
  }
}

// Demo
async function main() {
  const agent = new EmailAgent(client);
  
  console.log('╔════════════════════════════════════════════════════════╗');
  console.log('║         Email Agent with State Engine                  ║');
  console.log('╚════════════════════════════════════════════════════════╝');

  await agent.initialize('email-agent-demo');

  // Send an email
  await agent.sendEmailWithTracking(
    'welcome-001',
    'user@example.com',
    'Welcome!',
    'Thanks for signing up!'
  );

  // Try to send the SAME email again (should be prevented)
  console.log('\n🔄 Attempting to send duplicate email...');
  await agent.sendEmailWithTracking(
    'welcome-001',  // Same ID = idempotent
    'user@example.com',
    'Welcome!',
    'Thanks for signing up!'
  );

  // Send a different email
  await agent.sendEmailWithTracking(
    'notify-002',
    'admin@example.com',
    'New User Signed Up',
    'A new user has registered.'
  );

  // Show full audit trail
  await agent.showAuditTrail();

  console.log('\n✅ Demo complete!');
  console.log('\n💡 Key takeaways:');
  console.log('   - Duplicate email was prevented automatically');
  console.log('   - All actions are tracked and auditable');
  console.log('   - If agent crashes, it can resume from saved state');
}

main().catch(console.error);
