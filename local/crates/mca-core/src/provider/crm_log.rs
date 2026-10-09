//! Local CRM sink: logs leads and manager emails without any network I/O.
//!
//! The default when `CRM_PROVIDER=log` (or unset), so a deployment that only
//! collects requests still starts and never drops meaningful data — and local
//! E2E tests can assert the handover happened by reading the trace.

use crate::ports::crm::{CrmLead, CrmProvider, CrmPushResult};

/// Accepts everything and records it to the trace.
pub struct LoggingCrmProvider;

#[async_trait::async_trait]
impl CrmProvider for LoggingCrmProvider {
    async fn push_lead(&self, lead: CrmLead) -> Result<CrmPushResult, crate::error::ProviderError> {
        tracing::info!(
            application_id = %lead.application_id,
            qualification = %lead.qualification.as_str(),
            "crm lead captured by logging sink"
        );
        Ok(CrmPushResult {
            reference: uuid::Uuid::new_v4().to_string(),
            accepted: true,
        })
    }

    async fn deliver_email(
        &self,
        to: &str,
        subject: &str,
        _body: &str,
    ) -> Result<CrmPushResult, crate::error::ProviderError> {
        tracing::info!(to, subject, "manager email captured by logging sink");
        Ok(CrmPushResult {
            reference: uuid::Uuid::new_v4().to_string(),
            accepted: true,
        })
    }

    fn name(&self) -> &'static str {
        "log"
    }
}
