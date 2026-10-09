//! Typed configuration, parsed once at start-up.
//!
//! Nothing reads `std::env` outside this module: a misconfigured deployment
//! fails immediately with a named field instead of at the first phone call.

pub mod ai;
pub mod env;
pub mod provider;
pub mod server;

pub use ai::{CrmConfig, LlmConfig, SttConfig, TtsConfig};
pub use env::{flag, millis, number, port, seconds, var, var_or};
pub use provider::{ProviderKind, RemoteProviderConfig};
pub use server::{CommonConfig, GrpcConfig, HttpConfig};
