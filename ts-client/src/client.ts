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
}
