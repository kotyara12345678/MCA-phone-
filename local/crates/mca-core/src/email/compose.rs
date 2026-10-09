use crate::email::draft::{EmailDraft, EmailInput};

/// «Новая заявка на перевозку — Германия → Алматы»
pub fn subject_line(input: &EmailInput) -> String {
    use crate::domain::qualification::Qualification;
    let prefix = match input.state.qualification {
        Qualification::Rejected => "Обращение не по перевозке",
        Qualification::Qualified => "Новая заявка на перевозку",
        _ => "Новая заявка на перевозку (уточнить данные)",
    };
    format!("{prefix} — {}", input.state.route_label_ru())
}

/// Renders the business email a manager receives for every finished call.
///
/// Deterministic by design: no model in this path. A manager must be able to
/// trust that the text contains nothing the customer did not say, and that the
/// missing-data list matches the qualification reason exactly.
pub fn compose(input: &EmailInput) -> EmailDraft {
    let subject = subject_line(input);
    let plain = super::body::plain_body(input);
    let html = super::html::render(input, &subject, &plain);
    EmailDraft {
        id: crate::ids::EmailId::new(),
        application_id: input.application_id,
        session_id: input.session_id,
        to: input.manager_email.clone(),
        subject,
        plain_text_body: plain,
        html_body: html,
        summary: crate::domain::summary::summarize(&input.state),
        missing_fields: input
            .state
            .missing_fields
            .iter()
            .map(|f| f.label_ru().to_string())
            .collect(),
        qualification: input.state.qualification.label_ru().to_string(),
        generated_at: chrono::Utc::now(),
    }
}
