//! Overpass Planner
//!
//! A crate for planning satellite overpasses.

use chrono::{DateTime, Duration, FixedOffset, Utc};
use thiserror::Error;

pub mod planning;
pub mod tle;

pub use planning::ObserverLocation;
use planning::{
    calculate_alt_az, find_max_elevation, find_rise_time, find_set_time, is_night_at_location,
    is_satellite_lit,
};
use tle::fetch_tle;
pub use tle::get_satellite_name;

/// Result type alias for overpass planner operations.
pub type OverpassPlannerResult<T> = Result<T, OverpassPlannerError>;

/// Error types for overpass planner operations.
#[derive(Error, Debug, Clone)]
pub enum OverpassPlannerError {
    #[error("TLEError: {0}")]
    TLEError(String),
    #[error("CalculationError: {0}")]
    CalculationError(String),
    #[error("NetworkError: {0}")]
    NetworkError(String),
    #[error("ParseError: {0}")]
    ParseError(String),
    #[error("InvalidInput: {0}")]
    InvalidInput(String),
}

/// Represents a satellite overpass with timing and elevation information.
#[derive(Debug, Clone)]
pub struct Overpass {
    /// Start time of the overpass
    pub start_time: DateTime<Utc>,
    /// End time of the overpass
    pub end_time: DateTime<Utc>,
    /// Maximum elevation reached during the overpass (degrees)
    pub max_elevation: f64,
    /// Midpoint time of the overpass
    pub midpoint_time: DateTime<Utc>,
    /// Whether the overpass occurs during nighttime (sun below -6° horizon)
    pub is_night: bool,
    /// Whether the satellite is illuminated by the sun during the overpass
    pub is_lit: bool,
}

/// Represents a satellite position at a specific time.
#[derive(Debug, Clone)]
pub struct SatellitePosition {
    /// Timestamp of this position
    pub timestamp: DateTime<Utc>,
    /// Altitude angle (degrees, 0-90)
    pub altitude: f64,
    /// Azimuth angle (degrees, 0-360)
    pub azimuth: f64,
}

/// Get all overpasses for a satellite within a specified time window.
///
/// # Arguments
/// * `norad_id` - The NORAD ID of the satellite
/// * `location` - Observer's location on Earth
/// * `time_from_now` - Duration from now to search for overpasses
///
/// # Returns
/// A vector of overpasses, each containing start time, end time, max elevation, and midpoint time.
pub async fn get_overpasses(
    norad_id: u32,
    location: ObserverLocation,
    time_from_now: Duration,
) -> OverpassPlannerResult<Vec<Overpass>> {
    // Fetch TLE data
    let tle = fetch_tle(norad_id).await?;

    let start_time = Utc::now();
    let end_time = start_time + time_from_now;

    // Search step: 1 minute intervals for initial detection
    let search_step = Duration::minutes(1);
    // Refinement step: 1 second for finding exact rise/set times
    let refine_step = Duration::seconds(1);

    let mut overpasses = Vec::new();
    let mut current_overpass: Option<(DateTime<Utc>, f64)> = None; // (start_time, max_elevation)

    // Initial check at start time
    let (altitude, _) = calculate_alt_az(&tle, location, start_time)?;
    let mut was_above_horizon = altitude > 0.0;
    if was_above_horizon {
        current_overpass = Some((start_time, altitude));
    }

    // Search through the time window
    let mut current_time = start_time + search_step;
    while current_time <= end_time {
        let (altitude, _) = match calculate_alt_az(&tle, location, current_time) {
            Ok(result) => result,
            Err(_) => {
                // If calculation fails, skip this time point
                current_time += search_step;
                continue;
            }
        };

        let is_above_horizon = altitude > 0.0;

        if is_above_horizon && !was_above_horizon {
            // Satellite rising above horizon - start of overpass
            let rise_time = find_rise_time(
                &tle,
                location,
                current_time - search_step,
                current_time,
                refine_step,
            )?;
            current_overpass = Some((rise_time, altitude));
        } else if !is_above_horizon && was_above_horizon {
            // Satellite setting below horizon - end of overpass
            if let Some((start, _)) = current_overpass.take() {
                let set_time = find_set_time(
                    &tle,
                    location,
                    current_time - search_step,
                    current_time,
                    refine_step,
                )?;

                // Find maximum elevation during this overpass
                let max_elevation =
                    find_max_elevation(&tle, location, start, set_time, refine_step)?;

                let midpoint_time = start + (set_time - start) / 2;

                // Calculate if overpass occurs at night and if satellite is lit
                // Check multiple points: start, midpoint, and end to catch transitions
                let is_night_start = is_night_at_location(location, start)?;
                let is_night_mid = is_night_at_location(location, midpoint_time)?;
                let is_night_end = is_night_at_location(location, set_time)?;
                // Consider it night if any part of the overpass is at night
                let is_night = is_night_start || is_night_mid || is_night_end;

                // For satellite illumination, check at midpoint (most representative)
                let is_lit = is_satellite_lit(&tle, midpoint_time)?;

                overpasses.push(Overpass {
                    start_time: start,
                    end_time: set_time,
                    max_elevation,
                    midpoint_time,
                    is_night,
                    is_lit,
                });
            }
        }

        // Update max elevation if we're in an overpass
        if let Some((_, ref mut max_elev)) = current_overpass {
            if altitude > *max_elev {
                *max_elev = altitude;
            }
        }

        was_above_horizon = is_above_horizon;
        current_time += search_step;
    }

    // Handle overpass that extends beyond end_time
    if let Some((start, max_elev)) = current_overpass {
        // Find when it sets (might be after end_time, but we'll use end_time as limit)
        let set_time = find_set_time(
            &tle,
            location,
            end_time - search_step,
            end_time,
            refine_step,
        )
        .unwrap_or(end_time);

        let max_elevation =
            find_max_elevation(&tle, location, start, set_time.min(end_time), refine_step)
                .unwrap_or(max_elev);

        let midpoint_time = start + (set_time.min(end_time) - start) / 2;

        // Calculate if overpass occurs at night and if satellite is lit
        // Check multiple points: start, midpoint, and end to catch transitions
        let is_night_start = is_night_at_location(location, start)?;
        let is_night_mid = is_night_at_location(location, midpoint_time)?;
        let is_night_end = is_night_at_location(location, set_time.min(end_time))?;
        // Consider it night if any part of the overpass is at night
        let is_night = is_night_start || is_night_mid || is_night_end;

        // For satellite illumination, check at midpoint (most representative)
        let is_lit = is_satellite_lit(&tle, midpoint_time)?;

        overpasses.push(Overpass {
            start_time: start,
            end_time: set_time.min(end_time),
            max_elevation,
            midpoint_time,
            is_night,
            is_lit,
        });
    }

    Ok(overpasses)
}

