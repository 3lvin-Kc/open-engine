//! JSON-RPC state management API

use jsonrpsee::RpcModule;
use uuid::Uuid;

use crate::models::*;
use crate::persistence::Database;

pub struct StateRpc {
    pub db: Database,
}

impl StateRpc {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn create_user(
        &self,
        username: String,
        display_name: Option<String>,
        email: Option<String>,
    ) -> Result<User, jsonrpsee::types::ErrorObject<'static>> {
        let user = User::new(username)
            .with_display_name(display_name.unwrap_or_default())
            .with_email(email.unwrap_or_default());

        let repo = crate::persistence::UserRepository::new(self.db.clone());
        repo.create(&user).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(user)
    }

    pub fn get_user(&self, id: Uuid) -> Result<User, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::UserRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn create_channel(
        &self,
        user_id: Uuid,
        channel_type: ChannelType,
        channel_name: String,
        channel_identifier: String,
    ) -> Result<Channel, jsonrpsee::types::ErrorObject<'static>> {
        let channel = Channel::new(user_id, channel_type, channel_name, channel_identifier);
        let repo = crate::persistence::ChannelRepository::new(self.db.clone());
        repo.create(&channel).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(channel)
    }

    pub fn get_channel(&self, id: Uuid) -> Result<Channel, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ChannelRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn update_channel(
        &self,
        channel: Channel,
    ) -> Result<(), jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ChannelRepository::new(self.db.clone());
        repo.update(&channel).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn list_channels(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Channel>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ChannelRepository::new(self.db.clone());
        repo.list_for_user(user_id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn create_session(
        &self,
        user_id: Uuid,
        channel_id: Option<Uuid>,
    ) -> Result<Session, jsonrpsee::types::ErrorObject<'static>> {
        let mut session = Session::new(user_id);
        if let Some(cid) = channel_id {
            session = session.with_channel(cid);
        }
        let repo = crate::persistence::SessionRepository::new(self.db.clone());
        repo.create(&session).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(session)
    }

    pub fn get_session(&self, id: Uuid) -> Result<Session, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::SessionRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn get_active_session(
        &self,
        user_id: Uuid,
    ) -> Result<Option<Session>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::SessionRepository::new(self.db.clone());
        repo.get_active_for_user(user_id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn list_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Session>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::SessionRepository::new(self.db.clone());
        repo.list_for_user(user_id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn update_session(
        &self,
        session: Session,
    ) -> Result<(), jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::SessionRepository::new(self.db.clone());
        repo.update(&session).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn create_goal(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        title: String,
        description: Option<String>,
    ) -> Result<Goal, jsonrpsee::types::ErrorObject<'static>> {
        let goal =
            Goal::new(user_id, session_id, title).with_description(description.unwrap_or_default());
        let repo = crate::persistence::GoalRepository::new(self.db.clone());
        repo.create(&goal).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(goal)
    }

    pub fn get_goal(&self, id: Uuid) -> Result<Goal, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::GoalRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn update_goal(&self, goal: Goal) -> Result<(), jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::GoalRepository::new(self.db.clone());
        repo.update(&goal).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn list_pending_goals(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<Goal>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::GoalRepository::new(self.db.clone());
        repo.list_pending_for_session(session_id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn create_tool_execution(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        goal_id: Uuid,
        tool_name: String,
        tool_input: serde_json::Value,
    ) -> Result<ToolExecution, jsonrpsee::types::ErrorObject<'static>> {
        let execution = ToolExecution::new(user_id, session_id, goal_id, tool_name, tool_input);
        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        repo.create(&execution).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(execution)
    }

    /// Create a tool execution with hard idempotency enforcement.
    /// Returns existing execution if same idempotency_key was used before.
    pub fn execute_tool_idempotent(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        goal_id: Uuid,
        tool_name: String,
        tool_input: serde_json::Value,
        idempotency_key: String,
    ) -> Result<ToolExecution, jsonrpsee::types::ErrorObject<'static>> {
        let execution = ToolExecution::new(user_id, session_id, goal_id, tool_name, tool_input)
            .with_idempotency_key(idempotency_key);

        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        let result = repo.create_idempotent(&execution).map_err(|e| {
            jsonrpsee::types::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;

        match result {
            crate::persistence::IdempotentCreateResult::Created(exec) => Ok(exec),
            crate::persistence::IdempotentCreateResult::Duplicate(existing) => {
                tracing::info!(
                    "Duplicate tool execution prevented: {:?}",
                    existing.idempotency_key
                );
                Ok(existing)
            }
        }
    }

    pub fn get_tool_execution(
        &self,
        id: Uuid,
    ) -> Result<ToolExecution, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn find_by_idempotency_key(
        &self,
        key: String,
    ) -> Result<Option<ToolExecution>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        repo.find_by_idempotency_key(&key).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn update_tool_execution(
        &self,
        execution: ToolExecution,
    ) -> Result<(), jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        repo.update(&execution).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn list_tool_executions(
        &self,
        goal_id: Uuid,
    ) -> Result<Vec<ToolExecution>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::ToolExecutionRepository::new(self.db.clone());
        repo.list_for_goal(goal_id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn create_memory(
        &self,
        user_id: Uuid,
        content: String,
        importance: Option<MemoryImportance>,
        tags: Option<Vec<String>>,
    ) -> Result<Memory, jsonrpsee::types::ErrorObject<'static>> {
        let mut memory = Memory::new_short_term(user_id, content);
        if let Some(imp) = importance {
            memory = memory.with_importance(imp);
        }
        if let Some(t) = tags {
            memory = memory.with_tags(t);
        }
        let repo = crate::persistence::MemoryRepository::new(self.db.clone());
        repo.create(&memory).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })?;
        Ok(memory)
    }

    pub fn get_memory(&self, id: Uuid) -> Result<Memory, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::MemoryRepository::new(self.db.clone());
        repo.get(id).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn update_memory(
        &self,
        memory: Memory,
    ) -> Result<(), jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::MemoryRepository::new(self.db.clone());
        repo.update(&memory).map_err(|e| {
            jsonrpsee::types::error::ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                e.to_string(),
                None::<()>,
            )
        })
    }

    pub fn list_memories(
        &self,
        user_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<Memory>, jsonrpsee::types::ErrorObject<'static>> {
        let repo = crate::persistence::MemoryRepository::new(self.db.clone());
        repo.list_for_user(user_id, limit.unwrap_or(50))
            .map_err(|e| {
                jsonrpsee::types::error::ErrorObject::owned(
                    jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                    e.to_string(),
                    None::<()>,
                )
            })
    }
}

