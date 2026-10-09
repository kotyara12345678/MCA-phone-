//! gRPC helpers shared by the AI handlers: status mapping and request parsing.
//!
//! `AppError::code()` is the single source of truth for the gRPC status
//! translation, so a new error cannot silently become an `INTERNAL`.

use std::pin::Pin;

use futures::stream::Stream;
use mca_core::error::AppError;
use mca_core::ids::{CorrelationId, SessionId};
use mca_proto::ai::v1::SessionEvent;
use tonic::Status;

pub type FuseStream = Pin<Box<dyn Stream<Item = Result<SessionEvent, Status>> + Send + 'static>>;

pub fn parse_session(raw: &str) -> Result<SessionId, Status> {
    raw.parse::<SessionId>()
        .map_err(|err| invalid_arg(err.to_string()))
}

pub fn optional(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

pub fn optional_string(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

pub fn positive(value: i64) -> Option<i64> {
    (value > 0).then_some(value)
}

pub fn optional_correlation(raw: &str) -> Result<Option<CorrelationId>, Status> {
    if raw.is_empty() {
        return Ok(None);
    }
    raw.parse::<CorrelationId>()
        .map(Some)
        .map_err(|err| invalid_arg(err.to_string()))
}

pub fn invalid_arg(message: impl Into<String>) -> Status {
    Status::invalid_argument(message.into())
}

pub fn map_err(err: AppError) -> Status {
    let message = err.to_string();
    match err.code() {
        "not_found" | "application_not_found" => Status::not_found(message),
        "session_state" => Status::failed_precondition(message),
        "invalid_request" => Status::invalid_argument(message),
        "payload_too_large" => Status::resource_exhausted(message),
        "conflict" => Status::aborted(message),
        _ => Status::internal(message),
    }
}
