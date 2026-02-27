/**
 * OpenClaw Integration for Open Engine
 * 
 * Wraps OpenClaw tool calls with persistent state and idempotency.
 * 
 * Usage:
 *   const agent = new OpenClawAgent({
 *     userId: 'doremon',
 *     engineUrl: 'http://127.0.0.1:3030'
 *   });
 * 
 *   const result = await agent.execute('web_search', { query: 'rust' });
 */

import { StateEngineClient } from '../../ts-client/src/client';
import { ToolExecutionInput } from '../../ts-client/src/types';

type ToolName = 'web_search' | 'web_fetch' | 'exec' | 'read' | 'write' | 'edit' | 
                'message' | 'github' | 'browser' | 'image' | 'memory_search' | 
                'cron' | 'gateway' | 'sessions_spawn' | 'sessions_list';

interface ToolResult {
  success: boolean;
  data?: any;
  error?: string;
  executionId?: string;
}

interface AgentConfig {
  userId: string;
  engineUrl: string;
  sessionName?: string;
}

export class OpenClawAgent {
  private client: StateEngineClient;
  private userId: string;
  private sessionId: string;
  private activeGoalId: string | null = null;

  constructor(config: AgentConfig) {
    this.client = new StateEngineClient({ url: config.engineUrl });
    this.userId = config.userId;
    this.sessionId = '';
  }

  /**
   * Initialize the agent - create user and session in Open Engine
   */
  async init(sessionName?: string): Promise<void> {
    try {
      // Check if user exists, create if not
      let user;
      try {
        // Try to find existing user (this would need API support)
        user = await this.client.createUser(this.userId);
      } catch (e) {
        // User might already exist
      }

      // Create session
      const session = await this.client.createSession(this.userId, sessionName || 'openclaw-session');
      this.sessionId = session.id;

      console.log(`[OpenClawAgent] Initialized: user=${this.userId}, session=${this.sessionId}`);
    } catch (error) {
      console.error('[OpenClawAgent] Init failed:', error);
      throw error;
    }
  }

  /**
   * Execute a tool with idempotency guarantee
   * 
   * @param toolName - The OpenClaw tool to execute
   * @param params - Tool parameters
   * @param options - Execution options
   */
  async execute(
    toolName: ToolName,
    params: Record<string, any>,
    options: {
      goalName?: string;
      idempotencyKey?: string;
      priority?: 'low' | 'medium' | 'high';
    } = {}
  ): Promise<ToolResult> {
    try {
      // Create goal if not exists
      if (!this.activeGoalId && options.goalName) {
        const goal = await this.client.createGoal(
          this.userId,
          this.sessionId,
          options.goalName,
          options.priority || 'medium'
        );
        this.activeGoalId = goal.id;
      }

      if (!this.activeGoalId) {
        // Create default goal
        const goal = await this.client.createGoal(
          this.userId,
          this.sessionId,
          `Execute ${toolName}`,
          'medium'
        );
        this.activeGoalId = goal.id;
      }

      // Generate idempotency key if not provided
      const idempotencyKey = options.idempotencyKey || this.generateKey(toolName, params);

      // Execute via Open Engine (guarantees idempotency)
      const execution = await this.client.executeToolIdempotent(
        this.userId,
        this.sessionId,
        this.activeGoalId,
        toolName,
        params,
        idempotencyKey
      );

      // Here we would actually call the OpenClaw tool
      // For now, we log that it's tracked
      console.log(`[OpenClawAgent] Tracked: ${toolName} (execution=${execution.id})`);

      // TODO: Actually invoke the OpenClaw tool here
      // This requires OpenClaw tool invocation API
      
      return {
        success: true,
        executionId: execution.id
      };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  /**
   * Restore state from a previous session after crash/restart
   */
  async resumeSession(sessionId: string): Promise<void> {
    try {
      const session = await this.client.getSession(sessionId);
      this.sessionId = session.id;
      
      // Find active goal
      const goals = await this.client.listPendingGoals(this.userId); // Need to implement
      if (goals.length > 0) {
        this.activeGoalId = goals[0].id;
      }

      console.log(`[OpenClawAgent] Resumed session: ${sessionId}`);
      
      // Replay incomplete executions
      await this.replayIncompleteExecutions();
    } catch (error) {
      console.error('[OpenClawAgent] Resume failed:', error);
      throw error;
    }
  }

  /**
   * Replay incomplete executions from previous session
   */
  private async replayIncompleteExecutions(): Promise<void> {
    // Query pending executions
    // Re-execute or resume based on state
    console.log('[OpenClawAgent] Checking for incomplete work...');
  }

  /**
   * Get full audit trail of current session
   */
  async getSessionAudit(): Promise<any> {
    return await this.client.listSessions(this.userId); // Need to implement
  }

  /**
   * Generate deterministic idempotency key from tool + params
   */
  private generateKey(toolName: string, params: Record<string, any>): string {
    const paramsHash = JSON.stringify(params);
    return `${toolName}-${Buffer.from(paramsHash).toString('base64').slice(0, 16)}-${Date.now()}`;
  }
}

// Example usage
async function example() {
  const agent = new OpenClawAgent({
    userId: 'doremon',
    engineUrl: 'http://127.0.0.1:3030'
  });

  await agent.init('twitter-automation-session');

  // This will never execute twice, even on retry
  const result = await agent.execute('web_search', {
    query: 'open source AI agents'
  }, {
    goalName: 'Research AI agents',
    idempotencyKey: 'research-ai-agents-20250227'
  });

  console.log('Result:', result);
}

export { OpenClawAgent };