pub fn create_rpc_module(db: Database) -> RpcModule<StateRpc> {
    let state = StateRpc::new(db);
    let mut module = RpcModule::new(state);

    module
        .register_method("create_user", |params, state| {
            let p: CreateUserParams = params.parse()?;
            state.create_user(p.username, p.display_name, p.email)
        })
        .unwrap();

    module
        .register_method("get_user", |params, state| {
            let p: GetUserParams = params.parse()?;
            state.get_user(p.id)
        })
        .unwrap();

    module
        .register_method("create_channel", |params, state| {
            let p: CreateChannelParams = params.parse()?;
            state.create_channel(
                p.user_id,
                p.channel_type,
                p.channel_name,
                p.channel_identifier,
            )
        })
        .unwrap();

    module
        .register_method("get_channel", |params, state| {
            let p: GetChannelParams = params.parse()?;
            state.get_channel(p.id)
        })
        .unwrap();

    module
        .register_method("update_channel", |params, state| {
            let p: UpdateChannelParams = params.parse()?;
            state.update_channel(p.channel)
        })
        .unwrap();

    module
        .register_method("list_channels", |params, state| {
            let p: ListChannelsParams = params.parse()?;
            state.list_channels(p.user_id)
        })
        .unwrap();

    module
        .register_method("create_session", |params, state| {
            let p: CreateSessionParams = params.parse()?;
            state.create_session(p.user_id, p.channel_id)
        })
        .unwrap();

    module
        .register_method("get_session", |params, state| {
            let p: GetSessionParams = params.parse()?;
            state.get_session(p.id)
        })
        .unwrap();

    module
        .register_method("get_active_session", |params, state| {
            let p: GetActiveSessionParams = params.parse()?;
            state.get_active_session(p.user_id)
        })
        .unwrap();

    module
        .register_method("list_sessions", |params, state| {
            let p: ListSessionsParams = params.parse()?;
            state.list_sessions(p.user_id)
        })
        .unwrap();

    module
        .register_method("update_session", |params, state| {
            let p: UpdateSessionParams = params.parse()?;
            state.update_session(p.session)
        })
        .unwrap();

    module
        .register_method("create_goal", |params, state| {
            let p: CreateGoalParams = params.parse()?;
            state.create_goal(p.user_id, p.session_id, p.title, p.description)
        })
        .unwrap();

    module
        .register_method("get_goal", |params, state| {
            let p: GetGoalParams = params.parse()?;
            state.get_goal(p.id)
        })
        .unwrap();

    module
        .register_method("update_goal", |params, state| {
            let p: UpdateGoalParams = params.parse()?;
            state.update_goal(p.goal)
        })
        .unwrap();

    module
        .register_method("list_pending_goals", |params, state| {
            let p: ListPendingGoalsParams = params.parse()?;
            state.list_pending_goals(p.session_id)
        })
        .unwrap();

    module
        .register_method("create_tool_execution", |params, state| {
            let p: CreateToolExecutionParams = params.parse()?;
            state.create_tool_execution(
                p.user_id,
                p.session_id,
                p.goal_id,
                p.tool_name,
                p.tool_input,
            )
        })
        .unwrap();

    module
        .register_method("get_tool_execution", |params, state| {
            let p: GetToolExecutionParams = params.parse()?;
            state.get_tool_execution(p.id)
        })
        .unwrap();

    module
        .register_method("find_by_idempotency_key", |params, state| {
            let p: FindByIdempotencyKeyParams = params.parse()?;
            state.find_by_idempotency_key(p.key)
        })
        .unwrap();

    module
        .register_method("update_tool_execution", |params, state| {
            let p: UpdateToolExecutionParams = params.parse()?;
            state.update_tool_execution(p.execution)
        })
        .unwrap();

    module
        .register_method("list_tool_executions", |params, state| {
            let p: ListToolExecutionsParams = params.parse()?;
            state.list_tool_executions(p.goal_id)
        })
        .unwrap();

    module
        .register_method("execute_tool_idempotent", |params, state| {
            let p: ExecuteToolIdempotentParams = params.parse()?;
            let key = p.idempotency_key.unwrap_or_else(|| {
                ToolExecution::generate_idempotency_key(
                    p.user_id,
                    p.session_id,
                    p.goal_id,
                    &p.tool_name,
                    &p.tool_input,
                )
            });
            state.execute_tool_idempotent(
                p.user_id,
                p.session_id,
                p.goal_id,
                p.tool_name,
                p.tool_input,
                key,
            )
        })
        .unwrap();

    module
        .register_method("create_memory", |params, state| {
            let p: CreateMemoryParams = params.parse()?;
            state.create_memory(p.user_id, p.content, p.importance, p.tags)
        })
        .unwrap();

    module
        .register_method("get_memory", |params, state| {
            let p: GetMemoryParams = params.parse()?;
            state.get_memory(p.id)
        })
        .unwrap();

    module
        .register_method("update_memory", |params, state| {
            let p: UpdateMemoryParams = params.parse()?;
            state.update_memory(p.memory)
        })
        .unwrap();

    module
        .register_method("list_memories", |params, state| {
            let p: ListMemoriesParams = params.parse()?;
            state.list_memories(p.user_id, p.limit)
        })
        .unwrap();

    module
}

