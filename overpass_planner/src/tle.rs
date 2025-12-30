//! TLE (Two-Line Element) handling module.
//!
//! This module provides functionality to fetch TLE data from CelesTrak API
//! for use with satellite propagation calculations.
//!
//! The module implements caching to reduce API calls. TLE data for all active
//! satellites is fetched once and cached for 2 hours.

use crate::{OverpassPlannerError, OverpassPlannerResult};
use chrono::{DateTime, Duration, Utc};
use std::path::PathBuf;

const CACHE_FILE_NAME: &str = "tle_cache.txt";
const TIMESTAMP_FILE_NAME: &str = "tle_cache_timestamp.txt";
const CACHE_DURATION_HOURS: i64 = 2;

/// Gets the cache directory path for storing TLE data.
fn get_cache_dir() -> OverpassPlannerResult<PathBuf> {
    let cache_dir = dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
        .ok_or_else(|| {
            OverpassPlannerError::NetworkError("Could not determine cache directory".to_string())
        })?;

    let tle_cache_dir = cache_dir.join("overpass_planner");
    Ok(tle_cache_dir)
}

/// Gets the path to the TLE cache file.
fn get_cache_file_path() -> OverpassPlannerResult<PathBuf> {
    let cache_dir = get_cache_dir()?;
    Ok(cache_dir.join(CACHE_FILE_NAME))
}

/// Gets the path to the timestamp file.
fn get_timestamp_file_path() -> OverpassPlannerResult<PathBuf> {
    let cache_dir = get_cache_dir()?;
    Ok(cache_dir.join(TIMESTAMP_FILE_NAME))
}

