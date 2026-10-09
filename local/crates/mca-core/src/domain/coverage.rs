use super::application::ApplicationState;
use super::missing::MissingField;

/// Recomputes which fields are still missing. Volume and contact are optional
/// (a manager can chase them), everything else blocks a qualified handover.
pub fn compute_missing(state: &ApplicationState) -> Vec<MissingField> {
    let mut missing = Vec::new();
    if state.cargo.is_none() {
        missing.push(MissingField::Cargo);
    }
    if state.weight_kg.is_none() {
        missing.push(MissingField::Weight);
    }
    if state.destination_country.is_none() && state.destination_city.is_none() {
        missing.push(MissingField::Destination);
    }
    if state.origin_country.is_none() && state.origin_city.is_none() {
        missing.push(MissingField::Origin);
    }
    if state.ready_date.is_none() {
        missing.push(MissingField::ReadyDate);
    }
    if state.contact.is_none() {
        missing.push(MissingField::Contact);
    }
    missing
}

/// Volume is never "missing" — it is simply optional, so the agent must not
/// burn a turn asking for it. Exposed as a function for symmetry with
/// [`compute_missing`] and to make the decision explicit in tests.
pub fn volume_is_optional() -> bool {
    true
}

/// Progress ratio over mandatory fields only, used by `/ready`-style reports
/// and by the load test to assert convergence.
pub fn mandatory_completion(state: &ApplicationState) -> f64 {
    state.completion()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_is_missing_everything_except_volume() {
        let state = ApplicationState::default();
        let missing = compute_missing(&state);
        assert!(!missing.contains(&MissingField::Volume));
        assert_eq!(missing.len(), MissingField::ALL.len() - 1);
    }

    #[test]
    fn destination_city_alone_satisfies_destination() {
        let state = ApplicationState {
            destination_city: Some("Almaty".into()),
            ..Default::default()
        };
        assert!(!compute_missing(&state).contains(&MissingField::Destination));
    }

    #[test]
    fn origin_city_alone_satisfies_origin() {
        let state = ApplicationState {
            origin_city: Some("Hamburg".into()),
            ..Default::default()
        };
        assert!(!compute_missing(&state).contains(&MissingField::Origin));
    }
}
