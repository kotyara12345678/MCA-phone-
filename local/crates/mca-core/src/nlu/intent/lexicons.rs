//! Marker lexicons and the single-utterance predicates built on them.

use crate::nlu::place::PlaceMatch;

const SHIPPING_VERBS: &[&str] = &[
    "перевез",
    "перевоз",
    "достав",
    "отправ",
    "везти",
    "транспорт",
    "перекат",
    "ship",
    "send",
    "deliver",
    "cargo",
    "груз",
];

const TRACKING_MARKERS: &[&str] = &[
    "где мой груз",
    "где груз",
    "отследи",
    "статус груза",
    "накладной",
    "уже привез",
    "трек",
    "track",
];

const UNRELATED_MARKERS: &[&str] = &[
    "работа вакансии",
    "вакансии",
    "собеседование",
    "погода",
    "курс доллара",
    "рецепт",
    "жалоба на водителя",
    "техподдержка роутера",
    "подключить интернет",
    "записаться к врачу",
    "купить продукт",
    "записать на прием",
];

pub fn is_hangup(text: &str) -> bool {
    const HANGUP: &[&str] = &[
        "всего",
        "до свидания",
        "пока",
        "заканчива",
        "не беспокой",
        "свяжитесь",
        "перезвоните",
        "hang up",
    ];
    HANGUP.iter().any(|m| text.contains(m))
}

pub fn is_greeting(text: &str) -> bool {
    const GREETING: &[&str] = &[
        "здравствуй",
        "добрый день",
        "доброе утро",
        "добрый вечер",
        "привет",
        "хало",
    ];
    GREETING.iter().any(|m| text.contains(m)) && text.split_whitespace().count() <= 6
}

pub fn is_unrelated(text: &str) -> bool {
    UNRELATED_MARKERS.iter().any(|m| text.contains(m))
}

pub fn is_tracking(text: &str) -> bool {
    TRACKING_MARKERS.iter().any(|m| text.contains(m))
}

/// A shipping request is signalled by a transport verb **plus** either a place
/// or a quantity — a single bare "груз" is not enough.
pub fn has_shipping_signal(text: &str, place: &PlaceMatch) -> bool {
    let verb = SHIPPING_VERBS.iter().any(|v| text.contains(v));
    let has_place = !place.is_empty();
    let has_quantity = text.contains("кг")
        || text.contains("тонн")
        || text.contains(" м3")
        || text.contains("куб");
    verb && (has_place || has_quantity)
}
