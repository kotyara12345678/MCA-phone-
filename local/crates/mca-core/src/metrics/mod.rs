pub mod registry;

pub use registry::{
    Histogram, Metrics, MetricsRegistry, NoopMetrics, SharedMetrics, TurnCounter, BUCKETS_MS,
};

/// Canonical metric names. Centralised so dashboards and tests cannot drift.
pub mod names {
    pub const CALL_DURATION: &str = "call_duration_ms";
    pub const STT_LATENCY: &str = "stt_latency_ms";
    pub const LLM_LATENCY: &str = "llm_latency_ms";
    pub const TTS_LATENCY: &str = "tts_latency_ms";
    pub const TOTAL_RESPONSE_LATENCY: &str = "total_response_latency_ms";
    pub const FIRST_TOKEN_LATENCY: &str = "first_token_latency_ms";
    pub const PROVIDER_ERRORS: &str = "provider_errors";
    pub const PROVIDER_CALLS: &str = "provider_calls";
    pub const ACTIVE_SESSIONS: &str = "active_sessions";
    pub const SESSIONS_STARTED: &str = "sessions_started";
    pub const CALLS_FINISHED: &str = "calls_finished";
    pub const QUALIFIED_CALLS: &str = "qualified_calls";
    pub const REJECTED_CALLS: &str = "rejected_calls";
    pub const HUMAN_REVIEW_CALLS: &str = "human_review_calls";
    pub const TURNS_PROCESSED: &str = "turns_processed";
    pub const HTTP_REQUESTS: &str = "http_requests";
    pub const GRPC_REQUESTS: &str = "grpc_requests";
}
