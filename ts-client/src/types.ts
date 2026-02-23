export interface EntityBase {
  id: string;
  created_at: string;
  updated_at: string;
}

export interface User extends EntityBase {
  username: string;
  display_name: string | null;
  email: string | null;
  metadata: Record<string, unknown>;
  active: boolean;
}

export interface Session extends EntityBase {
  user_id: string;
  channel_id: string | null;
  status: SessionStatus;
  metadata: Record<string, unknown>;
  context: Record<string, unknown>;
  last_activity_at: string;
}

export type SessionStatus = "active" | "paused" | "completed" | "failed" | "abandoned";

export interface Goal extends EntityBase {
  user_id: string;
  session_id: string;
  parent_goal_id: string | null;
  title: string;
  description: string | null;
  status: GoalStatus;
  priority: GoalPriority;
  context: Record<string, unknown>;
  result: Record<string, unknown> | null;
  started_at: string | null;
  completed_at: string | null;
}

export type GoalStatus = "pending" | "in_progress" | "completed" | "failed" | "cancelled";
export type GoalPriority = "low" | "medium" | "high" | "critical";

export interface ToolExecution extends EntityBase {
  goal_id: string;
  session_id: string;
  user_id: string;
  tool_name: string;
  tool_input: Record<string, unknown>;
  status: ToolExecutionStatus;
  output: Record<string, unknown> | null;
  error: string | null;
  idempotency_key: string | null;
  started_at: string | null;
  completed_at: string | null;
}

export type ToolExecutionStatus = "pending" | "running" | "completed" | "failed";

export interface Memory extends EntityBase {
  user_id: string;
  memory_type: MemoryType;
  importance: MemoryImportance;
  content: string;
  embedding: number[] | null;
  source_session_id: string | null;
  source_tool_execution_id: string | null;
  tags: string[];
  expires_at: string | null;
}

export type MemoryType = "short_term" | "long_term" | "episodic";
export type MemoryImportance = "low" | "medium" | "high" | "critical";

export interface Channel extends EntityBase {
  user_id: string;
  channel_type: ChannelType;
  channel_name: string;
  channel_identifier: string;
  metadata: Record<string, unknown>;
}

export type ChannelType = "discord" | "slack" | "telegram" | "web" | "api";