#[derive(serde::Deserialize)]
struct CreateUserParams {
    username: String,
    display_name: Option<String>,
    email: Option<String>,
}

#[derive(serde::Deserialize)]
struct GetUserParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct CreateChannelParams {
    user_id: Uuid,
    channel_type: ChannelType,
    channel_name: String,
    channel_identifier: String,
}

#[derive(serde::Deserialize)]
struct GetChannelParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct UpdateChannelParams {
    channel: Channel,
}

#[derive(serde::Deserialize)]
struct ListChannelsParams {
    user_id: Uuid,
}

#[derive(serde::Deserialize)]
struct CreateSessionParams {
    user_id: Uuid,
    channel_id: Option<Uuid>,
}

#[derive(serde::Deserialize)]
struct GetSessionParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct GetActiveSessionParams {
    user_id: Uuid,
}

#[derive(serde::Deserialize)]
struct ListSessionsParams {
    user_id: Uuid,
}

#[derive(serde::Deserialize)]
struct UpdateSessionParams {
    session: Session,
}

#[derive(serde::Deserialize)]
struct CreateGoalParams {
    user_id: Uuid,
    session_id: Uuid,
    title: String,
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct GetGoalParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct UpdateGoalParams {
    goal: Goal,
}

#[derive(serde::Deserialize)]
struct ListPendingGoalsParams {
    session_id: Uuid,
}

#[derive(serde::Deserialize)]
struct CreateToolExecutionParams {
    user_id: Uuid,
    session_id: Uuid,
    goal_id: Uuid,
    tool_name: String,
    tool_input: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct GetToolExecutionParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct FindByIdempotencyKeyParams {
    key: String,
}

#[derive(serde::Deserialize)]
struct UpdateToolExecutionParams {
    execution: ToolExecution,
}

#[derive(serde::Deserialize)]
struct ListToolExecutionsParams {
    goal_id: Uuid,
}

#[derive(serde::Deserialize)]
struct CreateMemoryParams {
    user_id: Uuid,
    content: String,
    importance: Option<MemoryImportance>,
    tags: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
struct GetMemoryParams {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct UpdateMemoryParams {
    memory: Memory,
}

#[derive(serde::Deserialize)]
struct ListMemoriesParams {
    user_id: Uuid,
    limit: Option<usize>,
}

#[derive(serde::Deserialize)]
struct ExecuteToolIdempotentParams {
    user_id: Uuid,
    session_id: Uuid,
    goal_id: Uuid,
    tool_name: String,
    tool_input: serde_json::Value,
    #[serde(default)]
    idempotency_key: Option<String>,
}
