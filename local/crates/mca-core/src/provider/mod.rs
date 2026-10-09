//! Outbound AI provider adapters.
//!
//! Each adapter is a thin, typed HTTP client for one OpenAI-compatible shape.
//! Business logic never calls them directly — it goes through the traits in
//! `crate::ports`, which is what makes the fakes in `mca-testing` droppable.

pub mod crm;
pub mod crm_log;
pub mod http;
pub mod http_error;
pub mod openai;
pub mod retry;
pub mod stt;
pub mod tts;
