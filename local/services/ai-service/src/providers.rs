//! Provider instantiation from environment configuration.
//!
//! The LLM is always an `HttpLlmProvider` (live model) or the deterministic
//! `RuleBasedLlmProvider`; even the live one carries the rule fallback, so an
//! outage degrades phrasing quality, never the collected data set.

use std::sync::Arc;

use anyhow::{anyhow, Context};
use mca_core::config::ai::{CrmConfig, LlmConfig};
use mca_core::config::provider::ProviderKind;
use mca_core::ports::crm::CrmProvider;
use mca_core::ports::llm::LlmProvider;
use mca_core::provider::crm::{HttpCrmProvider, LoggingCrmProvider};
use mca_core::provider::http::build_http_client;
use mca_core::provider::openai::{
    ChatClientConfig, HttpChatClient, HttpLlmProvider, RuleBasedLlmProvider,
};

/// Everything the engine holds onto: each entry is already an `Arc<dyn _>`.
pub struct Providers {
    pub llm: Arc<dyn LlmProvider>,
    pub crm: Arc<dyn CrmProvider>,
}

pub fn build(llm_cfg: &LlmConfig, crm_cfg: &CrmConfig) -> Result<Providers, anyhow::Error> {
    let fallback = Arc::new(RuleBasedLlmProvider::new());
    let llm: Arc<dyn LlmProvider> = match llm_cfg.remote.kind {
        ProviderKind::Http => {
            let base_url = llm_cfg
                .remote
                .base_url
                .clone()
                .ok_or_else(|| anyhow!("LLM_BASE_URL is required for LLM_PROVIDER=http"))?;
            let model = llm_cfg
                .remote
                .model
                .clone()
                .unwrap_or_else(|| "gpt-4o-mini".into());
            let mut config = ChatClientConfig::new(base_url, model)
                .with_api_key(llm_cfg.remote.api_key.clone())
                .with_timeout(llm_cfg.remote.timeout)
                .with_retry(llm_cfg.remote.retry_policy());
            config.temperature = llm_cfg.temperature;
            config.max_tokens = llm_cfg.max_tokens;
            let http = build_http_client(ProviderKind::Http, llm_cfg.remote.timeout)
                .with_context(|| "cannot build LLM http client")?;
            let client = HttpChatClient::new(http, config);
            Arc::new(HttpLlmProvider::new(client, fallback))
        }
        _ => fallback as Arc<dyn LlmProvider>,
    };

    let crm: Arc<dyn CrmProvider> = match crm_cfg.remote.kind {
        ProviderKind::Http => {
            let base_url = crm_cfg
                .remote
                .base_url
                .clone()
                .ok_or_else(|| anyhow!("CRM_BASE_URL is required for CRM_PROVIDER=http"))?;
            Arc::new(
                HttpCrmProvider::new(
                    base_url,
                    crm_cfg.remote.api_key.clone(),
                    crm_cfg.request_timeout,
                    crm_cfg.remote.retry_policy(),
                )
                .with_context(|| "cannot build CRM http client")?,
            )
        }
        _ => Arc::new(LoggingCrmProvider),
    };

    Ok(Providers { llm, crm })
}
