//! CRM adapters: an HTTP push for real deployments and a local logging sink.
//!
//! Both are intentionally non-blocking for the dialogue: a CRM failure is
//! logged and the handover still completes, so a lead is never lost behind a
//! network blip. See `crate::ports::crm` for the contract.

use std::time::Duration;

use crate::config::provider::ProviderKind;
use crate::error::ProviderError;
use crate::ports::crm::{CrmLead, CrmProvider, CrmPushResult};
use crate::provider::http::build_http_client;
use crate::provider::retry::{with_retry, RetryPolicy};

pub use crate::provider::crm_log::LoggingCrmProvider;

pub(crate) const PROVIDER: &str = "crm";

pub struct HttpCrmProvider {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    retry: RetryPolicy,
}

impl HttpCrmProvider {
    pub fn new(
        base_url: impl Into<String>,
        api_key: Option<String>,
        timeout: Duration,
        retry: RetryPolicy,
    ) -> Result<Self, ProviderError> {
        Ok(Self {
            http: build_http_client(ProviderKind::Http, timeout)?,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            timeout,
            retry,
        })
    }

    async fn push(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<CrmPushResult, ProviderError> {
        let url = format!("{}{path}", self.base_url);
        let (response, latency_ms) = with_retry(PROVIDER, &self.retry, self.timeout, |_attempt| {
            let mut builder = self.http.post(&url).json(&body);
            if let Some(key) = &self.api_key {
                builder = builder.header("Authorization", format!("Bearer {key}"));
            }
            async move {
                builder.send().await.map_err(|e| ProviderError::Transport {
                    provider: PROVIDER,
                    message: format!("crm request failed: {e}"),
                })
            }
        })
        .await?;
        if !response.status().is_success() {
            return Err(ProviderError::Status {
                provider: PROVIDER,
                status: response.status().as_u16(),
                message: format!("CRM rejected the payload after {latency_ms}ms"),
            });
        }
        Ok(CrmPushResult {
            reference: uuid::Uuid::new_v4().to_string(),
            accepted: true,
        })
    }
}

#[async_trait::async_trait]
impl CrmProvider for HttpCrmProvider {
    async fn push_lead(&self, lead: CrmLead) -> Result<CrmPushResult, ProviderError> {
        let payload = serde_json::to_value(&lead).map_err(|e| ProviderError::InvalidResponse {
            provider: PROVIDER,
            message: format!("cannot serialize lead: {e}"),
        })?;
        self.push("/leads", payload).await
    }

    async fn deliver_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<CrmPushResult, ProviderError> {
        self.push(
            "/emails",
            serde_json::json!({ "to": to, "subject": subject, "body": body }),
        )
        .await
    }

    fn name(&self) -> &'static str {
        PROVIDER
    }
}
