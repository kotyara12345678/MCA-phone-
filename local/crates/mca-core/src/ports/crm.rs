use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::application::ApplicationState;
use crate::domain::qualification::Qualification;
use crate::error::ProviderError;
use crate::ids::ApplicationId;

/// What a CRM push is expected to achieve. Kept intentionally small: the MVP
/// has no real CRM, and a wide surface here would only add untested code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrmLead {
    pub application_id: ApplicationId,
    pub summary: String,
    pub state: ApplicationState,
    pub qualification: Qualification,
    pub manager_email_subject: String,
    pub transcript_excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrmPushResult {
    pub reference: String,
    pub accepted: bool,
}

/// CRM port. `HttpCrmProvider` posts to a configured endpoint;
/// `LoggingCrmProvider` records the payload locally. Both are non-blocking for
/// the dialogue: a CRM failure is logged and the handover still completes.
#[async_trait]
pub trait CrmProvider: Send + Sync {
    async fn push_lead(&self, lead: CrmLead) -> Result<CrmPushResult, ProviderError>;

    /// Second-stage delivery: the composed manager email.
    async fn deliver_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<CrmPushResult, ProviderError> {
        let _ = (to, subject, body);
        Ok(CrmPushResult {
            reference: "noop".to_string(),
            accepted: false,
        })
    }

    fn name(&self) -> &'static str;
}
