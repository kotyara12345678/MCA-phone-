use super::*;

#[test]
fn audio_format_names_parse() {
    assert_eq!(
        crate::ports::stt::AudioFormat::from_name("pcm16"),
        Some(crate::ports::stt::AudioFormat::PcmS16Le)
    );
    assert_eq!(crate::ports::stt::AudioFormat::from_name("nope"), None);
}

#[test]
fn provider_kinds_parse() {
    assert_eq!(ProviderKind::parse("fake").unwrap(), ProviderKind::Fake);
    assert_eq!(ProviderKind::parse("OpenAI").unwrap(), ProviderKind::Http);
    assert!(ProviderKind::parse("wat").is_err());
}
