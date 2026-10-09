//! Provider instantiation from environment configuration.
//!
//! STT/TTS default to `Fake` (the deterministic, offline-safe fakes from
//! `mca-testing` — the only speech implementations that need no credentials).
//! `STT_PROVIDER=http` / `TTS_PROVIDER=http` switch to the OpenAI-compatible
//! HTTP adapters in `mca-core`. Telephony has no live backend in the MVP, so
//! it is always the recording fake.

use std::sync::Arc;

use anyhow::{anyhow, Context};
use mca_core::config::ai::{SttConfig, TtsConfig};
use mca_core::config::provider::ProviderKind;
use mca_core::ports::stt::{AudioFormat, SttProvider};
use mca_core::ports::telephony::TelephonyProvider;
use mca_core::ports::tts::TtsProvider;
use mca_core::provider::stt::HttpSttProvider;
use mca_core::provider::tts::HttpTtsProvider;
use mca_testing::{FakeSttProvider, FakeTelephonyProvider, FakeTtsProvider};

/// Everything the engine holds onto; each entry is already an `Arc<dyn _>`.
pub struct Providers {
    pub stt: Arc<dyn SttProvider>,
    pub tts: Arc<dyn TtsProvider>,
    pub telephony: Arc<dyn TelephonyProvider>,
}

pub fn build(stt_cfg: &SttConfig, tts_cfg: &TtsConfig) -> Result<Providers, anyhow::Error> {
    let stt: Arc<dyn SttProvider> = match stt_cfg.remote.kind {
        ProviderKind::Http => {
            let base_url = stt_cfg
                .remote
                .base_url
                .clone()
                .ok_or_else(|| anyhow!("STT_BASE_URL is required for STT_PROVIDER=http"))?;
            let model = stt_cfg
                .remote
                .model
                .clone()
                .unwrap_or_else(|| "whisper-1".into());
            Arc::new(
                HttpSttProvider::new(
                    base_url,
                    stt_cfg.remote.api_key.clone(),
                    model,
                    stt_cfg.remote.timeout,
                    stt_cfg.remote.retry_policy(),
                )
                .with_context(|| "cannot build STT http client")?,
            )
        }
        _ => Arc::new(FakeSttProvider::new()),
    };

    let tts: Arc<dyn TtsProvider> = match tts_cfg.remote.kind {
        ProviderKind::Http => {
            let base_url = tts_cfg
                .remote
                .base_url
                .clone()
                .ok_or_else(|| anyhow!("TTS_BASE_URL is required for TTS_PROVIDER=http"))?;
            let model = tts_cfg
                .remote
                .model
                .clone()
                .unwrap_or_else(|| "tts-1".into());
            let format = AudioFormat::from_name(&tts_cfg.format)
                .ok_or_else(|| anyhow!("invalid TTS_FORMAT `{}`", tts_cfg.format))?;
            Arc::new(
                HttpTtsProvider::new(
                    base_url,
                    tts_cfg.remote.api_key.clone(),
                    model,
                    tts_cfg.voice.clone(),
                    format,
                    tts_cfg.remote.timeout,
                    tts_cfg.remote.retry_policy(),
                )
                .with_context(|| "cannot build TTS http client")?,
            )
        }
        _ => Arc::new(FakeTtsProvider::default()),
    };

    let telephony: Arc<dyn TelephonyProvider> = Arc::new(FakeTelephonyProvider::new());

    Ok(Providers {
        stt,
        tts,
        telephony,
    })
}
