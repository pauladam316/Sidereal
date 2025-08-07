use protos::protos::planetarium_server::{Planetarium, PlanetariumServer};
use protos::protos::{SetLocationRequest, SetLocationResponse};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct MyPlanetariumServer {} // renamed for clarity

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyPlanetariumServer::default();

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

        let reply = SetLocationResponse {
            description: format!(
                "{} {} {}",
                contents.latitude, contents.longitude, contents.altitude,
            ),
        };
        Ok(Response::new(reply))
    }
}
