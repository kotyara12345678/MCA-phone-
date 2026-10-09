use super::body::who_label;
use crate::domain::summary;
use crate::email::draft::EmailInput;

/// HTML rendering derived from the *same* structured fields as the plain-text
/// body, so the two can never disagree about what was collected.
pub fn render(input: &EmailInput, subject: &str, plain: &str) -> String {
    format!(
        "<html><body style=\"font-family:Arial,sans-serif;font-size:14px\">\
         <h2>{}</h2>\
         <p>Поступила новая заявка от AI-оператора компании {}.</p>\
         <p><b>Маршрут:</b> {}</p>\
         <table cellpadding=\"4\">{}</table>\
         <p><b>Квалификация:</b> {}</p>{}\
         <p><b>Отсутствующие данные:</b></p><ul>{}</ul>\
         <p><b>История диалога:</b></p><ul>{}</ul>\
         <p style=\"color:#888\">ID заявки: {}<br>ID сессии: {}</p>\
         <hr><pre style=\"white-space:pre-wrap\">{}</pre></body></html>",
        escape(subject),
        escape(&input.company_name),
        escape(&input.state.route_label_ru()),
        field_rows(input),
        escape(input.state.qualification.label_ru()),
        reason_paragraph(input),
        missing_list(input),
        transcript_list(input),
        input.application_id,
        input.session_id,
        escape(plain)
    )
}

fn field_rows(input: &EmailInput) -> String {
    summary::field_lines(&input.state)
        .iter()
        .map(|(k, v)| {
            format!(
                "<tr><th align=\"left\">{}</th><td>{}</td></tr>",
                escape(k),
                escape(v)
            )
        })
        .collect()
}

fn missing_list(input: &EmailInput) -> String {
    let fields = &input.state.missing_fields;
    if fields.is_empty() {
        return "<li>Все обязательные данные получены</li>".to_string();
    }
    fields
        .iter()
        .map(|f| format!("<li>{}</li>", escape(f.label_ru())))
        .collect()
}

fn transcript_list(input: &EmailInput) -> String {
    input
        .transcript
        .iter()
        .map(|m| {
            format!(
                "<li><b>{}</b>: {}</li>",
                escape(who_label(m.speaker)),
                escape(m.text.trim())
            )
        })
        .collect()
}

fn reason_paragraph(input: &EmailInput) -> String {
    input
        .state
        .qualification_reason
        .as_deref()
        .map(|r| format!("<p><b>Причина:</b> {}</p>", escape(r)))
        .unwrap_or_default()
}

/// Minimal HTML escaping; this module is the only place that emits HTML.
pub fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests;
