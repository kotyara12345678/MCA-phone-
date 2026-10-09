//! Recording CRM fake: accepts every push and email, records the exact payloads,
//! and honours one scripted failure to exercise the degrade-and-continue path.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mca_core::error::ProviderError;
use mca_core::ports::crm::{CrmLead, CrmProvider, CrmPushResult};

#[derive(Default)]
struct State {
    leads: Vec<CrmLead>,
    emails: Vec<(String, String, String)>,
    fail_next_push: Option<ProviderError>,
    counter: u64,
}

/// Deterministic stand-in for the CRM port.
#[derive(Clone, Default)]
pub struct RecordingCrmProvider {
    state: Arc<Mutex<State>>,
}

impl RecordingCrmProvider {
    pub fn new() -> Self {
        Self::default()
    }

    /// The next lead push fails once; subsequent pushes succeed.
    pub fn fail_next_push(&self, err: ProviderError) -> &Self {
        self.state.lock().expect("crm lock").fail_next_push = Some(err);
        self
    }

    pub fn leads(&self) -> Vec<CrmLead> {
        self.state.lock().expect("crm lock").leads.clone()
    }

    pub fn emails(&self) -> Vec<(String, String, String)> {
        self.state.lock().expect("crm lock").emails.clone()
    }

    pub fn push_count(&self) -> usize {
        self.state.lock().expect("crm lock").leads.len()
    }
}

#[async_trait]
impl CrmProvider for RecordingCrmProvider {
    async fn push_lead(&self, lead: CrmLead) -> Result<CrmPushResult, ProviderError> {
        let mut state = self.state.lock().expect("crm lock");
        if let Some(err) = state.fail_next_push.take() {
            return Err(err);
        }
        state.counter += 1;
        let reference = format!("fake-crm-{:03}", state.counter);
        state.leads.push(lead);
        Ok(CrmPushResult {
            reference,
            accepted: true,
        })
    }

    async fn deliver_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<CrmPushResult, ProviderError> {
        let mut state = self.state.lock().expect("crm lock");
        state.counter += 1;
        let reference = format!("fake-crm-{:03}", state.counter);
        state
            .emails
            .push((to.to_string(), subject.to_string(), body.to_string()));
        Ok(CrmPushResult {
            reference,
            accepted: true,
        })
    }

    fn name(&self) -> &'static str {
        "recording_crm"
    }
}