/// Get satellite positions at regular intervals around a midpoint time.
///
/// # Arguments
/// * `norad_id` - The NORAD ID of the satellite
/// * `location` - Observer's location on Earth
/// * `midpoint_time` - The center time around which to calculate positions
/// * `interval` - Time interval between position points
///
/// # Returns
/// A vector of satellite positions, each containing a timestamp and alt/az coordinates.
#[allow(unused_variables)]
pub fn get_satellite_positions(
    norad_id: u32,
    location: ObserverLocation,
    midpoint_time: DateTime<Utc>,
    interval: Duration,
) -> OverpassPlannerResult<Vec<SatellitePosition>> {
    // TODO: Implement position calculation logic
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iss_overpasses_washington_dc() {
        // Washington DC location: 38.8892°N, 77.1664°W
        let location = ObserverLocation {
            latitude: 38.8892,
            longitude: -77.1664,
            altitude: 0.0, // Sea level
        };

        // ISS NORAD ID
        let iss_norad_id = 25544;

        // Search for next 24 hours
        let time_window = Duration::hours(24);

        println!("Fetching ISS overpasses for Washington DC (38.8892°N, 77.1664°W)...");
        println!("Searching for next 24 hours...\n");

        // Fetch and print the TLE being used
        match fetch_tle(iss_norad_id).await {
            Ok(tle) => {
                println!("Using TLE:\n{}", tle);
                println!();
            }
            Err(e) => {
                println!("Warning: Failed to fetch TLE: {}", e);
                println!();
            }
        }

        match get_overpasses(iss_norad_id, location, time_window).await {
            Ok(overpasses) => {
                println!("Found {} overpass(es):\n", overpasses.len());

                // EST is UTC-5
                let est_offset = FixedOffset::east_opt(-5 * 3600).unwrap();

                for (i, overpass) in overpasses.iter().enumerate() {
                    let start_est = overpass.start_time.with_timezone(&est_offset);
                    let end_est = overpass.end_time.with_timezone(&est_offset);
                    let midpoint_est = overpass.midpoint_time.with_timezone(&est_offset);

                    println!("Overpass #{}:", i + 1);
                    println!("  Start:    {} / {} EST", overpass.start_time, start_est);
                    println!("  End:      {} / {} EST", overpass.end_time, end_est);
                    println!(
                        "  Duration: {:.1} minutes",
                        (overpass.end_time - overpass.start_time).num_seconds() as f64 / 60.0
                    );
                    println!("  Max Elevation: {:.2}°", overpass.max_elevation);
                    println!(
                        "  Midpoint: {} / {} EST",
                        overpass.midpoint_time, midpoint_est
                    );
                    println!();
                }

                // Assert that we found at least some overpasses (ISS typically has multiple per day)
                assert!(
                    !overpasses.is_empty(),
                    "Should find at least one overpass in 24 hours"
                );
            }
            Err(e) => {
                panic!("Failed to get overpasses: {}", e);
            }
        }
    }
}