/// Checks if the cache is valid (less than 2 hours old).
async fn is_cache_valid() -> bool {
    let timestamp_path = match get_timestamp_file_path() {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !timestamp_path.exists() {
        return false;
    }

    let timestamp_str = match tokio::fs::read_to_string(&timestamp_path).await {
        Ok(s) => s,
        Err(_) => return false,
    };

    let timestamp = match timestamp_str.trim().parse::<i64>() {
        Ok(t) => t,
        Err(_) => return false,
    };

    let cache_time = DateTime::<Utc>::from_timestamp(timestamp, 0);
    let cache_time = match cache_time {
        Some(t) => t,
        None => return false,
    };

    let now = Utc::now();
    let age = now.signed_duration_since(cache_time);

    age < Duration::hours(CACHE_DURATION_HOURS)
}

/// Writes the cache timestamp to disk.
async fn write_cache_timestamp() -> OverpassPlannerResult<()> {
    let cache_dir = get_cache_dir()?;
    tokio::fs::create_dir_all(&cache_dir).await.map_err(|e| {
        OverpassPlannerError::NetworkError(format!("Failed to create cache directory: {e}"))
    })?;

    let timestamp_path = get_timestamp_file_path()?;
    let timestamp = Utc::now().timestamp();
    tokio::fs::write(&timestamp_path, timestamp.to_string())
        .await
        .map_err(|e| {
            OverpassPlannerError::NetworkError(format!("Failed to write timestamp file: {e}"))
        })?;

    Ok(())
}

/// Fetches all active satellites from CelesTrak API.
async fn fetch_all_active_satellites() -> OverpassPlannerResult<String> {
    let url = "https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=TLE";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            OverpassPlannerError::NetworkError(format!("Failed to create HTTP client: {e}"))
        })?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| OverpassPlannerError::NetworkError(format!("HTTP request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(OverpassPlannerError::NetworkError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let text = response
        .text()
        .await
        .map_err(|e| OverpassPlannerError::NetworkError(format!("Failed to read response: {e}")))?;

    Ok(text)
}

/// Updates the cache by fetching fresh data from the API.
async fn update_cache() -> OverpassPlannerResult<()> {
    let tle_data = fetch_all_active_satellites().await?;

    // Ensure cache directory exists before writing
    let cache_dir = get_cache_dir()?;
    tokio::fs::create_dir_all(&cache_dir).await.map_err(|e| {
        OverpassPlannerError::NetworkError(format!("Failed to create cache directory: {e}"))
    })?;

    let cache_file_path = get_cache_file_path()?;
    tokio::fs::write(&cache_file_path, &tle_data)
        .await
        .map_err(|e| {
            OverpassPlannerError::NetworkError(format!("Failed to write cache file: {e}"))
        })?;

    write_cache_timestamp().await?;

    Ok(())
}

/// Reads the cached TLE data from disk.
async fn read_cache() -> OverpassPlannerResult<String> {
    let cache_file_path = get_cache_file_path()?;
    let tle_data = tokio::fs::read_to_string(&cache_file_path)
        .await
        .map_err(|e| {
            OverpassPlannerError::NetworkError(format!("Failed to read cache file: {e}"))
        })?;

    Ok(tle_data)
}

/// Parses a specific TLE from cached data by NORAD ID.
fn parse_tle_from_cache(cache_data: &str, norad_id: u32) -> OverpassPlannerResult<String> {
    let lines: Vec<&str> = cache_data.lines().collect();
    let norad_id_str = norad_id.to_string();

    // Search for the TLE with matching NORAD ID
    // TLE format: name line, line 1 (starts with "1 "), line 2 (starts with "2 ")
    // There may be blank lines between entries
    let mut i = 0;
    while i < lines.len() {
        // Check if this is a TLE line 1
        let line1_raw = lines[i].trim();
        if line1_raw.starts_with("1 ") && line1_raw.len() > 7 {
            // Extract NORAD ID from line 1 (positions 2-7 after "1 ")
            // Format: "1 25544U ..." - NORAD ID is at positions 2-7
            let id_str = line1_raw[2..7].trim();
            if id_str == norad_id_str {
                // Found matching line 1, now find the name line and line 2
                // Look backwards for the name line (skip blank lines)
                let mut name_idx = i;
                while name_idx > 0 {
                    name_idx -= 1;
                    let candidate = lines[name_idx].trim();
                    if !candidate.is_empty()
                        && !candidate.starts_with("1 ")
                        && !candidate.starts_with("2 ")
                    {
                        // Found the name line
                        // Check for line 2
                        if i + 1 < lines.len() {
                            let line2_raw = lines[i + 1].trim();
                            if line2_raw.starts_with("2 ") && line2_raw.len() > 7 {
                                let line2_id = line2_raw[2..7].trim();
                                if line2_id == norad_id_str {
                                    // Found complete TLE
                                    let name_line = candidate;
                                    let line1 = line1_raw;
                                    let line2 = line2_raw;
                                    let tle = format!("{}\n{}\n{}", name_line, line1, line2);
                                    validate_tle(&tle)?;
                                    return Ok(tle);
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        i += 1;
    }

    Err(OverpassPlannerError::ParseError(format!(
        "TLE for NORAD ID {} not found in cache",
        norad_id
    )))
}

/// Fetches the TLE for a satellite from CelesTrak API with caching.
///
/// This function checks the cache first. If the cache is valid (less than 2 hours old),
/// it returns the TLE from cache. Otherwise, it fetches fresh data from the API.
///
/// # Arguments
/// * `norad_id` - The NORAD catalog number (NORAD ID) of the satellite
///
/// # Returns
/// A string containing the TLE data (name, line 1, and line 2).
///
/// # Errors
/// Returns `OverpassPlannerError` if:
/// - Network request fails
/// - HTTP response is not successful
/// - TLE data cannot be parsed from the response
/// - Cache operations fail
///
/// # Example
/// ```no_run
/// use overpass_planner::tle::fetch_tle;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tle = fetch_tle(25544).await?; // ISS NORAD ID
/// println!("TLE:\n{}", tle);
/// # Ok(())
/// # }
/// ```
pub async fn fetch_tle(norad_id: u32) -> OverpassPlannerResult<String> {
    // Check if cache is valid
    if is_cache_valid().await {
        // Try to read from cache
        match read_cache().await {
            Ok(cache_data) => {
                match parse_tle_from_cache(&cache_data, norad_id) {
                    Ok(tle) => return Ok(tle),
                    Err(_) => {
                        // TLE not found in cache, fall through to update cache
                    }
                }
            }
            Err(_) => {
                // Cache read failed, fall through to update cache
            }
        }
    }

    // Cache is invalid or TLE not found, update cache
    update_cache().await?;

    // Read from updated cache
    let cache_data = read_cache().await?;
    parse_tle_from_cache(&cache_data, norad_id)
}

/// Validates that the response contains valid TLE data.
///
/// Expects the format:
/// ```
/// SATELLITE NAME
/// 1 25544U 98067A   12345.67890123  .00001234  00000-0  12345-4 0  1234
/// 2 25544  51.6450 123.4567 0001234 234.5678 123.4567 15.12345678 12345
/// ```
fn validate_tle(text: &str) -> OverpassPlannerResult<()> {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.len() < 3 {
        return Err(OverpassPlannerError::ParseError(format!(
            "Expected at least 3 lines in TLE response, got {}",
            lines.len()
        )));
    }

    // Check that we have TLE lines (they start with "1 " and "2 ")
    let mut has_line1 = false;
    let mut has_line2 = false;

    for line in lines {
        if line.starts_with("1 ") {
            has_line1 = true;
        } else if line.starts_with("2 ") {
            has_line2 = true;
        }
    }

    if !has_line1 {
        return Err(OverpassPlannerError::ParseError(
            "TLE line 1 not found in response".to_string(),
        ));
    }

    if !has_line2 {
        return Err(OverpassPlannerError::ParseError(
            "TLE line 2 not found in response".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tle() {
        let tle_text = r#"ISS (ZARYA)
1 25544U 98067A   12345.67890123  .00001234  00000-0  12345-4 0  1234
2 25544  51.6450 123.4567 0001234 234.5678 123.4567 15.12345678 12345"#;

        assert!(validate_tle(tle_text).is_ok());
    }

    #[test]
    fn test_validate_tle_invalid() {
        let invalid_text = "Not a TLE";
        assert!(validate_tle(invalid_text).is_err());
    }

    #[test]
    fn test_parse_tle_from_cache() {
        let cache_data = r#"ISS (ZARYA)
1 25544U 98067A   12345.67890123  .00001234  00000-0  12345-4 0  1234
2 25544  51.6450 123.4567 0001234 234.5678 123.4567 15.12345678 12345
ANOTHER SATELLITE
1 25551U 98067B   12345.67890123  .00001234  00000-0  12345-4 0  1234
2 25551  51.6450 123.4567 0001234 234.5678 123.4567 15.12345678 12345"#;

        let result = parse_tle_from_cache(cache_data, 25544);
        assert!(result.is_ok());
        let tle = result.unwrap();
        assert!(tle.contains("ISS (ZARYA)"));
        assert!(tle.contains("1 25544U"));
        assert!(tle.contains("2 25544"));

        let result2 = parse_tle_from_cache(cache_data, 25551);
        assert!(result2.is_ok());
        let tle2 = result2.unwrap();
        assert!(tle2.contains("ANOTHER SATELLITE"));
        assert!(tle2.contains("1 25551U"));
        assert!(tle2.contains("2 25551"));

        let result3 = parse_tle_from_cache(cache_data, 99999);
        assert!(result3.is_err());
    }

    #[tokio::test]
    async fn test_fetch_tle_api() {
        // Test with ISS NORAD ID (25544)
        let result = fetch_tle(25544).await;

        assert!(
            result.is_ok(),
            "Failed to fetch TLE from API: {:?}",
            result.as_ref().err()
        );

        let tle = result.unwrap();
        assert!(!tle.is_empty(), "TLE string should not be empty");

        // Verify it contains TLE lines
        assert!(tle.contains("1 "), "TLE should contain line 1");
        assert!(tle.contains("2 "), "TLE should contain line 2");

        // Print the TLE for manual inspection
        println!("Fetched TLE:\n{tle}");
    }
}
