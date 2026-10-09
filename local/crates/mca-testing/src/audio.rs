//! Deterministic PCM synthesis for the fakes.
//!
//! Audio is the trickiest seam in an offline test: an STT fake must not just
//! echo a scripted transcript back (that would let a buggy pipeline pass on
//! garbage frames), so the fakes tie the waveform to the utterance. Distinct
//! texts map to distinct samples, silence maps to silence, and the mapping is
//! stable across runs and machines.

use mca_core::ports::stt::{AudioFormat, AudioFrame};

/// Any frame whose samples are at or below this absolute value is "silence".
pub const TRANSCRIPT_SILENCE_THRESHOLD: i16 = 8;

/// Builds a mono frame whose samples are seeded by the text.
///
/// The waveform mixes a sawtooth tone (so it is audibly speech-like) with a
/// deterministic PRNG keyed on the text's bytes. `ms` bounds the frame length
/// so a test can assert on `duration_ms` while never allocating huge buffers.
pub fn frame_for(text: &str, format: AudioFormat, sample_rate: u32, ms: u64) -> AudioFrame {
    let total = sample_rate as u64 * ms / 1000;
    let count = total.min(96_000) as usize;
    let mut seed = 0x9E37_79B9_u32;
    for byte in text.bytes() {
        seed ^= (byte as u32).wrapping_mul(0x7F4A_7C15);
        seed = seed.rotate_left(7);
    }
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        samples.push((((i as i32 % 32) * 950) as i16) ^ (seed as i16));
    }
    AudioFrame {
        format,
        sample_rate,
        channels: 1,
        samples,
    }
}

/// Standard one-utterance frame: 16-bit LE PCM at 8 kHz, 240 ms long.
pub fn utterance(text: &str) -> AudioFrame {
    frame_for(text, AudioFormat::PcmS16Le, 8_000, 240)
}

/// A 240 ms frame of pure silence.
pub fn silence(sample_rate: u32) -> AudioFrame {
    AudioFrame {
        format: AudioFormat::PcmS16Le,
        sample_rate,
        channels: 1,
        samples: vec![0; sample_rate as usize * 240 / 1000],
    }
}
