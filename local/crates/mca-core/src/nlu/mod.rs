/// Deterministic, offline natural-language understanding for the MCA Logistics
/// voice agent.
///
/// This module is the *fallback* engine, not a second source of truth: the
/// online path calls an OpenAI-compatible LLM and parses its JSON into the very
/// same [`result::ExtractionResult`]. Having a rule-based implementation means
/// an outage degrades phrasing quality, never the collected data set.
pub mod contact;
pub mod direction;
pub mod extract;
pub mod intent;
pub mod lexicon;
pub mod normalize;
pub mod place;
pub mod quantity;
pub mod ready_date;
pub mod result;
pub mod volume;

pub use extract::extract;
pub use result::ExtractionResult;
