// src/server.rs

use std::sync::mpsc::Sender;
use tonic::{transport::Server, Request, Response, Status};

use protos::protos::planetarium_server::{Planetarium, PlanetariumServer};
use protos::protos::{SetLocationRequest, SetLocationResponse};

use crate::starfield::SetLocationEvent;

/// Our gRPC service, holding the channel sender
#[derive(Clone)]
pub struct MyPlanetariumServer {
    location_sender: Sender<SetLocationEvent>,
}

impl MyPlanetariumServer {
    pub fn new(location_sender: Sender<SetLocationEvent>) -> Self {
        Self { location_sender }
    }
}

/// Launch the gRPC server, giving it the channel sender.
pub async fn run(
    location_sender: Sender<SetLocationEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyPlanetariumServer::new(location_sender);

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
        let evt = SetLocationEvent {
            lat_deg: contents.latitude as f64,
            lon_deg: contents.longitude as f64,
        };

        // Send it into your Bevy channel
        self.location_sender
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
}
