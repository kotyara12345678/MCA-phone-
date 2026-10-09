//! Deterministic intent classification.
//!
//! Runs on normalised text so the same rules work for the fake provider and
//! for the production fallback. Order matters: checks that are cheap and
//! unambiguous (hangup, unrelated) run before slower ones.

mod lexicons;

pub use lexicons::{is_greeting, is_hangup, is_tracking, is_unrelated};

use super::place::PlaceMatch;
use crate::domain::intent::Intent;
use crate::policy::questions::QuestionTopic;

/// Classifies one normalised utterance.
pub fn detect_intent(text: &str, place: &PlaceMatch) -> Intent {
    if is_hangup(text) {
        return Intent::Unknown;
    }
    if is_unrelated(text) {
        return Intent::Unrelated;
    }
    if is_tracking(text) {
        return Intent::Tracking;
    }
    if lexicons::has_shipping_signal(text, place) {
        return Intent::ShippingRequest;
    }
    if QuestionTopic::Price.detect(text) {
        return Intent::PriceInquiry;
    }
    if is_greeting(text) {
        return Intent::Greeting;
    }
    Intent::Unknown
}
