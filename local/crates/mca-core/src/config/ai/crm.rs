//! CRM and email configuration.
//!
//! CRM is optional by design: a deployment that only collects and logs requests
//! still starts, with delivery switched to the log sink.

use std::time::Duration;

use crate::config::env::{flag, millis, var};
use crate::config::provider::{ProviderKind, RemoteProviderConfig};
use crate::validation::ValidationError;

/// CRM + email configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct CrmConfig {
    pub remote: RemoteProviderConfig,
    pub email_enabled: bool,
    pub manager_email: String,
    pub company_name: String,
    pub request_timeout: Duration,
}

impl CrmConfig {
    pub fn from_env() -> Result<Self, ValidationError> {
        let manager_email =
            var("MANAGER_EMAIL").unwrap_or_else(|| "logistics@mca-logistics.example".to_string());
        if !manager_email.contains('@') {
            return Err(ValidationError::invalid(
                "MANAGER_EMAIL",
                "must contain '@'",
            ));
        }
        Ok(Self {
            remote: crm_remote()?,
            email_enabled: flag("EMAIL_COMPOSER_ENABLED", true),
            manager_email,
            company_name: var("COMPANY_NAME").unwrap_or_else(|| "MCA Logistics".to_string()),
            request_timeout: millis("CRM_TIMEOUT_MS", 4_000),
        })
    }
}

/// Falls back to the log sink when `CRM_*` is unset, but a bad value that is
/// present is still an error — silently degrading a configured CRM would lose
/// lead requests without anyone noticing.
fn crm_remote() -> Result<RemoteProviderConfig, ValidationError> {
    match RemoteProviderConfig::from_env("CRM") {
        Ok(remote) => Ok(remote),
        Err(_) if var("CRM_PROVIDER").is_none() => Ok(RemoteProviderConfig {
            kind: ProviderKind::Log,
            base_url: None,
            api_key: None,
            model: None,
            timeout: millis("CRM_TIMEOUT_MS", 4_000),
            retry_attempts: 1,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 500,
        }),
        Err(err) => Err(err),
    }
}
