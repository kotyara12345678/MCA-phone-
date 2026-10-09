//! voice-service: the realtime voice layer.
//!
//! Owns the call lifecycle and the audio pipeline; every external dependency
//! (speech providers, telephony, the ai-service) sits behind a trait, so the
//! same binary runs against HTTP vendors, the deterministic `mca-testing`
//! fakes, or the in-process ai-service during integration tests.
//!
//! ```text
//! engine  →  call lifecycle + audio pipeline, pure against ports
//! ai      →  gRPC gateway to ai-service (the dialogue brain)
//! grpc    →  tonic `VoiceService` + health server
//! http    →  axum liveness / readiness endpoints
//! ```

pub mod ai;
pub mod config;
pub mod engine;
pub mod error;
pub mod grpc;
pub mod http;
pub mod map;
pub mod providers;

pub use ai::{AiGateway, GrpcAiGateway};
pub use config::AppConfig;
pub use engine::VoiceEngine;
