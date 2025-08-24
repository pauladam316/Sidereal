// src/server.rs

use std::sync::mpsc::Sender;
use tonic::{transport::Server, Request, Response, Status};

use protos::protos::planetarium_server::{Planetarium, PlanetariumServer};
use protos::protos::SetMountLocationRequest;
use protos::protos::SetMountLocationResponse;
use protos::protos::{SetLocationRequest, SetLocationResponse};

use crate::events::PlanetariumEvent;
/// Our gRPC service, holding the channel sender
#[derive(Clone)]
pub struct MyPlanetariumServer {
    sender: Sender<PlanetariumEvent>,
}

impl MyPlanetariumServer {
    pub fn new(sender: Sender<PlanetariumEvent>) -> Self {
        Self { sender }
    }
}

/// Launch the gRPC server, giving it the channel sender.
pub async fn run(sender: Sender<PlanetariumEvent>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyPlanetariumServer::new(sender);

    println!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(PlanetariumServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tonic::async_trait]
impl Planetarium for MyPlanetariumServer {
    async fn set_location(
        &self,
        request: Request<SetLocationRequest>,
    ) -> Result<Response<SetLocationResponse>, Status> {
        let contents = request.into_inner();

        // Build the event
        let evt = PlanetariumEvent::SetSiteLocation {
            lat_deg: contents.latitude as f64,
            lon_deg: contents.longitude as f64,
        };

        // Send it into your Bevy channel
        self.sender
            .send(evt)
            .map_err(|e| Status::internal(format!("Channel send error: {}", e)))?;

        // Reply to the gRPC client
        let reply = SetLocationResponse {
            description: format!(
                "Location set: lat={}°, lon={}°",
                contents.latitude, contents.longitude
            ),
        };
        Ok(Response::new(reply))
    }

    async fn set_mount_location(
        &self,
        request: Request<SetMountLocationRequest>,
    ) -> Result<Response<SetMountLocationResponse>, Status> {
        let contents = request.into_inner();
        // Build the event
        let evt = PlanetariumEvent::SetMountPosition {
            ra_hours: contents.ra,
            dec_deg: contents.dec,
        };
        self.sender
            .send(evt)
            .map_err(|e| Status::internal(format!("Channel send error: {}", e)))?;

        let reply = SetMountLocationResponse {
            description: format!(
                "Mount Position set: ra={}°, dec={}°",
                contents.ra, contents.dec
            ),
        };
        Ok(Response::new(reply))
    }
}
