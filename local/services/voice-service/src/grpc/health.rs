//! gRPC health service: reports SERVING while the process is up.

use std::pin::Pin;

use futures::stream::{once as stream_once, Stream};
use mca_proto::health::v1::health_check_response::ServingStatus;
use mca_proto::health::v1::health_server::Health;
use mca_proto::health::v1::{HealthCheckRequest, HealthCheckResponse};
use tonic::{Request, Response, Status};

pub struct HealthGrpc;

#[tonic::async_trait]
impl Health for HealthGrpc {
    type WatchStream =
        Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send + 'static>>;

    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: ServingStatus::Serving as i32,
        }))
    }

    async fn watch(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let response = async move {
            Ok(HealthCheckResponse {
                status: ServingStatus::Serving as i32,
            })
        };
        Ok(Response::new(Box::pin(stream_once(response))))
    }
}
