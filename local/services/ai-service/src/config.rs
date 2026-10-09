//! Assembled environment configuration for the ai-service.
//!
//! Every parser reuses `mca-core`'s `config` module so defaults, validation and
//! `.env.example` stay a single source of truth across both services.

use std::time::Duration;

use mca_core::config::ai::{CrmConfig, LlmConfig};
use mca_core::config::env;
use mca_core::config::server::{CommonConfig, GrpcConfig, HttpConfig};
use mca_core::validation::ValidationError;

/// Postgres connection settings for the ai-service's own pool.
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        let url = env::var("DATABASE_URL")
            .ok_or_else(|| ValidationError::invalid("DATABASE_URL", "is required"))?;
        Ok(Self {
            url,
            max_connections: env::number("DATABASE_MAX_CONNECTIONS", 10),
            acquire_timeout: env::millis("DATABASE_ACQUIRE_TIMEOUT_MS", 5_000),
        })
    }
}

/// Everything the process needs to boot. Assembled once in `main`.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub common: CommonConfig,
    pub http: HttpConfig,
    pub grpc: GrpcConfig,
    pub llm: LlmConfig,
    pub crm: CrmConfig,
    pub db: DbConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ValidationError> {
        Ok(Self {
            common: CommonConfig::from_env()?,
            http: HttpConfig::from_env("AI")?,
            grpc: GrpcConfig::from_env("AI")?,
            llm: LlmConfig::from_env()?,
            crm: CrmConfig::from_env()?,
            db: DbConfig::from_env()?,
        })
    }
}
