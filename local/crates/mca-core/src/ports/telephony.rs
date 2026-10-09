use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

/// An inbound call as seen by the telephony layer. Deliberately vendor-neutral:
/// an Asterisk/ARI adapter maps `ChannelCreate` onto this, and the mock
/// implements the same shape locally.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboundCall {
    /// Identifier assigned by the telephony system (ARI `uniqueid` in future).
    pub external_call_id: String,
    pub from_number: Option<String>,
    pub to_number: Option<String>,
    pub language: Option<String>,
}

/// A framed audio packet delivered by the telephony system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboundMedia {
    pub external_call_id: String,
    pub seq: u64,
    pub audio: Vec<u8>,
    pub format: crate::ports::stt::AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
    pub end_of_utterance: bool,
}

/// Media streamed back to the caller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboundMedia {
    pub external_call_id: String,
    pub seq: u64,
    pub audio: Vec<u8>,
    pub format: crate::ports::stt::AudioFormat,
}

/// Handle to an established call, used to stream media and to hang up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallHandle {
    pub external_call_id: String,
}

/// Telephony port. The MVP ships `MockTelephonyProvider`; `AsteriskProvider`
/// will implement this same interface later without touching the voice pipeline.
#[async_trait]
pub trait TelephonyProvider: Send + Sync {
    /// Registers a new inbound call. Returns the handle used for media.
    async fn answer(&self, call: InboundCall) -> Result<CallHandle, ProviderError>;

    /// Streams one media frame to the caller.
    async fn send_media(
        &self,
        handle: &CallHandle,
        media: OutboundMedia,
    ) -> Result<(), ProviderError>;

    /// Terminates the call.
    async fn hangup(&self, handle: &CallHandle, reason: &str) -> Result<(), ProviderError>;

    fn name(&self) -> &'static str;

    /// False for the local mock; Asterisk would return true.
    fn is_live(&self) -> bool {
        false
    }
}
