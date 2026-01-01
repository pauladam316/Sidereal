use crate::colors;
use bevy_egui::egui;

/// A custom button widget for the planetarium UI.
/// Styled to match the text input with rounded corners, custom background color, and border.
/// On hover, the text and border change to the accent yellow color.
///
/// Usage:
/// ```rust
/// if planetarium_button(ui, "Click Me", 100.0, 24.0).clicked() {
///     // Handle click
/// }
/// ```
pub fn planetarium_button(
    ui: &mut egui::Ui,
    text: impl Into<egui::WidgetText>,
    width: f32,
    height: f32,
) -> egui::Response {
    let desired = egui::vec2(width, height);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

    // Check if hovering
    let is_hovered = response.hovered();

    // Choose colors based on hover state
    let border_color = if is_hovered {
        colors::egui::ACCENT_YELLOW
    } else {
        egui::Color32::from_rgb(179, 179, 179)
    };

    let bg_color = egui::Color32::from_rgb(77, 77, 77);

    // Paint border + bg
    ui.painter().rect_filled(rect, 4.0, border_color);

    let inner_rect = rect.shrink(1.0);
    ui.painter().rect_filled(inner_rect, 3.5, bg_color);

    // Put the button text inside the rect, with padding
    let pad_x = 6.0;
    let pad_y = 2.0; // Small vertical padding
    let text_rect = inner_rect.shrink2(egui::vec2(pad_x, pad_y));

    // Draw the button text - use a non-interactive label so it doesn't interfere with clicks
    ui.scope(|ui| {
        // Set font size to 12pt for button text
        let style = ui.style_mut();
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        );

        // Use vertical centering layout to center the text, then shift up 2pt
        let text_color = if is_hovered {
            colors::egui::ACCENT_YELLOW
        } else {
            colors::egui::WINDOW_TITLE_COLOR
        };

        // Convert WidgetText to galley and draw it using painter
        // This avoids creating a separate interactive widget that might interfere with clicks
        let text_widget: egui::WidgetText = text.into();
        let galley = text_widget.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            text_rect.width(),
            egui::TextStyle::Button,
        );

        // Center the text vertically and horizontally, shift up 2pt
        let text_pos =
            text_rect.center() - egui::vec2(galley.size().x / 2.0, galley.size().y / 2.0 + 2.0);
        ui.painter().galley(text_pos, galley, text_color);
    });

    // Set cursor to pointing hand when hovering
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
