use bevy::prelude::*;

/// Color palette for the planetarium UI
/// Matches the sidereal_gui color scheme for consistency
// Menu bar colors
pub const MENU_BAR_BACKGROUND: Color = Color::srgb(0.10, 0.10, 0.12);
pub const MENU_DROPDOWN_BACKGROUND: Color = Color::srgb(0.14, 0.14, 0.17);
pub const MENU_DROPDOWN_BORDER: Color = Color::srgb(0.25, 0.25, 0.30);

// Text colors
pub const TEXT_COLOR_NORMAL: Color = Color::srgb(0.82, 0.82, 0.85);
pub const TEXT_COLOR_SECONDARY: Color = Color::srgb(0.78, 0.78, 0.82);
pub const TEXT_COLOR_HIGHLIGHT: Color = Color::srgb(0.918, 0.878, 0.349); // Accent yellow
pub const TEXT_COLOR_BRIGHT: Color = Color::srgb(0.88, 0.88, 0.90);

// Modal window colors
pub const MODAL_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);
pub const MODAL_BACKGROUND: Color = Color::srgb(0.12, 0.12, 0.14);
pub const MODAL_BORDER: Color = Color::srgb(0.28, 0.28, 0.32);
pub const MODAL_TITLE_BAR: Color = Color::srgb(0.16, 0.16, 0.19);
pub const MODAL_TITLE_BORDER: Color = Color::srgb(0.22, 0.22, 0.26);

// Button colors
pub const CLOSE_BUTTON: Color = Color::srgb(0.65, 0.18, 0.18);

// Accent colors (matching sidereal_gui)
pub const ACCENT_YELLOW: Color = Color::srgb(0.918, 0.878, 0.349);
pub const GREEN_TEXT: Color = Color::srgb(0.431, 0.969, 0.431);
pub const RED_TEXT: Color = Color::srgb(0.969, 0.431, 0.431);
pub const AMBER_TEXT: Color = Color::srgb(0.969, 0.824, 0.431);

// Target marker colors (3D scene)
pub const TRACKING_TARGET_COLOR: Color = Color::srgb(0.918, 0.878, 0.349); // Accent yellow
pub const MOUNT_TARGET_COLOR: Color = Color::srgb(0.475, 0.941, 0.475); // Green
