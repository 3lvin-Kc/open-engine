//! OpenClaw Persistent State & Session Management Engine
//!
//! This crate provides a durable, concurrency-safe state storage layer for OpenClaw agents.
//! It exposes a JSON-RPC API for interaction with the TypeScript stack.

pub mod api;
pub mod concurrency;
pub mod models;
pub mod persistence;

pub use api::*;
pub use concurrency::*;
pub use models::*;
pub use persistence::*;

/// Re-export error types for convenience
pub use anyhow::Result;
pub use thiserror::Error;
