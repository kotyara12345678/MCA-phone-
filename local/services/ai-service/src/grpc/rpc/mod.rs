//! Free-function implementations of the `AiService` RPCs.
//!
//! Taking `&Arc<DialogueEngine>` instead of the server struct keeps each RPC a
//! self-contained function, so the file can be split freely without ever
//! touching the tonic trait's single-impl requirement.

pub mod finish;
pub mod read;
pub mod start;
pub mod stream;
pub mod turn;

pub use finish::finish_session;
pub use read::{get_application, get_session};
pub use start::start_session;
pub use stream::stream_session_events;
pub use turn::process_turn;

use std::pin::Pin;

use futures::stream::Stream;
use mca_proto::ai::v1 as proto;
use tonic::Status;

pub type FuseStream =
    Pin<Box<dyn Stream<Item = Result<proto::SessionEvent, Status>> + Send + 'static>>;
