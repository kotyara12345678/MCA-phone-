pub mod broker;
pub mod crm;
pub mod llm;
pub mod stt;
pub mod telephony;
pub mod tts;

pub use broker::{DomainEvent, EventBroker};
pub use crm::{CrmLead, CrmProvider, CrmPushResult};
pub use llm::{ExtractionRequest, ExtractionResult, LlmProvider, ResponseDraft, ResponseRequest};
pub use stt::{AudioFormat, AudioFrame, SttProvider, Transcript};
pub use telephony::{CallHandle, InboundCall, InboundMedia, OutboundMedia, TelephonyProvider};
pub use tts::{Synthesis, TtsProvider};
