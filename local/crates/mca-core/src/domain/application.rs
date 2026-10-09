use serde::{Deserialize, Serialize};

use super::missing::MissingField;
use super::qualification::Qualification;

/// Canonical structured transport application. This is the single source of
/// truth persisted in `applications.state` and returned by every API.
///
/// `Option<T>` semantics: `None` means "not collected yet". Applying a
/// non-`None` value always overwrites, which is what lets a customer correct
/// an earlier answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ApplicationState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight_kg: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_m3: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_country: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub special_requirements: Option<String>,
    #[serde(default)]
    pub intent: super::intent::Intent,
    #[serde(default)]
    pub language: super::language::Language,
    #[serde(default)]
    pub missing_fields: Vec<MissingField>,
    /// Fields the customer explicitly refused to provide ("не знаю"). Kept so
    /// the agent stops asking and hands over to a human instead of looping.
    #[serde(default)]
    pub unknown_fields: Vec<MissingField>,
    #[serde(default)]
    pub qualification: Qualification,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qualification_reason: Option<String>,
}

impl ApplicationState {
    /// Recomputes `missing_fields` and the qualification verdict. Called after
    /// every merge so the stored state can never drift from the rules.
    pub fn refresh(&mut self) {
        self.missing_fields = crate::domain::coverage::compute_missing(self);
        let verdict = crate::policy::rules::evaluate(self);
        self.qualification = verdict.qualification;
        self.qualification_reason = verdict.reason;
    }

    /// True when every mandatory field is present and the intent is in scope.
    pub fn has_all_mandatory(&self) -> bool {
        self.missing_fields.iter().all(|f| !f.is_mandatory())
    }

    /// Fraction of mandatory fields collected, in `0.0..=1.0`.
    pub fn completion(&self) -> f64 {
        let total = MissingField::MANDATORY.len();
        let done = total
            - self
                .missing_fields
                .iter()
                .filter(|f| f.is_mandatory())
                .count();
        done as f64 / total as f64
    }

    /// Fields the customer explicitly said are unknown ("не знаю").
    pub fn is_declared_unknown(&self, field: MissingField) -> bool {
        self.unknown_fields.contains(&field)
    }

    /// Route label used in the email subject, e.g. `Германия → Алматы`.
    pub fn route_label_ru(&self) -> String {
        let from = self
            .origin_country
            .as_deref()
            .map(crate::policy::company::country_ru)
            .unwrap_or_else(|| "неизвестно".to_string());
        let to = self
            .destination_city
            .as_deref()
            .map(str::to_string)
            .or_else(|| self.destination_country.as_deref().map(|c| c.to_string()))
            .unwrap_or_else(|| "неизвестно".to_string());
        format!("{from} → {to}")
    }
}
