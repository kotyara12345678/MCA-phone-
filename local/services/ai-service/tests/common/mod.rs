//! Shared fixtures for the ai-service integration suite.
//!
//! One engine-with-fakes wiring is reused by every test file, so the raw
//! `DialogueEngine` construction (and `EmailSettings`) lives here. The gRPC
//! server bootstrap lives in `common_grpc`, which only the transport tests
//! pull in.

use std::sync::Arc;

use ai_service::engine::{DialogueEngine, EmailSettings, EventBus};
use ai_service::store::mem::MemStore;
use mca_core::metrics::NoopMetrics;
use mca_core::ports::crm::CrmProvider;
use mca_core::ports::llm::LlmProvider;
use mca_core::provider::crm::LoggingCrmProvider;
use mca_core::provider::openai::RuleBasedLlmProvider;

pub fn settings() -> EmailSettings {
    EmailSettings {
        enabled: true,
        manager_email: "logistics@mca-logistics.example".into(),
        company_name: "MCA Logistics".into(),
        transcript_line_limit: 20,
    }
}

pub fn fixture() -> (Arc<DialogueEngine>, Arc<MemStore>) {
    let store = Arc::new(MemStore::new());
    let llm: Arc<dyn LlmProvider> = Arc::new(RuleBasedLlmProvider::new());
    let crm: Arc<dyn CrmProvider> = Arc::new(LoggingCrmProvider);
    let engine = Arc::new(DialogueEngine::new(
        store.clone(),
        store.clone(),
        llm,
        crm,
        Arc::new(NoopMetrics),
        EventBus::new(32),
        settings(),
        8,
        2_000,
    ));
    (engine, store)
}
