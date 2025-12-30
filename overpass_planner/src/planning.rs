//! Overpass planning module.
//!
//! This module provides functionality to calculate satellite positions
//! and plan overpasses using SGP4 propagation.

use crate::{OverpassPlannerError, OverpassPlannerResult};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use satkit::{frametransform, types::Vec3, ITRFCoord, Instant};
use sgp4::{Elements, Prediction};

/// Observer location on Earth.
#[derive(Debug, Clone, Copy)]
pub struct ObserverLocation {
    /// Latitude in degrees (-90 to 90)
    pub latitude: f64,
    /// Longitude in degrees (-180 to 180)
    pub longitude: f64,
    /// Altitude in meters above sea level
    pub altitude: f64,
}

/// Calculates the altitude and azimuth of a satellite at a given time.
///
/// # Arguments
/// * `tle` - The TLE string (containing name, line 1, and line 2)
/// * `location` - Observer's location on Earth
/// * `timestamp` - UTC timestamp for the calculation
///
/// # Returns
/// A tuple containing (altitude_degrees, azimuth_degrees) where:
/// - altitude: 0-90 degrees (0 = horizon, 90 = zenith)
/// - azimuth: 0-360 degrees (0 = North, 90 = East, 180 = South, 270 = West)
///
/// # Errors
/// Returns `OverpassPlannerError` if:
/// - TLE parsing fails
/// - Satellite propagation fails
/// - Coordinate conversion fails
pub fn calculate_alt_az(
    tle: &str,
    location: ObserverLocation,
    timestamp: DateTime<Utc>,
) -> OverpassPlannerResult<(f64, f64)> {
    // Parse TLE string into lines
    let lines: Vec<&str> = tle
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.len() < 3 {
        return Err(OverpassPlannerError::ParseError(
            "TLE must contain at least 3 lines (name, line1, line2)".to_string(),
        ));
    }

    // Find TLE lines (they start with "1 " and "2 ")
    let mut line1 = None;
    let mut line2 = None;

    for line in &lines {
        if line.starts_with("1 ") {
            line1 = Some(*line);
        } else if line.starts_with("2 ") {
            line2 = Some(*line);
        }
    }

    let line1 = line1
        .ok_or_else(|| OverpassPlannerError::ParseError("TLE line 1 not found".to_string()))?;

    let line2 = line2
        .ok_or_else(|| OverpassPlannerError::ParseError("TLE line 2 not found".to_string()))?;

    // Parse TLE using sgp4
    let elements = Elements::from_tle(None, line1.as_bytes(), line2.as_bytes())
        .map_err(|e| OverpassPlannerError::TLEError(format!("Failed to parse TLE: {e}")))?;

    // Create constants for propagation
    let constants = sgp4::Constants::from_elements(&elements).map_err(|e| {
        OverpassPlannerError::CalculationError(format!("Failed to create constants: {e}"))
    })?;

    // Calculate minutes since TLE epoch (with fractional precision)
    let tle_epoch = elements.datetime.and_utc();
    let duration = timestamp.signed_duration_since(tle_epoch);
    let minutes_since_epoch = duration.num_seconds() as f64 / 60.0;

    // Propagate satellite position
    let prediction = constants
        .propagate(minutes_since_epoch)
        .map_err(|e| OverpassPlannerError::CalculationError(format!("Propagation failed: {e}")))?;

    // Convert satellite position (in ECI/TEME frame) to alt/az
    let (altitude, azimuth) = eci_to_alt_az(prediction, location, timestamp)?;

    Ok((altitude, azimuth))
}

