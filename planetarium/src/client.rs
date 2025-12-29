// src/client.rs

use prost_types::Timestamp;
use protos::protos::{sidereal_client::SiderealClient, GenericTrack, SetTrackingTargetRequest};

pub async fn send_event(_payload: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SiderealClient::connect("http://[::1]:50052").await?;
    println!("SENDING");
    // current UTC time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    let ts = Timestamp {
        seconds: now.as_secs() as i64,
        nanos: now.subsec_nanos() as i32,
    };

    // construct the GenericTrack
    let generic = GenericTrack {
        ra_hours: 5.0,      // e.g., 5h RA
        dec_degrees: -30.0, // e.g., -30° Dec
        time: Some(ts),
    };

    let request = SetTrackingTargetRequest {
        tracking_type: Some(
            protos::protos::set_tracking_target_request::TrackingType::GenericTrack(generic),
        ),
    };

    client.set_tracking_target(request).await?;
    println!("SENT");
    Ok(())
}
