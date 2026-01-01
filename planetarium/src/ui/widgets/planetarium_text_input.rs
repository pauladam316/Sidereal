use crate::colors;
use bevy_egui::egui;

/// A custom text input widget for the planetarium UI.
/// Works like a normal egui text input but allows for custom styling.
/// Uses a fixed width and height specified by the caller.
/// Styled with rounded corners, custom background color, and border.
///
/// Usage:
/// ```rust
/// planetarium_text_input(ui, &mut my_string, 200.0, 24.0); // 200px width, 24px height
/// ```
pub fn planetarium_text_input(
    ui: &mut egui::Ui,
    text: &mut String,
    width: f32,
    height: f32,
) -> egui::Response {
    let desired = egui::vec2(width, height);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover()); // reserve space in layout

    // Paint border + bg
    let border_color = egui::Color32::from_rgb(179, 179, 179);
    let bg_color = egui::Color32::from_rgb(77, 77, 77);

    ui.painter().rect_filled(rect, 4.0, border_color);

    let inner_rect = rect.shrink(1.0);
    ui.painter().rect_filled(inner_rect, 3.5, bg_color);

    // Put the TextEdit inside the rect, with padding
    let pad_x = 6.0;
    let pad_y = 2.0; // Small vertical padding
    let edit_rect = inner_rect.shrink2(egui::vec2(pad_x, pad_y));

    let resp = ui.scope(|ui| {
        // Set font size to 12pt for this text input
        let style = ui.style_mut();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        );

        // Use vertical centering layout to center the text, then shift up 2pt
        let text_edit_response = ui.allocate_ui_at_rect(edit_rect, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.set_height(edit_rect.height());
                ui.vertical_centered(|ui| {
                    ui.add_space(-2.0); // Shift text up by 2pt
                    ui.add(
                        egui::TextEdit::singleline(text)
                            .frame(false) // we draw bg/border ourselves
                            .desired_width(f32::INFINITY)
                            .text_color(colors::egui::WINDOW_TITLE_COLOR),
                    )
                })
            })
            .inner
        });
        text_edit_response.inner.inner
    });

    resp.inner
}
