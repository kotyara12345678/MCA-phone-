//! The dialogue engine: the single place a "turn" is orchestrated.
//!
//! Pure against its ports — LLM, CRM and the store are all behind traits used
//! by `mca-core`, so the same engine runs on Postgres or in-memory, against a
//! live model or the deterministic rule fallback. That is the guarantee behind
//! the tests in `tests/`: they exercise the exact binaries' logic.

mod events;
mod finish;
mod handoff;
mod output;
mod queries;
mod reason;
mod stage;
mod start;
mod turn;
mod types;

use std::sync::Arc;

use mca_core::error::AppError;
use mca_core::metrics::SharedMetrics;
use mca_core::ports::crm::CrmProvider;
use mca_core::ports::llm::LlmProvider;
use tokio::sync::broadcast;

pub use events::{EventBus, SessionEvent};
pub(crate) use types::recent_turns;
pub use types::{EmailSettings, FinishOutcome, StartOutcome, TurnOutcome};

use crate::store::{ApplicationStore, SessionStore};

pub struct DialogueEngine {
    pub(crate) sessions: Arc<dyn SessionStore>,
    pub(crate) applications: Arc<dyn ApplicationStore>,
    pub(crate) llm: Arc<dyn LlmProvider>,
    pub(crate) crm: Arc<dyn CrmProvider>,
    pub(crate) metrics: SharedMetrics,
    pub(crate) events: Arc<EventBus>,
    pub(crate) email: EmailSettings,
    pub(crate) context_recent_turns: usize,
    pub(crate) max_transcript_chars: usize,
}

impl DialogueEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sessions: Arc<dyn SessionStore>,
        applications: Arc<dyn ApplicationStore>,
        llm: Arc<dyn LlmProvider>,
        crm: Arc<dyn CrmProvider>,
        metrics: SharedMetrics,
        events: EventBus,
        email: EmailSettings,
        context_recent_turns: usize,
        max_transcript_chars: usize,
    ) -> Self {
        Self {
            sessions,
            applications,
            llm,
            crm,
            metrics,
            events: Arc::new(events),
            email,
            context_recent_turns,
            max_transcript_chars,
        }
    }

    /// Liveness of the datastore, used by `/ready`.
    pub async fn ping(&self) -> Result<(), AppError> {
        self.sessions.ping().await.map_err(AppError::from)
    }

    /// The Prometheus exposition text, when metrics are enabled.
    pub fn metrics_text(&self) -> Option<String> {
        mca_core::metrics::Metrics::render(&*self.metrics)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.events.subscribe()
    }
}
