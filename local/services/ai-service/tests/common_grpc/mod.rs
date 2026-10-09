//! Shared gRPC server bootstrap for the transport tests only.
//!
//! Engine-test crates never declare this module, so `spawn` only exists where
//! it is actually used and `clippy -D warnings` stays clean.

use mca_proto::ai::v1::ai_service_client::AiServiceClient;
use mca_proto::ai::v1::ai_service_server::AiServiceServer;
use tokio_stream::wrappers::TcpListenerStream;

use crate::common::fixture;
use ai_service::grpc::AiGrpc;

pub async fn spawn() -> AiServiceClient<tonic::transport::Channel> {
    let (engine, _) = fixture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tonic::transport::Server::builder()
        .add_service(AiServiceServer::new(AiGrpc::new(engine)))
        .serve_with_incoming(TcpListenerStream::new(listener));
    tokio::spawn(async move {
        server.await.expect("grpc server failed");
    });
    AiServiceClient::connect(format!("http://{addr}"))
        .await
        .unwrap()
}
