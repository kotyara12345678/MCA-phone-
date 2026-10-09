//! Deterministic fakes and harness helpers for tests.
//!
//! Every external dependency in `mca-core`'s `ports` has a stand-in here that
//! is scriptable, observable and offline-safe, so the whole platform (unit and
//! E2E) runs with no API keys, no telephony hardware and no CRM. Each fake
//! records what was asked of it so tests can assert on behaviour, not on more
//! mocks.

pub mod audio;
pub mod crm;
pub mod db;
pub mod llm;
pub mod stt;
pub mod telephony;
pub mod tts;

pub use audio::{frame_for, TRANSCRIPT_SILENCE_THRESHOLD};
pub use crm::RecordingCrmProvider;
pub use db::{configured as test_database_configured, TestDb};
pub use llm::FakeLlmProvider;
pub use stt::FakeSttProvider;
pub use telephony::FakeTelephonyProvider;
pub use tts::FakeTtsProvider;
