//! The voice engine: one place a call lifecycle and its audio pipeline are
//! orchestrated. Pure against its ports — STT/TTS/telephony and the ai gateway
//! are all traits, so the same engine runs against HTTP vendors, the
//! deterministic `mca-testing` fakes, or the live ai-service.

pub mod end;
pub mod open;
mod take;
pub mod turn;
pub mod types;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use mca_core::domain::language::Language;
use mca_core::ports::stt::SttProvider;
use mca_core::ports::telephony::TelephonyProvider;
use mca_core::ports::tts::TtsProvider;

use crate::ai::AiGateway;
use crate::error::VoiceError;

pub use types::{
    now_ms, CallRecord, EventOut, OpenOutcome, CALL_FINISHED, CALL_IN_PROGRESS, MAX_AUDIO_BYTES,
};

pub struct VoiceEngine {
    pub(crate) calls: RwLock<HashMap<String, CallRecord>>,
    pub(crate) stt: Arc<dyn SttProvider>,
    pub(crate) tts: Arc<dyn TtsProvider>,
    pub(crate) telephony: Arc<dyn TelephonyProvider>,
    pub(crate) ai: Arc<dyn AiGateway>,
    pub(crate) default_language: String,
}

impl VoiceEngine {
    pub fn new(
        stt: Arc<dyn SttProvider>,
        tts: Arc<dyn TtsProvider>,
        telephony: Arc<dyn TelephonyProvider>,
        ai: Arc<dyn AiGateway>,
        default_language: impl Into<String>,
    ) -> Self {
        Self {
            calls: RwLock::new(HashMap::new()),
            stt,
            tts,
            telephony,
            ai,
            default_language: default_language.into(),
        }
    }

    /// Liveness check used by `/ready`.
    pub async fn ping(&self) -> Result<(), VoiceError> {
        Ok(())
    }

    /// Resolves the request language; an empty value falls back to the
    /// deployment's default (`STT_LANGUAGE`).
    fn resolve_language(&self, raw: &str) -> Result<Language, String> {
        let effective = if raw.trim().is_empty() {
            self.default_language.as_str()
        } else {
            raw
        };
        crate::map::parse_language(effective)
    }
}
