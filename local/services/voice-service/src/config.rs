//! Assembled environment configuration for the voice-service.
//!
//! Every parser reuses `mca-core`'s `config` module so defaults, validation and
//! `.env.example` stay a single source of truth across both services.

use mca_core::config::ai::{SttConfig, TtsConfig};
use mca_core::config::env;
use mca_core::config::server::{CommonConfig, GrpcConfig, HttpConfig};
use mca_core::validation::ValidationError;

/// Connection settings for the ai-service this voice layer drives.
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub url: String,
    pub timeout_ms: u64,
}

impl AiConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        Ok(Self {
            url: env::var_or("AI_GRPC_URL", "http://127.0.0.1:50051"),
            timeout_ms: env::millis("AI_GRPC_TIMEOUT_MS", 15_000).as_millis() as u64,
        })
    }
}

/// Everything the process needs to boot. Assembled once in `main`.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub common: CommonConfig,
    pub http: HttpConfig,
    pub grpc: GrpcConfig,
    pub stt: SttConfig,
    pub tts: TtsConfig,
    pub ai: AiConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self, ValidationError> {
        Ok(Self {
            common: CommonConfig::from_env()?,
            http: HttpConfig::from_env("APP")?,
            grpc: GrpcConfig::from_env("VOICE")?,
            stt: SttConfig::from_env()?,
            tts: TtsConfig::from_env()?,
            ai: AiConfig::from_env()?,
        })
    }
}
