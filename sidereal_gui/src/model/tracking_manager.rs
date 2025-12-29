#[allow(dead_code)]
enum TrackingMode {
    Generic,
    Planet,
    Lunar,
    Solar,
    Satellite,
}

#[allow(dead_code)]
enum TrackingState {
    Idle,
    Tracking,
}
#[allow(dead_code)]
struct TrackingManager {
    state: TrackingState,
    mode: TrackingMode,
}
