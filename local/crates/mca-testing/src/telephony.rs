//! Scripted telephony fake: records answered calls, streamed media and hang-ups,
//! with one-shot fault scripts for the retry / graceful-degrade paths.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mca_core::error::ProviderError;
use mca_core::ports::telephony::{CallHandle, InboundCall, OutboundMedia, TelephonyProvider};

#[derive(Default)]
struct State {
    answered: Vec<InboundCall>,
    media: HashMap<String, Vec<OutboundMedia>>,
    hangups: Vec<(String, String)>,
    fail_answer: Option<ProviderError>,
    fail_media: Option<ProviderError>,
}

/// Deterministic stand-in for the telephony port.
#[derive(Clone, Default)]
pub struct FakeTelephonyProvider {
    state: Arc<Mutex<State>>,
}

impl FakeTelephonyProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_answer(&self, err: ProviderError) -> &Self {
        self.state.lock().expect("telephony lock").fail_answer = Some(err);
        self
    }

    pub fn fail_media(&self, err: ProviderError) -> &Self {
        self.state.lock().expect("telephony lock").fail_media = Some(err);
        self
    }

    pub fn answered_calls(&self) -> Vec<InboundCall> {
        self.state.lock().expect("telephony lock").answered.clone()
    }

    pub fn media_for(&self, external_call_id: &str) -> Vec<OutboundMedia> {
        self.state
            .lock()
            .expect("telephony lock")
            .media
            .get(external_call_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn hangups(&self) -> Vec<(String, String)> {
        self.state.lock().expect("telephony lock").hangups.clone()
    }
}

#[async_trait]
impl TelephonyProvider for FakeTelephonyProvider {
    async fn answer(&self, call: InboundCall) -> Result<CallHandle, ProviderError> {
        let mut state = self.state.lock().expect("telephony lock");
        if let Some(err) = state.fail_answer.take() {
            return Err(err);
        }
        state.answered.push(call.clone());
        Ok(CallHandle {
            external_call_id: call.external_call_id,
        })
    }

    async fn send_media(
        &self,
        handle: &CallHandle,
        media: OutboundMedia,
    ) -> Result<(), ProviderError> {
        let mut state = self.state.lock().expect("telephony lock");
        if let Some(err) = state.fail_media.take() {
            return Err(err);
        }
        state
            .media
            .entry(handle.external_call_id.clone())
            .or_default()
            .push(media);
        Ok(())
    }
    async fn hangup(&self, handle: &CallHandle, reason: &str) -> Result<(), ProviderError> {
        let mut state = self.state.lock().expect("telephony lock");
        state
            .hangups
            .push((handle.external_call_id.clone(), reason.to_string()));
        Ok(())
    }

    fn name(&self) -> &'static str {
        "fake_telephony"
    }
}
