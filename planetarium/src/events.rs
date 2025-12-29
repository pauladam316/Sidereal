use bevy::ecs::event::Event;
use chrono::{DateTime, Utc};

#[derive(Event, Debug, Clone)]
pub enum PlanetariumEvent {
    SetSiteLocation { lat_deg: f64, lon_deg: f64 },
    #[allow(dead_code)]
    SetTime { time: DateTime<Utc> },
    SetMountPosition { ra_hours: f32, dec_deg: f32 },
}
