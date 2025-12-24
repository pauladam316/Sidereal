use indi::client::active_device::ActiveDevice;

/// Camera-specific helper functions
///
/// This module contains functions for interacting with INDI camera devices.
/// Add camera-specific telemetry and command functions here as needed.
///
/// **Device handling pattern:**
/// - All control functions should return `SiderealResult` and handle missing devices gracefully
/// - Error messages should be user-friendly: "Camera device not available. Please ensure the device is connected to the INDI server."
/// - Watchers (if added) should follow the same pattern as mount and telescope_controller watchers:
///   - Wait and periodically check if device is not available
///   - Re-discover devices periodically
///   - Distinguish between server disconnection and device disappearance
///   - Never fail or error out when device is missing
///
/// Get the active camera device if available
#[allow(dead_code)]
pub fn get_camera() -> Option<ActiveDevice> {
    // Note: This is a blocking read, but should be fine for occasional access
    // For async access, use CONNECTED_DEVICES.read().await in async contexts
    None // TODO: Implement when needed
}