/// Converts satellite position from ECI (Earth-Centered Inertial) coordinates to alt/az.
///
/// This function performs the coordinate transformation from ECI to topocentric
/// (observer-centered) coordinates and then calculates altitude and azimuth.
fn eci_to_alt_az(
    prediction: Prediction,
    location: ObserverLocation,
    timestamp: DateTime<Utc>,
) -> OverpassPlannerResult<(f64, f64)> {
    // Get satellite position in TEME frame (km) - position is [f64; 3]
    let sat_pos = prediction.position;

    // Convert chrono DateTime to satkit Instant
    let naive = timestamp.naive_utc();
    let instant = Instant::from_datetime(
        naive.year(),
        naive.month() as i32,
        naive.day() as i32,
        naive.hour() as i32,
        naive.minute() as i32,
        naive.second() as f64 + naive.nanosecond() as f64 / 1e9,
    );

    // Create observer location as ITRFCoord
    // satkit uses meters for altitude
    let observer =
        ITRFCoord::from_geodetic_deg(location.latitude, location.longitude, location.altitude);

    // SGP4 returns positions in km, convert to meters for satkit
    // Create position vector in TEME frame (in meters)
    let pos_teme_m = Vec3::new(
        sat_pos[0] * 1000.0,
        sat_pos[1] * 1000.0,
        sat_pos[2] * 1000.0,
    );

    // Convert TEME to ITRF using satkit's frame transformation
    // This handles all the Earth rotation automatically
    // Note: qteme2itrf requires EOP data files - if missing, it will panic
    // Check if EOP data is available before calling
    use satkit::earth_orientation_params;
    if earth_orientation_params::get(&instant).is_none() {
        return Err(OverpassPlannerError::CalculationError(
            "Earth Orientation Parameters (EOP) data not available. Please run satkit::utils::update_datafiles() first.".to_string(),
        ));
    }
    let q_teme2itrf = frametransform::qteme2itrf(&instant);

    // Apply quaternion rotation to convert TEME to ITRF
    // Use to_rotation_matrix() to ensure proper matrix multiplication
    let rot_matrix = q_teme2itrf.to_rotation_matrix();
    let pos_itrf_m = rot_matrix * pos_teme_m;

    // Create ITRFCoord from the converted position (in meters)
    let sat_itrf = ITRFCoord::from_slice(pos_itrf_m.as_slice()).map_err(|e| {
        OverpassPlannerError::CalculationError(format!("Failed to create ITRFCoord: {e}"))
    })?;

    // Compute observer→satellite vector in ITRF (ECEF) frame
    // This is the relative vector from observer to satellite
    let rel_itrf = sat_itrf.itrf - observer.itrf;

    // Convert relative vector to ENU frame at observer's location
    // Use observer's ENU frame rotation (not satellite's!)
    let q_enu2itrf_obs = observer.q_enu2itrf();
    let enu = q_enu2itrf_obs.conjugate() * rel_itrf;

    // ENU components: [0] = East, [1] = North, [2] = Up (meters)
    // Now 'up' can be negative when satellite is below horizon
    let east = enu[0];
    let north = enu[1];
    let up = enu[2];

    // Calculate horizontal range (distance in horizontal plane)
    let horizontal_range = (east * east + north * north).sqrt();

    // Calculate total range (distance from observer to satellite)
    let range = (horizontal_range * horizontal_range + up * up).sqrt();

    if range < 1e-6 {
        return Err(OverpassPlannerError::CalculationError(
            "Satellite is at observer location".to_string(),
        ));
    }

    // Calculate altitude (elevation angle) - angle above horizon
    // Positive when satellite is above horizon, negative when below
    // Use atan2(up, horizontal_range) for proper quadrant handling
    let altitude = up.atan2(horizontal_range).to_degrees();

    // Calculate azimuth (0 = North, 90 = East, 180 = South, 270 = West)
    // Azimuth is measured clockwise from North
    // atan2(east, north) gives angle from North axis, increasing clockwise
    let azimuth = east.atan2(north).to_degrees();

    // Normalize azimuth to 0-360
    let azimuth = if azimuth < 0.0 {
        azimuth + 360.0
    } else if azimuth >= 360.0 {
        azimuth - 360.0
    } else {
        azimuth
    };

    Ok((altitude, azimuth))
}

