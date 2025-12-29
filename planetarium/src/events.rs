use bevy::prelude::Message;
use chrono::{DateTime, Utc};

#[derive(Message, Debug, Clone)]
pub enum PlanetariumEvent {
    SetSiteLocation {
        lat_deg: f64,
        lon_deg: f64,
    },
    #[allow(dead_code)]
    SetTime {
        time: DateTime<Utc>,
    },
    SetMountPosition {
        ra_hours: f32,
        dec_deg: f32,
    },
}
