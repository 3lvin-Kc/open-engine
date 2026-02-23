//! OpenClaw State Engine Binary
//!
//! This binary runs the JSON-RPC server for the state engine.

use std::net::SocketAddr;

use clap::{Parser, Subcommand, ValueHint};
use jsonrpsee::server::ServerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use openclaw_state_engine::{api::create_rpc_module, persistence::Database, DatabaseConfig};

#[derive(Parser, Debug)]
#[command(name = "openclaw-state-engine")]
#[command(about = "OpenClaw Persistent State & Session Management Engine")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the JSON-RPC server
    Serve {
        #[arg(short, long, default_value = "127.0.0.1:3030", help = "Server address")]
        addr: SocketAddr,

        #[arg(short, long, help = "Database file path", value_hint = ValueHint::FilePath)]
        database: Option<std::path::PathBuf>,
    },
    
    /// Replay a session and show all actions
    Replay {
        #[arg(short, long, help = "Database file path", value_hint = ValueHint::FilePath)]
        database: std::path::PathBuf,
        
        #[arg(short, long, help = "Session ID to replay")]
        session: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "openclaw_state_engine=info,tokio=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Serve { addr, database } => {
            let db_path = database.unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .join("openclaw_state.db")
            });

            let config = DatabaseConfig {
                path: db_path,
                ..Default::default()
            };

            let db = Database::open(config)?;

            let module = create_rpc_module(db);

            let server = ServerBuilder::new()
                .build(addr)
                .await?;

            let _handle = server.start(module);

            tracing::info!("State engine server listening on {}", addr);

            std::thread::park();
        }
        Commands::Replay { database, session } => {
            replay_session(database, session)?;
        }
    }

    Ok(())
}

fn replay_session(db_path: std::path::PathBuf, session_id: String) -> anyhow::Result<()> {
    use uuid::Uuid;
    use openclaw_state_engine::persistence::{SessionRepository, GoalRepository, ToolExecutionRepository};
    
    let config = DatabaseConfig {
        path: db_path,
        ..Default::default()
    };
    
    let db = Database::open(config)?;
    
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;
    
    let session_repo = SessionRepository::new(db.clone());
    let goal_repo = GoalRepository::new(db.clone());
    let tool_repo = ToolExecutionRepository::new(db.clone());
    
    let session = session_repo.get(session_uuid)?;
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    SESSION REPLAY                              ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Session ID:     {}", session.base.id);
    println!("User ID:        {}", session.user_id);
    println!("Status:         {:?}", session.status);
    println!("Created:        {}", session.base.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Last Activity:  {}", session.last_activity_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!();
    
    let goals = goal_repo.list_for_session(session_uuid)?;
    
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ GOALS ({})                                                        │", goals.len());
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    for (i, goal) in goals.iter().enumerate() {
        println!();
        println!("  Goal #{}: {}", i + 1, goal.title);
        println!("  ID:          {}", goal.base.id);
        println!("  Status:      {:?}", goal.status);
        println!("  Priority:    {:?}", goal.priority);
        if let Some(ref desc) = goal.description {
            if !desc.is_empty() {
                println!("  Description: {}", desc);
            }
        }
        if let Some(ref started) = goal.started_at {
            println!("  Started:     {}", started.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        if let Some(ref completed) = goal.completed_at {
            println!("  Completed:   {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));
        }
        
        let executions = tool_repo.list_for_goal(goal.base.id)?;
        
        if !executions.is_empty() {
            println!();
            println!("  ┌── TOOL EXECUTIONS ({}) ──", executions.len());
            
            for exec in &executions {
                println!("  │");
                println!("  │  Tool:      {}", exec.tool_name);
                println!("  │  Status:    {:?}", exec.status);
                println!("  │  Input:     {}", serde_json::to_string(&exec.tool_input).unwrap_or_else(|_| "N/A".to_string()));
                
                if let Some(ref output) = exec.output {
                    let output_str = serde_json::to_string(output).unwrap_or_else(|_| "N/A".to_string());
                    if output_str.len() > 60 {
                        println!("  │  Output:    {}...", &output_str[..60]);
                    } else {
                        println!("  │  Output:    {}", output_str);
                    }
                }
                
                if let Some(ref error) = exec.error {
                    println!("  │  Error:     {}", error);
                }
                
                if let Some(ref key) = exec.idempotency_key {
                    println!("  │  Idempotency Key: {}", key);
                }
                
                let created = exec.base.created_at.format("%H:%M:%S");
                println!("  │  Time:      {}", created);
            }
            println!("  └───────────────────────────");
        }
    }
    
    println!();
    println!("══════════════════════════════════════════════════════════════════");
    println!("Replay complete. {} goal(s), {} execution(s) total.", 
        goals.len(),
        goals.iter().map(|g| tool_repo.list_for_goal(g.base.id).map(|e| e.len()).unwrap_or(0)).sum::<usize>()
    );
    
    Ok(())
}
