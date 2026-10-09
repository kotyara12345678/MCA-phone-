//! ai-service binary: assemble config, providers, store and engine, then run
//! the gRPC and HTTP listeners side by side until a shutdown signal arrives.
//!
//! No business rule lives here — this file only wires ports into the engine.

use std::sync::Arc;

use ai_service::config::AppConfig;
use ai_service::engine::{DialogueEngine, EmailSettings, EventBus};
use ai_service::{db, grpc, http, metrics, providers};
use mca_core::config::server::CommonConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_tracing(&config.common)?;

    let shared_metrics = metrics::build(config.common.metrics_enabled);
    let (sessions, applications) = db::connect(&config.db).await?;
    let built = providers::build(&config.llm, &config.crm)?;

    let engine = Arc::new(DialogueEngine::new(
        Arc::new(sessions),
        Arc::new(applications),
        built.llm,
        built.crm,
        shared_metrics,
        EventBus::new(256),
        EmailSettings {
            enabled: config.crm.email_enabled,
            manager_email: config.crm.manager_email.clone(),
            company_name: config.crm.company_name.clone(),
            transcript_line_limit: 20,
        },
        config.llm.context_recent_turns,
        2_000,
    ));

    let http_app = http::app(http::HttpState {
        engine: engine.clone(),
        environment: config.common.environment.clone(),
    });

    let grpc_listener = tokio::net::TcpListener::bind(&config.grpc.bind_addr()).await?;
    let grpc_incoming = tokio_stream::wrappers::TcpListenerStream::new(grpc_listener);
    let grpc_server = tonic::transport::Server::builder()
        .add_service(mca_proto::ai::v1::ai_service_server::AiServiceServer::new(
            grpc::AiGrpc::new(engine.clone()),
        ))
        .add_service(mca_proto::health::v1::health_server::HealthServer::new(
            grpc::HealthGrpc,
        ))
        .serve_with_incoming(grpc_incoming);

    tracing::info!(grpc = %config.grpc.bind_addr(), http = %config.http.bind_addr(), "ai-service starting");
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
