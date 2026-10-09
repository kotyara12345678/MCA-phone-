//! Manual multipart/form-data assembly for the STT endpoint.

use crate::ports::stt::AudioFrame;

/// Stable boundary: a constant avoids allocating a random one per call, and the
/// audio payload never contains this ASCII sequence.
pub const BOUNDARY: &str = "mcaBoundary7f3a";

pub const TRANSCRIPTION_PATH: &str = "/audio/transcriptions";

/// Builds the whole request body in one buffer: two text fields plus the audio.
pub fn build_multipart(model: &str, language: &str, audio: &AudioFrame) -> Vec<u8> {
    let mut body = Vec::with_capacity(audio.samples.len() * 2 + 512);
    push_field(&mut body, "model", model);
    push_field(&mut body, "language", language);
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"audio.pcm\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(&audio.to_bytes());
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

pub fn push_field(body: &mut Vec<u8>, name: &str, value: &str) {
    body.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(value.as_bytes());
    body.extend_from_slice(b"\r\n");
}
