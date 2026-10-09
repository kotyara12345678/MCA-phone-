//! Call teardown and read paths: hang up the call + finish the ai-service
//! session, then the get/list queries for the control plane.

use mca_core::ports::telephony::CallHandle;

use super::types::now_ms;
use super::{CallRecord, VoiceEngine, CALL_FINISHED, CALL_IN_PROGRESS};
use crate::ai::FinishOutcome;
use crate::error::VoiceError;

impl VoiceEngine {
    pub async fn hangup(
        &self,
        call_id: &str,
        reason: &str,
    ) -> Result<(CallRecord, FinishOutcome), VoiceError> {
        let (session_id, external_call_id) = {
            let mut calls = self.calls.write().expect("call registry lock poisoned");
            let record = calls
                .get_mut(call_id)
                .ok_or_else(|| VoiceError::UnknownCall(call_id.to_string()))?;
            if record.status != CALL_IN_PROGRESS {
                return Err(VoiceError::NotOpen(
                    call_id.to_string(),
                    format!("status={}", record.status),
                ));
            }
            (record.session_id.clone(), record.external_call_id.clone())
        };

        let outcome = self.ai.finish(&session_id, reason).await?;
        self.telephony
            .hangup(&CallHandle { external_call_id }, reason)
            .await
            .map_err(VoiceError::from_telephony)?;

        let mut calls = self.calls.write().expect("call registry lock poisoned");
        let record = calls
            .get_mut(call_id)
            .ok_or_else(|| VoiceError::UnknownCall(call_id.to_string()))?;
        record.status = CALL_FINISHED;
        record.ended_at_unix_ms = now_ms();
        record.speaking = false;
        Ok((record.clone(), outcome))
    }

    pub fn get(&self, call_id: &str) -> Result<CallRecord, VoiceError> {
        self.calls
            .read()
            .expect("call registry lock poisoned")
            .get(call_id)
            .cloned()
            .ok_or_else(|| VoiceError::UnknownCall(call_id.to_string()))
    }

    /// All calls, oldest first.
    pub fn list(&self) -> Vec<CallRecord> {
        let mut records: Vec<CallRecord> = self
            .calls
            .read()
            .expect("call registry lock poisoned")
            .values()
            .cloned()
            .collect();
        records.sort_by_key(|record| record.started_at_unix_ms);
        records
    }
}
