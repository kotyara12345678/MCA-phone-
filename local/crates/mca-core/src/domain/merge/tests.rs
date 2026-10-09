use super::*;
use crate::domain::intent::Intent;
use crate::nlu::extract;

#[test]
fn none_never_erases_collected_data() {
    let mut state = ApplicationState {
        cargo: Some("Оборудование".into()),
        ..Default::default()
    };
    apply(&mut state, &ExtractionResult::default());
    assert_eq!(state.cargo.as_deref(), Some("Оборудование"));
}

#[test]
fn destination_can_be_changed() {
    let mut state = ApplicationState {
        destination_city: Some("Almaty".into()),
        ..Default::default()
    };
    apply(&mut state, &extract::extract("нет, в астану"));
    assert_eq!(state.destination_city.as_deref(), Some("Astana"));
}

#[test]
fn intent_escalates_but_never_demotes() {
    let mut state = ApplicationState {
        intent: Intent::ShippingRequest,
        ..Default::default()
    };
    apply(
        &mut state,
        &ExtractionResult {
            intent: Intent::Greeting,
            ..Default::default()
        },
    );
    assert_eq!(state.intent, Intent::ShippingRequest);
}

#[test]
fn price_question_keeps_shipping_intent() {
    let mut state = ApplicationState {
        intent: Intent::ShippingRequest,
        ..Default::default()
    };
    apply(
        &mut state,
        &ExtractionResult {
            intent: Intent::PriceInquiry,
            ..Default::default()
        },
    );
    assert_eq!(state.intent, Intent::ShippingRequest);
}
