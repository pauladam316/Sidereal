use bevy_egui::egui;

/// Creates a styled container frame with rounded corners and soft drop shadow.
/// This container is designed to wrap content sections with consistent styling.
///
/// Features:
/// - Rounded corners (8.0 radius)
/// - Background color: RGB(0.25, 0.25, 0.25)
/// - Soft drop shadow (2px downward offset, 4px blur)
/// - 8pt inner padding
///
/// Usage:
/// ```rust
/// content_container_frame().show(ui, |ui| {
///     // Your content here - container will stretch full width and have 8pt padding
/// });
/// ```
pub fn content_container_frame() -> egui::Frame {
    let mut frame = egui::Frame::new()
        .fill(egui::Color32::from_rgb(64, 64, 64)) // RGB(0.25, 0.25, 0.25) = 0.25 * 255 = 64
        .corner_radius(8.0); // Rounded corners

    // Set 8pt inner padding
    frame.inner_margin = egui::Margin {
        left: 8,
        right: 8,
        top: 8,
        bottom: 8,
    };

    // Set outer margin to 4pt top/bottom for 8pt total spacing between containers
    // (4pt from this container + 4pt from next container = 8pt total)
    // Left/right stay at 0 since containers should stretch to window edges
    frame.outer_margin = egui::Margin {
        left: 0,
        right: 0,
        top: 4,
        bottom: 4,
    };

    // Add soft drop shadow
    frame.shadow = egui::Shadow {
        offset: [0i8, 2i8], // Shadow offset downward
        blur: 4u8,          // Soft blur
        spread: 0u8,        // No spread
        color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 60), // Soft black shadow
    };

    frame
}
