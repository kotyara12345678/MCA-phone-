//! voice-service binary: assemble config, providers, the ai gateway and the
//! engine, then run the gRPC and HTTP listeners until a shutdown signal.
//!
//! No business rule lives here — this file only wires ports into the engine.

use std::sync::Arc;

use mca_core::config::server::CommonConfig;
use tokio_stream::wrappers::TcpListenerStream;
use tracing_subscriber::EnvFilter;
use voice_service::config::AppConfig;
use voice_service::engine::VoiceEngine;
use voice_service::{ai, grpc, http, providers};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config.common)?;

    let built = providers::build(&config.stt, &config.tts)?;
    let gateway = Arc::new(ai::GrpcAiGateway::connect(&config.ai.url, config.ai.timeout_ms).await?);
    let engine = Arc::new(VoiceEngine::new(
        built.stt,
        built.tts,
        built.telephony,
        gateway,
        config.stt.default_language.clone(),
    ));

    let http_app = http::app(http::HttpState {
        engine: engine.clone(),
        environment: config.common.environment.clone(),
    });

    let grpc_listener = tokio::net::TcpListener::bind(&config.grpc.bind_addr()).await?;
    let grpc_incoming = TcpListenerStream::new(grpc_listener);
    let grpc_server = tonic::transport::Server::builder()
        .add_service(
            mca_proto::voice::v1::voice_service_server::VoiceServiceServer::new(
                grpc::VoiceGrpc::new(engine.clone()),
            ),
        )
        .add_service(mca_proto::health::v1::health_server::HealthServer::new(
            grpc::HealthGrpc,
        ))
        .serve_with_incoming(grpc_incoming);

    tracing::info!(
        grpc = %config.grpc.bind_addr(),
        http = %config.http.bind_addr(),
        ai = %config.ai.url,
        "voice-service starting"
    );
    let http_addr = config.http.bind_addr();
    tokio::select! {
        result = grpc_server => result?,
        result = http::serve(http_app, &http_addr) => result?,
        () = shutdown_signal() => {}
    }
    tracing::info!("shutdown complete");
    Ok(())
}

fn init_tracing(common: &CommonConfig) -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&common.log_filter));
    let init = if common.log_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .try_init()
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).try_init()
    };
    init.map_err(|err| anyhow::anyhow!("tracing init failed: {err}"))
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
}