/// Find the exact time when satellite rises above horizon using binary search.
pub(crate) fn find_rise_time(
    tle: &str,
    location: ObserverLocation,
    time_before: DateTime<Utc>,
    time_after: DateTime<Utc>,
    step: Duration,
) -> OverpassPlannerResult<DateTime<Utc>> {
    let mut low = time_before;
    let mut high = time_after;

    // Binary search for rise time
    while (high - low).num_seconds() > step.num_seconds() {
        let mid = low + (high - low) / 2;
        let (altitude, _) = calculate_alt_az(tle, location, mid)?;

        if altitude > 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }

    Ok(high)
}

/// Find the exact time when satellite sets below horizon using binary search.
pub(crate) fn find_set_time(
    tle: &str,
    location: ObserverLocation,
    time_before: DateTime<Utc>,
    time_after: DateTime<Utc>,
    step: Duration,
) -> OverpassPlannerResult<DateTime<Utc>> {
    let mut low = time_before;
    let mut high = time_after;

    // Binary search for set time
    while (high - low).num_seconds() > step.num_seconds() {
        let mid = low + (high - low) / 2;
        let (altitude, _) = calculate_alt_az(tle, location, mid)?;

        if altitude > 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }

    Ok(low)
}

/// Find the maximum elevation during an overpass using golden section search.
pub(crate) fn find_max_elevation(
    tle: &str,
    location: ObserverLocation,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    step: Duration,
) -> OverpassPlannerResult<f64> {
    // First do a coarse search to find approximate peak
    let mut max_elevation = 0.0;
    let mut max_time = start_time;
    let mut current_time = start_time;

    while current_time <= end_time {
        let (altitude, _) = calculate_alt_az(tle, location, current_time)?;
        if altitude > max_elevation {
            max_elevation = altitude;
            max_time = current_time;
        }
        current_time += step;
    }

    // Refine around the peak using golden section search
    // Find the time window around max_time (1 minute total, 30 seconds each side)
    let half_window = Duration::seconds(30);
    let refine_start = (max_time - half_window).max(start_time);
    let refine_end = (max_time + half_window).min(end_time);

    // Golden section search for maximum
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // Golden ratio
    let mut a = refine_start;
    let mut b = refine_end;

    // Convert to seconds for easier calculation
    let total_seconds = (b - a).num_seconds() as f64;
    if total_seconds < 1.0 {
        // Window too small, return coarse result
        return Ok(max_elevation);
    }

    let mut c = a + Duration::seconds((total_seconds / phi) as i64);
    let mut d = b - Duration::seconds((total_seconds / phi) as i64);

    // Iterate until convergence (with max iterations to prevent infinite loop)
    let max_iterations = 50;
    let mut iterations = 0;
    while (c - d).num_seconds().abs() > 1 && iterations < max_iterations {
        iterations += 1;
        let (alt_c, _) = calculate_alt_az(tle, location, c)?;
        let (alt_d, _) = calculate_alt_az(tle, location, d)?;

        if alt_c > alt_d {
            b = d;
            d = c;
            let total_seconds = (b - a).num_seconds() as f64;
            if total_seconds < 1.0 {
                break;
            }
            c = a + Duration::seconds((total_seconds / phi) as i64);
            max_elevation = alt_c.max(max_elevation);
        } else {
            a = c;
            c = d;
            let total_seconds = (b - a).num_seconds() as f64;
            if total_seconds < 1.0 {
                break;
            }
            d = b - Duration::seconds((total_seconds / phi) as i64);
            max_elevation = alt_d.max(max_elevation);
        }
    }

    // Final check at midpoint
    let midpoint = a + (b - a) / 2;
    let (alt_mid, _) = calculate_alt_az(tle, location, midpoint)?;
    Ok(alt_mid.max(max_elevation))
}
