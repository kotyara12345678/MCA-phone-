use crate::domain::application::ApplicationState;
use crate::domain::intent::Intent;
use crate::domain::missing::MissingField;

/// Human-readable qualification reasons. Centralised so the HTTP API, the
/// email composer and the tests always produce byte-identical text.
pub fn no_signal() -> String {
    "Клиент ещё не описал задачу на перевозку.".to_string()
}

pub fn tracking() -> String {
    "Требуется проверка статуса груза в системе перевозчика.".to_string()
}

pub fn out_of_scope(intent: Intent) -> String {
    match intent {
        Intent::Unrelated => "Обращение не относится к перевозкам и передаче грузов.".to_string(),
        other => format!(
            "Обращение классифицировано как `{other}` и не может быть обработано автоматически."
        ),
    }
}

pub fn missing_mandatory(fields: &[MissingField]) -> String {
    let labels: Vec<&str> = fields.iter().map(|f| f.label_ru()).collect();
    join_ru(&labels, "Не указан", "Не указаны")
}

pub fn declined(field: MissingField) -> String {
    format!(
        "Клиент не смог указать {} — требуется уточнение менеджера.",
        field.label_ru()
    )
}

pub fn qualified(state: &ApplicationState) -> String {
    let mut parts = vec![format!(
        "Обязательные данные получены ({}%)",
        (state.completion() * 100.0).round() as i64
    )];
    if state.contact.is_none() {
        parts.push("контакт уточняется при звонке менеджера".to_string());
    }
    if state.volume_m3.is_none() {
        parts.push("объём уточняется по факту".to_string());
    }
    parts.join("; ")
}

/// Russian list joiner: «а», «а», ..., «и» for the final element.
pub fn join_ru(items: &[&str], singular_prefix: &str, plural_prefix: &str) -> String {
    match items.len() {
        0 => String::new(),
        1 => format!("{singular_prefix}: {}.", items[0]),
        2 => format!("{plural_prefix} {} и {}.", items[0], items[1]),
        _ => {
            let head = items[..items.len() - 1].join(", ");
            format!("{plural_prefix} {} и {}.", head, items[items.len() - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_two_items() {
        assert_eq!(
            join_ru(&["а", "б"], "Не указан", "Не указаны"),
            "Не указаны а и б."
        );
    }

    #[test]
    fn joins_three_items() {
        assert_eq!(join_ru(&["а", "б", "в"], "Нет", "Нет"), "Нет а, б и в.");
    }

    #[test]
    fn qualified_reason_mentions_completion() {
        let state = ApplicationState {
            weight_kg: Some(1.0),
            ..Default::default()
        };
        assert!(qualified(&state).contains('%'));
    }
}
