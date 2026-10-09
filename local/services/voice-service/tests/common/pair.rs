//! In-process ai-service + voice-service pair over real gRPC.
//!
//! The ai-service runs its real `DialogueEngine` with the deterministic
//! rule-based LLM; the voice-service runs the real engine over the STT/TTS/
//! telephony fakes. Two real gRPC surfaces, no credentials, no hardware —
//! the offline backbone for the cross-service flow test.

use std::sync::Arc;

use ai_service::engine::{DialogueEngine, EmailSettings, EventBus};
use ai_service::grpc::AiGrpc;
use ai_service::store::mem::MemStore;
use mca_core::metrics::NoopMetrics;
use mca_core::ports::crm::CrmProvider;
use mca_core::ports::llm::LlmProvider;
use mca_core::provider::crm::LoggingCrmProvider;
use mca_core::provider::openai::RuleBasedLlmProvider;
use mca_proto::ai::v1::ai_service_server::AiServiceServer;
use mca_proto::voice::v1::voice_service_client::VoiceServiceClient;
use mca_proto::voice::v1::voice_service_server::VoiceServiceServer;
use mca_testing::{FakeSttProvider, FakeTelephonyProvider, FakeTtsProvider};
use tokio_stream::wrappers::TcpListenerStream;
use voice_service::ai::GrpcAiGateway;
use voice_service::engine::VoiceEngine;
use voice_service::grpc::VoiceGrpc;

pub type Client = VoiceServiceClient<tonic::transport::Channel>;

pub struct Environment {
    pub client: Client,
    pub stt: Arc<FakeSttProvider>,
    pub tts: Arc<FakeTtsProvider>,
    pub telephony: Arc<FakeTelephonyProvider>,
}

pub async fn pair() -> Environment {
    let store = Arc::new(MemStore::new());
    let llm: Arc<dyn LlmProvider> = Arc::new(RuleBasedLlmProvider::new());
    let crm: Arc<dyn CrmProvider> = Arc::new(LoggingCrmProvider);
    let ai_engine = Arc::new(DialogueEngine::new(
        store.clone(),
        store.clone(),
        llm,
        crm,
        Arc::new(NoopMetrics),
        EventBus::new(32),
        EmailSettings {
            enabled: true,
            manager_email: "logistics@mca-logistics.example".into(),
            company_name: "MCA Logistics".into(),
            transcript_line_limit: 20,
        },
        8,
        2_000,
    ));

    let ai_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ai_addr = ai_listener.local_addr().unwrap();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(AiServiceServer::new(AiGrpc::new(ai_engine)))
            .serve_with_incoming(TcpListenerStream::new(ai_listener))
            .await
            .expect("ai grpc server failed");
    });

    let stt = Arc::new(FakeSttProvider::new());
    let tts = Arc::new(FakeTtsProvider::default());
    let telephony = Arc::new(FakeTelephonyProvider::new());
    let gateway = Arc::new(
        GrpcAiGateway::connect(&format!("http://{ai_addr}"), 15_000)
            .await
            .unwrap(),
    );
    let voice_engine = Arc::new(VoiceEngine::new(
        stt.clone(),
        tts.clone(),
        telephony.clone(),
        gateway,
        "ru".to_string(),
    ));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(VoiceServiceServer::new(VoiceGrpc::new(voice_engine)))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("voice grpc server failed");
    });

    let client = VoiceServiceClient::connect(format!("http://{addr}"))
        .await
        .unwrap();
    Environment {
        client,
        stt,
        tts,
        telephony,
    }
}
