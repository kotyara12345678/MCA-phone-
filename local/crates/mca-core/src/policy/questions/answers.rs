//! Company-approved answers, in Russian, for each question topic.
//!
//! These strings are the source of truth for what the agent may claim. Prices
//! are never invented — a manager calculates them — so the wording for
//! `Price` deliberately promises a call-back, not a tariff.

use super::QuestionTopic;

pub fn answer_ru(topic: QuestionTopic) -> &'static str {
    match topic {
        QuestionTopic::Price => {
            "Стоимость перевозки зависит от характера груза, веса, объёма и маршрута, \
             поэтому я фиксирую данные и передаю их менеджеру для точного расчёта."
        }
        QuestionTopic::Terms => {
            "Условия работы зависят от направления и типа груза, их подтвердит менеджер."
        }
        QuestionTopic::Documents => {
            "Для международной перевозки обычно нужны товаросопроводительные документы \
             и, в зависимости от груза, дополнительные. Состав подтвердит менеджер."
        }
        QuestionTopic::Customs => {
            "Таможенное оформление и оплата пошлин зависят от страны и кода товара — \
             это уточняет менеджер."
        }
        QuestionTopic::Timeline => {
            "Сроки зависят от маршрута, способа перевозки и загрузки. Ориентировочный \
             срок рассчитает менеджер после уточнения данных."
        }
        QuestionTopic::Insurance => {
            "Страхование груза оформляется по желанию клиента, условия подтвердит менеджер."
        }
        QuestionTopic::Tracking => {
            "Отследить груз по номеру накладной может менеджер после оформления заявки."
        }
        QuestionTopic::Contact => {
            "Менеджер свяжется с вами по указанному контакту. Если контакта нет, \
             назовите, пожалуйста, имя и телефон."
        }
        QuestionTopic::Capabilities => {
            "Мы организуем международные перевозки сборных и генеральных грузов \
             автомобильным, железнодорожным и морским транспортом."
        }
    }
}
