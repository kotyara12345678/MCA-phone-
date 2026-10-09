//! ai-service: dialogue state, LLM extraction, qualification and persistence.
//!
//! Layering mirrors the runtime split (`main.rs` is the only binary; everything
//! else lives here so integration tests can drive the real engine):
//!
//! ```text
//! store   →  persistence boundary (Postgres in prod, in-memory in tests)
//! engine  →  start/turn/finish orchestration, pure against ports
//! map     →  domain ↔ generated proto conversions
//! grpc    →  tonic `AiService` + health server
//! http    →  axum health / readiness / metrics endpoints
//! ```

pub mod config;
pub mod db;
pub mod engine;
pub mod grpc;
pub mod http;
pub mod map;
pub mod metrics;
pub mod providers;
pub mod store;

pub use engine::{DialogueEngine, EventBus, SessionEvent};
pub use map::MapError;
pub use store::{ApplicationStore, SessionStore};
