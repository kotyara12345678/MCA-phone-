//! Per-RPC handlers. Each is a small free function over `&Arc<VoiceEngine>`,
//! matching the ai-service layout.

pub mod hangup;
pub mod open;
pub mod push;
pub mod read;
pub mod stream;

pub use super::support::FuseStream;
pub use hangup::hangup;
pub use open::open_call;
pub use push::push_audio;
pub use read::{get_call, list_calls};
pub use stream::stream_audio;
