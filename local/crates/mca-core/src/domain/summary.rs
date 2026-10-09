use super::application::ApplicationState;
use crate::policy::company::{country_ru, format_volume, format_weight};

mod missing;

/// One-line business summary stored on the application row and reused as the
/// opening line of the manager email. Kept deterministic — no LLM involved.
pub fn summarize(state: &ApplicationState) -> String {
    let cargo = state.cargo.as_deref().unwrap_or("груз не указан");
    let route = format!("{} → {}", origin_label(state), destination_label(state));
    format!("Перевозка: {cargo}, маршрут {route}")
}

fn origin_label(state: &ApplicationState) -> String {
    match (&state.origin_city, &state.origin_country) {
        (Some(city), Some(country)) => format!("{city}, {}", country_ru(country)),
        (Some(city), None) => city.clone(),
        (None, Some(country)) => country_ru(country),
        (None, None) => "не указано".to_string(),
    }
}

fn destination_label(state: &ApplicationState) -> String {
    match (&state.destination_city, &state.destination_country) {
        (Some(city), Some(country)) => format!("{city}, {}", country_ru(country)),
        (Some(city), None) => city.clone(),
        (None, Some(country)) => country_ru(country),
        (None, None) => "не указано".to_string(),
    }
}

/// Multi-line field-by-field body used by the email composer. Order matches the
/// order a manager reads it in.
pub fn field_lines(state: &ApplicationState) -> Vec<(String, String)> {
    vec![
        (
            "Груз".into(),
            state.cargo.clone().unwrap_or_else(|| "Не указан".into()),
        ),
        ("Вес".into(), format_weight(state.weight_kg)),
        ("Объём".into(), format_volume(state.volume_m3)),
        (
            "Откуда".into(),
            match (&state.origin_city, &state.origin_country) {
                (Some(city), Some(country)) => format!("{city}, {}", country_ru(country)),
                (Some(city), None) => city.clone(),
                (None, Some(country)) => country_ru(country),
                (None, None) => "Не указан".into(),
            },
        ),
        (
            "Куда".into(),
            match (&state.destination_city, &state.destination_country) {
                (Some(city), Some(country)) => format!("{city}, {}", country_ru(country)),
                (Some(city), None) => city.clone(),
                (None, Some(country)) => country_ru(country),
                (None, None) => "Не указан".into(),
            },
        ),
        (
            "Дата готовности".into(),
            state
                .ready_date
                .clone()
                .unwrap_or_else(|| "Не указана".into()),
        ),
        (
            "Контакт".into(),
            state.contact.clone().unwrap_or_else(|| "Не указан".into()),
        ),
        (
            "Особые требования".into(),
            state
                .special_requirements
                .clone()
                .unwrap_or_else(|| "Нет".into()),
        ),
    ]
}

pub fn missing_sentence_ru(missing: &[crate::domain::missing::MissingField]) -> String {
    missing::missing_sentence_ru(missing)
}

#[cfg(test)]
mod tests;
