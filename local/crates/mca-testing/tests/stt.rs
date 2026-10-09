use mca_core::domain::language::Language;
use mca_core::ports::stt::SttProvider;
use mca_testing::audio;
use mca_testing::FakeSttProvider;

#[tokio::test]
async fn stt_ignores_silence_then_plays_the_script() {
    let stt = FakeSttProvider::new();
    stt.expect("хочу отправить груз в Алматы");

    let silent = stt
        .transcribe(&audio::silence(8_000), Language::Ru)
        .await
        .unwrap();
    assert!(silent.text.is_empty());

    let heard = stt
        .transcribe(
            &audio::utterance("хочу отправить груз в Алматы"),
            Language::Ru,
        )
        .await
        .unwrap();
    assert_eq!(heard.text, "хочу отправить груз в Алматы");
    assert_eq!(stt.calls(), 1);
}

#[tokio::test]
async fn stt_ignores_scripts_across_concurrent_callers() {
    let stt = FakeSttProvider::new();
    for index in 0..50 {
        stt.expect(format!("редняя-{index}"));
    }
    let mut tasks = Vec::new();
    for _ in 0..50 {
        let clone = stt.clone();
        tasks.push(tokio::spawn(async move {
            clone
                .transcribe(&audio::utterance("звук"), Language::Ru)
                .await
                .unwrap()
                .text
        }));
    }
    let mut texts: Vec<String> = Vec::new();
    for task in tasks {
        texts.push(task.await.unwrap());
    }
    texts.sort();
    assert_eq!(texts.len(), 50);
    assert_eq!(texts.first().unwrap(), "редняя-0");
}
