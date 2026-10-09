//! OpenAI-compatible LLM adapters.
//!
//! Two implementations sit behind the same trait: `HttpLlmProvider` (live) and
//! `RuleBasedLlmProvider` (deterministic, doubles as the outage fallback and as
//! the fake in tests).

mod client;
mod config;
mod http_llm;
mod prompts;
mod response_prompt;
mod rule_llm;
mod types;

pub use client::HttpChatClient;
pub use config::ChatClientConfig;
pub use http_llm::HttpLlmProvider;
pub use rule_llm::RuleBasedLlmProvider;
pub use types::{
    ApiError, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, Choice, ResponseFormat,
    Usage,
};
