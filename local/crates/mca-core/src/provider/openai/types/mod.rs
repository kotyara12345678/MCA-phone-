//! Typed models for any OpenAI-compatible chat endpoint.
//!
//! The request and response halves are separate modules: the request side is
//! what we send, the response side is what we must tolerate, and keeping them
//! apart makes it obvious which fields we control and which ones providers
//! improvise on.

mod request;
mod response;

pub use request::{ChatCompletionRequest, ChatMessage, ResponseFormat};
pub use response::{ApiError, ChatCompletionResponse, Choice, Usage};
