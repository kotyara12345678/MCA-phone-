//! Scripted LLM fake: deterministic by default (offline rules), scriptable per
//! call, with one-shot fault modes to exercise retries and the fallback path.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;
use mca_core::error::ProviderError;
use mca_core::nlu;
use mca_core::policy::compose::compose;
use mca_core::ports::llm::{
    ExtractionRequest, ExtractionResult, LlmProvider, ResponseDraft, ResponseRequest,
};

type Ex = Result<ExtractionResult, ProviderError>;
type Rsp = Result<String, ProviderError>;

#[derive(Default)]
struct State {
    extract_queue: VecDeque<Ex>,
    respond_queue: VecDeque<Rsp>,
    latency: Duration,
}

/// Deterministic stand-in for the LLM port.
#[derive(Clone, Default)]
pub struct FakeLlmProvider {
    state: Arc<Mutex<State>>,
}
impl FakeLlmProvider {
    pub fn with_latency(self, latency: Duration) -> Self {
        self.state.lock().expect("llm lock").latency = latency;
        self
    }
    pub fn expect_extract(&self, result: ExtractionResult) -> &Self {
        self.push(Some(Ok(result)), None)
    }

    pub fn expect_respond(&self, text: impl Into<String>) -> &Self {
        self.push(None, Some(Ok(text.into())))
    }

    pub fn fail_next(&self, err: ProviderError) -> &Self {
        self.push(Some(Err(err.clone())), Some(Err(err)))
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("llm lock")
    }

    fn push(&self, extract: Option<Ex>, respond: Option<Rsp>) -> &Self {
        let mut state = self.lock();
        if let Some(item) = extract {
            state.extract_queue.push_back(item);
        }
        if let Some(item) = respond {
            state.respond_queue.push_back(item);
        }
        self
    }

    fn next_extract(&self) -> Option<Ex> {
        self.lock().extract_queue.pop_front()
    }
    fn next_respond(&self) -> Option<Rsp> {
        self.lock().respond_queue.pop_front()
    }

    async fn simulate(&self) {
        let latency = self.lock().latency;
        if !latency.is_zero() {
            tokio::time::sleep(latency).await;
        }
    }
}

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn extract(&self, request: ExtractionRequest) -> Result<ExtractionResult, ProviderError> {
        self.simulate().await;
        self.next_extract()
            .unwrap_or_else(|| Ok(nlu::extract(&request.utterance)))
    }

    async fn respond(&self, request: ResponseRequest) -> Result<ResponseDraft, ProviderError> {
        self.simulate().await;
        match self.next_respond() {
            Some(Ok(text)) => Ok(ResponseDraft { text }),
            Some(Err(err)) => Err(err),
            None => Ok(ResponseDraft {
                text: compose(&request.directive),
            }),
        }
    }

    fn name(&self) -> &'static str {
        "fake_llm"
    }
}
