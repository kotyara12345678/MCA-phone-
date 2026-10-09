//! gRPC helpers shared by the voice handlers: status mapping and the streaming
//! event type. `VoiceError::code()` is the single source of truth for the
//! gRPC status translation.

use std::pin::Pin;

use futures::stream::Stream;
use mca_proto::voice::v1::VoiceEvent;
use tonic::Status;

use crate::error::VoiceError;

pub type FuseStream = Pin<Box<dyn Stream<Item = Result<VoiceEvent, Status>> + Send + 'static>>;

pub fn invalid_arg(message: impl Into<String>) -> Status {
    Status::invalid_argument(message.into())
}

pub fn map_err(err: VoiceError) -> Status {
    let message = err.to_string();
    match err.code() {
        "not_found" => Status::not_found(message),
        "call_state" => Status::failed_precondition(message),
        "invalid_request" => Status::invalid_argument(message),
        _ => Status::internal(message),
    }
}
