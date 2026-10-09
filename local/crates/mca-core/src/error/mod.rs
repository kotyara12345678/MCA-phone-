//! Every fallible operation funnels into these enums so that transport layers
//! can map errors without inspecting string messages. `mca-core` deliberately
//! has no database dependency: `StorageError` is produced by the service that
//! owns the pool (see `services/ai-service/src/repository`).

pub mod provider_error;

pub use provider_error::{ProviderError, ProviderErrorKind};

/// Domain / application level failures.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("call not found: {0}")]
    CallNotFound(String),
    #[error("application not found: {0}")]
    ApplicationNotFound(String),
    #[error("session is not active: {0}")]
    SessionNotActive(String),
    #[error("session already finished: {0}")]
    SessionAlreadyFinished(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),
    #[error("rate limit exceeded: {0}")]
    RateLimited(String),
    #[error("capacity exhausted: {0}")]
    CapacityExhausted(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Validation(#[from] crate::validation::ValidationError),
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

impl AppError {
    /// Stable machine-readable code, used by HTTP error bodies and metrics.
    pub fn code(&self) -> &'static str {
        match self {
            Self::SessionNotFound(_) | Self::CallNotFound(_) | Self::NotFound(_) => "not_found",
            Self::ApplicationNotFound(_) => "application_not_found",
            Self::SessionNotActive(_) | Self::SessionAlreadyFinished(_) => "session_state",
            Self::InvalidRequest(_) | Self::Validation(_) => "invalid_request",
            Self::PayloadTooLarge(_) => "payload_too_large",
            Self::RateLimited(_) => "rate_limited",
            Self::CapacityExhausted(_) => "capacity_exhausted",
            Self::Conflict(_) => "conflict",
            Self::Provider(_) => "provider_error",
            Self::Storage(_) => "storage_error",
        }
    }
}

/// Persistence failures, kept separate so `/ready` can degrade precisely.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database query failed: {0}")]
    Query(String),
    #[error("database pool exhausted: {0}")]
    PoolExhausted(String),
    #[error("migration failed: {0}")]
    Migration(String),
    #[error("row not found in `{table}`")]
    RowNotFound { table: &'static str },
    #[error("serialization failed: {0}")]
    Serde(String),
}
