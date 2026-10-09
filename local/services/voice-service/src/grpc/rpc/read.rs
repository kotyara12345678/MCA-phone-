//! `GetCall` / `ListCalls`: control-plane reads over the call registry.

use std::sync::Arc;

use mca_proto::voice::v1::{GetCallRequest, GetCallResponse, ListCallsRequest, ListCallsResponse};
use tonic::{Request, Response, Status};

use crate::engine::VoiceEngine;
use crate::grpc::support;
use crate::map;

pub async fn get_call(
    engine: &Arc<VoiceEngine>,
    request: Request<GetCallRequest>,
) -> Result<Response<GetCallResponse>, Status> {
    let record = engine
        .get(&request.get_ref().call_id)
        .map_err(support::map_err)?;
    Ok(Response::new(GetCallResponse {
        call: Some(map::call(&record)),
    }))
}

pub async fn list_calls(
    engine: &Arc<VoiceEngine>,
    _request: Request<ListCallsRequest>,
) -> Result<Response<ListCallsResponse>, Status> {
    let records = engine.list();
    Ok(Response::new(ListCallsResponse {
        calls: records.iter().map(map::call).collect(),
    }))
}
