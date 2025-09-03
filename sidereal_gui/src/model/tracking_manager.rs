enum TrackingMode {
    Generic,
    Planet,
    Lunar,
    Solar,
    Satellite,
}

enum TrackingState {
    Idle,
    Tracking,
}
struct TrackingManager {
    state: TrackingState,
    mode: TrackingMode,
}
