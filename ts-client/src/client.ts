export interface StateEngineClientOptions {
  url: string;
}

interface JSONRPCRequest {
  jsonrpc: "2.0";
  method: string;
  params?: Record<string, unknown>;
  id: number;
}

interface JSONRPCResponse {
  jsonrpc: "2.0";
  id: number;
  result?: unknown;
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
}

export class StateEngineClient {
  private url: string;
  private id: number = 0;

  constructor(options: StateEngineClientOptions) {
    this.url = options.url;
  }

  private async call<T>(method: string, params?: Record<string, unknown>): Promise<T> {
    const request: JSONRPCRequest = {
      jsonrpc: "2.0",
      method,
      params,
      id: ++this.id,
    };

    const response = await fetch(this.url, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`HTTP error: ${response.status}`);
    }

    const jsonResult = await response.json();
    const result = jsonResult as JSONRPCResponse;

    if (result.error) {
      throw new Error(`JSON-RPC error: ${result.error.message} (code: ${result.error.code})`);
    }

    return result.result as T;
  }

  async createUser(
    username: string,
    displayName?: string,
    email?: string
  ): Promise<import("./types").User> {
    return this.call("create_user", {
      username,
      display_name: displayName ?? null,
      email: email ?? null,
    });
  }

  async getUser(id: string): Promise<import("./types").User> {
    return this.call("get_user", { id });
  }

  async createChannel(
    userId: string,
    channelType: string,
    channelName: string,
    channelIdentifier: string
  ): Promise<import("./types").Channel> {
    return this.call("create_channel", {
      user_id: userId,
      channel_type: channelType,
      channel_name: channelName,
      channel_identifier: channelIdentifier,
    });
  }

  async getChannel(id: string): Promise<import("./types").Channel> {
    return this.call("get_channel", { id });
  }

  async updateChannel(channel: import("./types").Channel): Promise<void> {
    return this.call("update_channel", { channel });
  }

  async listChannels(userId: string): Promise<import("./types").Channel[]> {
    return this.call("list_channels", { user_id: userId });
  }

  async createSession(
    userId: string,
    channelId?: string
  ): Promise<import("./types").Session> {
    return this.call("create_session", {
      user_id: userId,
      channel_id: channelId ?? null,
    });
  }

  async getSession(id: string): Promise<import("./types").Session> {
    return this.call("get_session", { id });
  }

  async getActiveSession(userId: string): Promise<import("./types").Session | null> {
    return this.call("get_active_session", { user_id: userId });
  }

  async updateSession(session: import("./types").Session): Promise<void> {
    return this.call("update_session", { session });
  }

  async createGoal(
    userId: string,
    sessionId: string,
    title: string,
    description?: string
  ): Promise<import("./types").Goal> {
    return this.call("create_goal", {
      user_id: userId,
      session_id: sessionId,
      title,
      description: description ?? null,
    });
  }

  async getGoal(id: string): Promise<import("./types").Goal> {
    return this.call("get_goal", { id });
  }

  async updateGoal(goal: import("./types").Goal): Promise<void> {
    return this.call("update_goal", { goal });
  }

  async listPendingGoals(sessionId: string): Promise<import("./types").Goal[]> {
    return this.call("list_pending_goals", { session_id: sessionId });
  }

  async createToolExecution(
    userId: string,
    sessionId: string,
    goalId: string,
    toolName: string,
    toolInput: Record<string, unknown>
  ): Promise<import("./types").ToolExecution> {
    return this.call("create_tool_execution", {
      user_id: userId,
      session_id: sessionId,
      goal_id: goalId,
      tool_name: toolName,
      tool_input: toolInput,
    });
  }

  async executeToolIdempotent(
    userId: string,
    sessionId: string,
    goalId: string,
    toolName: string,
    toolInput: Record<string, unknown>,
    idempotencyKey: string
  ): Promise<import("./types").ToolExecution> {
    return this.call("execute_tool_idempotent", {
      user_id: userId,
      session_id: sessionId,
      goal_id: goalId,
      tool_name: toolName,
      tool_input: toolInput,
      idempotency_key: idempotencyKey,
    });
  }

  async getToolExecution(id: string): Promise<import("./types").ToolExecution> {
    return this.call("get_tool_execution", { id });
  }

  async findByIdempotencyKey(key: string): Promise<import("./types").ToolExecution | null> {
    return this.call("find_by_idempotency_key", { key });
  }

  async updateToolExecution(execution: import("./types").ToolExecution): Promise<void> {
    return this.call("update_tool_execution", { execution });
  }

  async listToolExecutions(goalId: string): Promise<import("./types").ToolExecution[]> {
    return this.call("list_tool_executions", { goal_id: goalId });
  }

  async createMemory(
    userId: string,
    content: string,
    importance?: string,
    tags?: string[]
  ): Promise<import("./types").Memory> {
    return this.call("create_memory", {
      user_id: userId,
      content,
      importance: importance ?? null,
      tags: tags ?? null,
    });
  }

  async getMemory(id: string): Promise<import("./types").Memory> {
    return this.call("get_memory", { id });
  }

  async updateMemory(memory: import("./types").Memory): Promise<void> {
    return this.call("update_memory", { memory });
  }

  async listMemories(userId: string, limit?: number): Promise<import("./types").Memory[]> {
    return this.call("list_memories", {
      user_id: userId,
      limit: limit ?? null,
    });
  }

  /**
   * Execute multiple tools in a batch with automatic tracking.
   * Each tool execution is tracked under a shared parent goal.
   * 
   * Example:
   * ```typescript
   * const results = await client.executeBatch(
   *   userId,
   *   sessionId,
   *   'Send welcome package',
   *   [
   *     { tool: 'send_email', input: { to: 'user@example.com' }, idempotencyKey: 'welcome-email' },
   *     { tool: 'update_db', input: { status: 'welcomed' }, idempotencyKey: 'db-update' },
   *     { tool: 'notify_slack', input: { message: 'New user!' }, idempotencyKey: 'slack-notify' }
   *   ]
   * );
   * ```
   */
  async executeBatch(
    userId: string,
    sessionId: string,
    batchName: string,
    tools: Array<{
      tool: string;
      input: Record<string, unknown>;
      idempotencyKey?: string;
    }>,
    options?: {
      continueOnError?: boolean;
      stopOnDuplicate?: boolean;
    }
  ): Promise<{
    goalId: string;
    results: Array<{
      tool: string;
      status: 'completed' | 'failed' | 'skipped';
      execution?: import("./types").ToolExecution;
      error?: string;
    }>;
    completed: number;
    failed: number;
    skipped: number;
  }> {
    // Create parent goal for the batch
    const goal = await this.createGoal(
      userId,
      sessionId,
      batchName,
      `Batch execution of ${tools.length} tool(s)`
    );

    const results: Array<{
      tool: string;
      status: 'completed' | 'failed' | 'skipped';
      execution?: import("./types").ToolExecution;
      error?: string;
    }> = [];

    let completed = 0;
    let failed = 0;
    let skipped = 0;

    for (const tool of tools) {
      try {
        let execution: import("./types").ToolExecution;

        if (tool.idempotencyKey) {
          // Check if already executed
          const existing = await this.findByIdempotencyKey(tool.idempotencyKey);
          if (existing) {
            if (options?.stopOnDuplicate) {
              results.push({
                tool: tool.tool,
                status: 'skipped',
                execution: existing,
                error: 'Duplicate - stopped on duplicate'
              });
              skipped++;
              break;
            }
            results.push({
              tool: tool.tool,
              status: existing.status === 'completed' ? 'completed' : 'failed',
              execution: existing
            });
            if (existing.status === 'completed') completed++;
            else failed++;
            continue;
          }

          execution = await this.executeToolIdempotent(
            userId,
            sessionId,
            goal.id,
            tool.tool,
            tool.input,
            tool.idempotencyKey
          );
        } else {
          execution = await this.createToolExecution(
            userId,
            sessionId,
            goal.id,
            tool.tool,
            tool.input
          );
        }

        // Mark as completed (actual tool execution is separate)
        execution.status = 'completed';
        await this.updateToolExecution(execution);

        results.push({
          tool: tool.tool,
          status: 'completed',
          execution
        });
        completed++;

      } catch (error) {
        results.push({
          tool: tool.tool,
          status: 'failed',
          error: error instanceof Error ? error.message : String(error)
        });
        failed++;

        if (!options?.continueOnError) {
          break;
        }
      }
    }

    // Update goal status
    if (failed === 0) {
      goal.status = 'completed';
    } else if (completed > 0) {
      goal.status = 'completed'; // Partial success
    } else {
      goal.status = 'failed';
    }
    await this.updateGoal(goal);

    return {
      goalId: goal.id,
      results,
      completed,
      failed,
      skipped
    };
  }

  /**
   * Resume a previously created session.
   * Returns null if no session exists or session is not active.
   */
  async resumeSession(userId: string): Promise<import("./types").Session | null> {
    const session = await this.getActiveSession(userId);
    if (session && session.status === 'active') {
      // Update last activity
      session.last_activity_at = new Date().toISOString();
      await this.updateSession(session);
      return session;
    }
    return null;
  }

  /**
   * List all goals for a session with their execution counts.
   */
  async getSessionOverview(sessionId: string): Promise<{
    goals: Array<{
      goal: import("./types").Goal;
      executions: import("./types").ToolExecution[];
    }>;
    totalGoals: number;
    pendingGoals: number;
    completedGoals: number;
    failedGoals: number;
  }> {
    const goals = await this.listPendingGoals(sessionId);

    const goalsWithExecutions = await Promise.all(
      goals.map(async (goal) => ({
        goal,
        executions: await this.listToolExecutions(goal.id)
      }))
    );

    return {
      goals: goalsWithExecutions,
      totalGoals: goals.length,
      pendingGoals: goals.filter(g => g.status === 'pending').length,
      completedGoals: goals.filter(g => g.status === 'completed').length,
      failedGoals: goals.filter(g => g.status === 'failed').length
    };
  }
}
