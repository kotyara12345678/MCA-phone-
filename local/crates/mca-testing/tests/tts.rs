use mca_core::domain::language::Language;
use mca_core::ports::tts::TtsProvider;
use mca_testing::FakeTtsProvider;

#[tokio::test]
async fn tts_records_and_splits_into_partials() {
    let tts = FakeTtsProvider::default().with_chunks(3);
    let parts = tts
        .synthesize_chunked("один два три четыре пять шесть", Language::Ru)
        .await
        .unwrap();
    assert_eq!(parts.len(), 3);
    assert!(parts[0].is_partial);
    assert!(!parts[2].is_partial);
    assert_eq!(parts[1].text, "три четыре");
    assert_eq!(tts.spoken(), vec!["один два три четыре пять шесть"]);
    assert!(!parts[0].audio.is_silent(8));
}

#[tokio::test]
async fn tts_unary_returns_the_final_chunk() {
    let tts = FakeTtsProvider::default().with_chunks(2);
    let single = tts.synthesize("а б в г д е", Language::Ru).await.unwrap();
    assert_eq!(single.text, "г д е");
    assert!(!single.is_partial);
    assert_eq!(tts.spoken(), vec!["а б в г д е"]);
}
