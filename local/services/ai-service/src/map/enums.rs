//! Scalar enum and language conversions between domain and proto forms.

use mca_core::domain::dialogue::DialogueStage;
use mca_core::domain::language::Language;
use mca_core::domain::qualification::Qualification;
use mca_core::domain::session::SessionStatus;
use mca_core::domain::speaker::Speaker;
use mca_proto::ai::v1 as proto;

/// Maps a domain speaker. `System` has no proto equivalent, so it degrades to
/// "unspecified" rather than pretending to be a customer.
pub fn speaker(value: Speaker) -> proto::Speaker {
    match value {
        Speaker::Agent => proto::Speaker::Agent,
        Speaker::Customer => proto::Speaker::Customer,
        Speaker::System => proto::Speaker::Unspecified,
    }
}

pub fn qualification(value: Qualification) -> proto::Qualification {
    match value {
        Qualification::Pending => proto::Qualification::Pending,
        Qualification::Qualified => proto::Qualification::Qualified,
        Qualification::Rejected => proto::Qualification::Rejected,
        Qualification::NeedsHuman => proto::Qualification::NeedsHuman,
    }
}

pub fn session_status(value: SessionStatus) -> proto::SessionStatus {
    match value {
        SessionStatus::Active => proto::SessionStatus::Active,
        SessionStatus::Finished => proto::SessionStatus::Finished,
        SessionStatus::Failed => proto::SessionStatus::Failed,
    }
}

pub fn application_status(value: Qualification) -> proto::ApplicationStatus {
    match value {
        Qualification::Pending => proto::ApplicationStatus::Draft,
        Qualification::Qualified => proto::ApplicationStatus::Qualified,
        Qualification::Rejected => proto::ApplicationStatus::Rejected,
        Qualification::NeedsHuman => proto::ApplicationStatus::NeedsHuman,
    }
}

pub fn stage(value: DialogueStage) -> proto::DialogueStage {
    use proto::DialogueStage as P;
    match value {
        DialogueStage::Greeting => P::Greeting,
        DialogueStage::CollectingCargo => P::CollectingCargo,
        DialogueStage::CollectingWeight => P::CollectingWeight,
        DialogueStage::CollectingVolume => P::CollectingVolume,
        DialogueStage::CollectingOrigin => P::CollectingOrigin,
        DialogueStage::CollectingDestination => P::CollectingDestination,
        DialogueStage::CollectingReadyDate => P::CollectingReadyDate,
        DialogueStage::CollectingContact => P::CollectingContact,
        DialogueStage::ReadyForQualification => P::ReadyForQualification,
        DialogueStage::Qualified => P::Qualified,
        DialogueStage::Rejected => P::Rejected,
        DialogueStage::NeedsHuman => P::NeedsHuman,
        DialogueStage::Completed => P::Completed,
    }
}

/// Best-effort language override from the request; an empty string means "auto"
/// and resolves to Russian (the deterministic detect behaviour).
pub fn parse_language(raw: &str) -> Result<Language, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "ru" | "ru-ru" => Ok(Language::Ru),
        "kk" | "kk-kz" => Ok(Language::Kk),
        "en" | "en-us" => Ok(Language::En),
        "de" | "de-de" => Ok(Language::De),
        other => Err(format!("unsupported language `{other}`")),
    }
}
