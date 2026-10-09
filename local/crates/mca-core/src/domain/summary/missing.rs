//! Human-readable sentence listing what is still missing before qualification.

use crate::domain::missing::MissingField as F;

/// Renders the human-readable missing-data sentence, e.g.
/// «Не указана дата готовности и контакт клиента.»
pub fn missing_sentence_ru(missing: &[F]) -> String {
    let labels: Vec<&str> = match missing {
        [] => return String::new(),
        [F::Cargo] => vec!["характер груза"],
        [F::Weight] => vec!["вес груза"],
        [F::Origin] => vec!["место отправления"],
        [F::Destination] => vec!["место назначения"],
        [F::ReadyDate] => vec!["дата готовности"],
        [F::Contact] => vec!["контакт клиента"],
        [F::Volume] => vec!["объём груза"],
        other => other.iter().map(|f| f.label_ru()).collect(),
    };
    match labels.len() {
        1 => format!("Не указан: {}.", labels[0]),
        2 => format!("Не указаны: {} и {}.", labels[0], labels[1]),
        _ => {
            let head = labels[..labels.len() - 1].join(", ");
            format!("Не указаны: {} и {}.", head, labels[labels.len() - 1])
        }
    }
}
