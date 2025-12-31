use crate::colors;
use bevy_egui::egui;

/// Creates a styled button with hover effects using the accent yellow color.
/// Returns the button response for further interaction.
pub fn planetarium_button(
    ui: &mut egui::Ui,
    hover_id: egui::Id,
    text: impl Into<egui::WidgetText>,
    is_highlighted: bool,
) -> egui::Response {
    let (was_hovered, widget_text) = {
        let ctx = ui.ctx();
        let default_text_color = ctx.style().visuals.text_color();
        let was_hovered = ctx.data(|data| data.get_temp::<bool>(hover_id).unwrap_or(false));

        let text_color = if was_hovered || is_highlighted {
            colors::egui::ACCENT_YELLOW
        } else {
            default_text_color
        };

        let widget_text = text.into().color(text_color);
        (was_hovered, widget_text)
    };

    let button = egui::Button::new(widget_text)
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
        .frame(false);

    let mut response = ui.add(button);

    // Set cursor to pointing hand when hovering
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

    // Track hover state for next frame
    let is_hovered = response.hovered();
    ui.ctx().data_mut(|data| {
        data.insert_temp(hover_id, is_hovered);
    });

    // Request repaint if hover changed
    if is_hovered != was_hovered {
        ui.ctx().request_repaint();
    }

    response
}

/// Creates a menu button that opens a popup when clicked.
/// Uses planetarium_button internally for consistent styling.
pub fn planetarium_menu_button(
    ui: &mut egui::Ui,
    menu_id: egui::Id,
    hover_id: egui::Id,
    text: impl Into<egui::WidgetText>,
    add_contents: impl FnOnce(&mut egui::Ui, egui::Id),
) {
    let ctx = ui.ctx();
    let is_popup_open = egui::Popup::is_id_open(ctx, menu_id);

    let response = planetarium_button(ui, hover_id, text, is_popup_open);

    // This is the "real" popup framework (keeps itself alive, closes on click-outside, etc.)
    egui::Popup::menu(&response)
        .id(menu_id) // make it use YOUR id (important if you want to close it explicitly)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .frame(egui::Frame::popup(ui.style()))
        .show(|ui| {
            ui.set_min_width(100.0);
            add_contents(ui, menu_id);
        });
}

