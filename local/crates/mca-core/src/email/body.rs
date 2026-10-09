use crate::domain::summary;
use crate::email::draft::{EmailDraft, EmailInput};

/// Plain-text body: a normal business letter, never a JSON dump.
pub fn plain_body(input: &EmailInput) -> String {
    let state = &input.state;
    let mut out = String::with_capacity(1024);
    out.push_str("Здравствуйте!\n\nПоступила новая заявка от AI-оператора компании ");
    out.push_str(&input.company_name);
    out.push_str(".\n\n");
    out.push_str(&format!("Маршрут:\n{}\n\n", state.route_label_ru()));
    out.push_str("Собранные данные:\n");
    for (label, value) in summary::field_lines(state) {
        out.push_str(&format!("{label}: {value}\n"));
    }
    out.push('\n');
    out.push_str(&format!(
        "Квалификация: {}\n",
        state.qualification.label_ru()
    ));
    if let Some(reason) = &state.qualification_reason {
        out.push_str(&format!("Причина: {reason}\n"));
    }
    out.push('\n');
    out.push_str("Отсутствующие данные:\n");
    if state.missing_fields.is_empty() {
        out.push_str("Все обязательные данные получены.\n");
    } else {
        for field in &state.missing_fields {
            out.push_str(&format!("- {}\n", field.label_ru()));
        }
    }
    out.push('\n');
    out.push_str("Краткое содержание разговора:\n");
    out.push_str(&EmailDraft::transcript_excerpt(
        &input.transcript,
        input.transcript_line_limit,
    ));
    out.push_str("\n\nИстория диалога:\n");
    for message in &input.transcript {
        out.push_str(&format!(
            "{}: {}\n",
            who_label(message.speaker),
            message.text.trim()
        ));
    }
    out.push_str(&format!(
        "\nID заявки: {}\nID сессии: {}\n",
        input.application_id, input.session_id
    ));
    out
}

pub(crate) fn who_label(speaker: crate::domain::speaker::Speaker) -> &'static str {
    match speaker {
        crate::domain::speaker::Speaker::Customer => "Клиент",
        crate::domain::speaker::Speaker::Agent => "AI-оператор",
        crate::domain::speaker::Speaker::System => "Система",
    }
}
