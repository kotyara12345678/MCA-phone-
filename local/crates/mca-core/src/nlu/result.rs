use serde::{Deserialize, Serialize};

use crate::domain::intent::Intent;
use crate::domain::language::Language;
use crate::domain::missing::MissingField;

/// The **only** structured shape an LLM is allowed to return.
///
/// Two very different producers implement it:
/// * `HttpLlmProvider` — parses the model's JSON into these typed fields;
/// * `HeuristicExtractor` — deterministic rules, used offline and as the
///   fallback whenever the provider times out or returns garbage.
///
/// Because both produce the same typed struct, the dialogue engine never needs
/// to know which one ran, and a malformed model response can never reach the
/// domain. Every `Option` means "this turn provided it"; `None` never erases
/// previously collected data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ExtractionResult {
    #[serde(default)]
    pub intent: Intent,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub cargo: Option<String>,
    #[serde(default)]
    pub weight_kg: Option<f64>,
    #[serde(default)]
    pub volume_m3: Option<f64>,
    #[serde(default)]
    pub origin_country: Option<String>,
    #[serde(default)]
    pub origin_city: Option<String>,
    #[serde(default)]
    pub destination_country: Option<String>,
    #[serde(default)]
    pub destination_city: Option<String>,
    #[serde(default)]
    pub ready_date: Option<String>,
    #[serde(default)]
    pub contact: Option<String>,
    #[serde(default)]
    pub special_requirements: Option<String>,
    /// Fields the customer explicitly said they do not know.
    #[serde(default)]
    pub unknown_fields: Vec<MissingField>,
    /// Set when the customer asked to stop / hang up.
    #[serde(default)]
    pub hangup_requested: bool,
    /// Set when the whole turn is a greeting with no information.
    #[serde(default)]
    pub greeting_only: bool,
}

impl ExtractionResult {
    /// True when the turn carried no actionable information at all.
    pub fn is_empty(&self) -> bool {
        self.cargo.is_none()
            && self.weight_kg.is_none()
            && self.volume_m3.is_none()
            && self.origin_country.is_none()
            && self.destination_country.is_none()
            && self.ready_date.is_none()
            && self.contact.is_none()
            && self.special_requirements.is_none()
    }
}
