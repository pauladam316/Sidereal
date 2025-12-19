// src/server.rs

use protos::protos::sidereal_server::{Sidereal, SiderealServer};
use std::sync::mpsc::Sender;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use protos::protos::planetarium_server::{Planetarium, PlanetariumServer};
use protos::protos::{SetLocationRequest, SetLocationResponse};
use protos::protos::{SetMountLocationRequest, SetTrackingTargetResponse};
use protos::protos::{SetMountLocationResponse, SetTrackingTargetRequest};

use crate::model::{SiderealError, SiderealResult};

#[derive(Debug, Clone)]
pub enum ForwardedRPC {
    SetTrackingTargetRequest(SetTrackingTargetRequest),
}

#[derive(Clone)]
struct SiderealServerInstance {
    tx: mpsc::UnboundedSender<ForwardedRPC>,
}

impl SiderealServerInstance {
    pub fn new(tx: mpsc::UnboundedSender<ForwardedRPC>) -> Self {
        Self { tx }
    }
}

#[tonic::async_trait]
impl Sidereal for SiderealServerInstance {
    async fn set_tracking_target(
        &self,
        request: Request<SetTrackingTargetRequest>,
    ) -> Result<Response<SetTrackingTargetResponse>, Status> {
        let cmd = request.into_inner();

        // Forward to Iced; if GUI is gone, report gracefully.
        if self
            .tx
            .send(ForwardedRPC::SetTrackingTargetRequest(cmd.clone()))
            .is_err()
        {
            return Ok(Response::new(SetTrackingTargetResponse {
                description: "GUI not available".into(),
            }));
        }

        Ok(Response::new(SetTrackingTargetResponse {
            description: "success".into(),
        }))
    }
}

pub async fn run(tx: mpsc::UnboundedSender<ForwardedRPC>) -> SiderealResult<()> {
    let addr = "[::1]:50052"
        .parse()
        .map_err(|e: std::net::AddrParseError| SiderealError::GrpcError(e.to_string()))?;
    println!("gRPC server listening on {}", addr);
    Server::builder()
        .add_service(SiderealServer::new(SiderealServerInstance::new(tx)))
        .serve(addr)
        .await
        .map_err(|e| SiderealError::GrpcError(e.to_string()))
}
