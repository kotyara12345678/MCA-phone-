use super::questions::QuestionTopic;

/// Keyword tables for topic detection. Kept in one place so adding a phrase
/// is a one-line change and the tables stay reviewable.
pub fn needles(topic: QuestionTopic) -> &'static [&'static str] {
    match topic {
        QuestionTopic::Price => PRICE,
        QuestionTopic::Terms => TERMS,
        QuestionTopic::Documents => DOCUMENTS,
        QuestionTopic::Customs => CUSTOMS,
        QuestionTopic::Timeline => TIMELINE,
        QuestionTopic::Insurance => INSURANCE,
        QuestionTopic::Tracking => TRACKING,
        QuestionTopic::Contact => CONTACT,
        QuestionTopic::Capabilities => CAPABILITIES,
    }
}

const PRICE: &[&str] = &[
    "сколько стоит",
    "сколько стоить",
    "стоимость",
    "цена",
    "прайс",
    "тариф",
    "по сколько",
    "дорого",
    "дешево",
    "сколько будет стоить",
    "это будет стоить",
    "будет стоить",
    "цена вопроса",
];

const TERMS: &[&str] = &[
    "условия",
    "договор",
    "оплата",
    "счет",
    "счёт",
    "оплатить",
    "рассрочка",
    "ндс",
];

const DOCUMENTS: &[&str] = &[
    "документ",
    "накладн",
    "инвойс",
    "invoice",
    "сертификат",
    "декларац",
];

const CUSTOMS: &[&str] = &["таможн", "пошлин", "тн вэд", "таможен"];

const TIMELINE: &[&str] = &[
    "срок",
    "сколько дней",
    "когда привез",
    "сколько по времени",
    "доставк",
];

const INSURANCE: &[&str] = &["страховк", "страх", "застрах"];

const TRACKING: &[&str] = &[
    "где мой груз",
    "где груз",
    "отследи",
    "статус груза",
    "накладной",
];

const CONTACT: &[&str] = &[
    "связаться",
    "позвонить",
    "телефон",
    "почт",
    "email",
    "мейл",
    "контакт",
];

const CAPABILITIES: &[&str] = &["чем занимаетесь", "что умеете", "какие перевозки", "услуги"];
