//! Shared domain kernel for the MCA Logistics voice AI platform.
//!
//! Layering inside this crate mirrors the runtime architecture:
//!
//! ```text
//! domain  →  pure types, dialogue rules, deterministic qualification
//! nlu     →  deterministic offline extraction (fallback engine)
//! policy  →  directives + company-approved wording (no model output)
//! ports   →  traits for every external dependency
//! provider→  HTTP adapters + rule-based implementations
//! ```
//!
//! Both microservices depend on this crate, so a change to a business rule is a
//! single edit rather than two divergent copies.

pub mod config;
pub mod domain;
pub mod email;
pub mod error;
pub mod ids;
pub mod metrics;
pub mod nlu;
pub mod policy;
pub mod ports;
pub mod provider;
pub mod validation;

pub use domain::application::ApplicationState;
pub use domain::dialogue::DialogueStage;
pub use domain::intent::Intent;
pub use domain::language::Language;
pub use domain::qualification::Qualification;
pub use domain::session::Session;
pub use error::{AppError, ProviderError, ProviderErrorKind, StorageError};
pub use ids::{ApplicationId, CallId, CorrelationId, EmailId, MessageId, SessionId};
pub use validation::ValidationError;
