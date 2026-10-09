use super::env::{flag, millis, number, port, var, var_or};
use super::provider::ProviderKind;
use crate::validation::ValidationError;

/// TCP listener + shared server settings for either service.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_ms: u64,
    pub body_limit_bytes: usize,
    pub enable_docs: bool,
}

impl HttpConfig {
    /// `prefix` is `APP` (voice) or `AI` (ai-service).
    pub fn from_env(prefix: &str) -> Result<Self, ValidationError> {
        let default_port = if prefix == "AI" { 8081 } else { 8080 };
        let port = port(&format!("{prefix}_HTTP_PORT"), default_port);
        if port == 0 {
            return Err(ValidationError::invalid(
                "port",
                "must be a non-zero TCP port",
            ));
        }
        Ok(Self {
            host: var(&format!("{prefix}_HOST")).unwrap_or_else(|| "0.0.0.0".to_string()),
            port,
            request_timeout_ms: millis(&format!("{prefix}_HTTP_TIMEOUT_MS"), 30_000).as_millis()
                as u64,
            body_limit_bytes: number(&format!("{prefix}_MAX_BODY_KB"), 64) as usize * 1024,
            enable_docs: flag("ENABLE_HTTP_DOCS", false),
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// gRPC server/client settings.
#[derive(Debug, Clone, PartialEq)]
pub struct GrpcConfig {
    pub host: String,
    pub port: u16,
    pub url: Option<String>,
    pub timeout_ms: u64,
    pub max_message_bytes: usize,
}

impl GrpcConfig {
    pub fn from_env(prefix: &str) -> Result<Self, ValidationError> {
        let default_port = 50051;
        let port = port(&format!("{prefix}_GRPC_PORT"), default_port);
        Ok(Self {
            host: var(&format!("{prefix}_GRPC_HOST")).unwrap_or_else(|| "0.0.0.0".to_string()),
            port,
            url: var("AI_GRPC_URL"),
            timeout_ms: millis("AI_GRPC_TIMEOUT_MS", 15_000).as_millis() as u64,
            max_message_bytes: number("GRPC_MAX_MESSAGE_KB", 4096) as usize * 1024,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Shared observability + capacity settings.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonConfig {
    pub environment: String,
    pub log_filter: String,
    pub log_json: bool,
    pub metrics_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub shutdown_grace_ms: u64,
}

impl CommonConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        let log_filter = var_or("RUST_LOG", "info,sqlx=warn,tower_http=warn");
        Ok(Self {
            environment: var("APP_ENV").unwrap_or_else(|| "development".to_string()),
            log_json: flag("LOG_JSON", false),
            log_filter,
            metrics_enabled: flag("METRICS_ENABLED", true),
            rate_limit_per_minute: number("RATE_LIMIT_PER_MINUTE", 600),
            shutdown_grace_ms: millis("SHUTDOWN_GRACE_MS", 10_000).as_millis() as u64,
        })
    }
}

/// Whether a provider should be built in live (HTTP) or fake mode.
pub fn wants_live(kind: ProviderKind) -> bool {
    kind == ProviderKind::Http
}

#[cfg(test)]
mod tests;
