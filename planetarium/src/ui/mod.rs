mod satellite_window;
mod widgets;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use widgets::{planetarium_button, planetarium_menu_button};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    TrackSatellite,
    TrackDSO,
    TrackPlanet,
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub satellite_window_open: bool,
}

pub struct MenuPlugin;

#[derive(Resource, Default)]
struct FontsConfigured(bool);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<satellite_window::SatelliteSearchState>()
            .init_resource::<satellite_window::SearchResultChannel>()
            .init_resource::<FontsConfigured>()
            .add_systems(Update, (setup_egui_fonts, render_menu_bar).chain())
            .add_systems(Update, satellite_window::render_satellite_window);
    }
}

fn setup_egui_fonts(
    mut fonts_configured: ResMut<FontsConfigured>,
    mut camera_query: Query<&mut EguiContext, With<Camera3d>>,
) {
    if fonts_configured.0 {
        return; // Already configured
    }

    if let Ok(mut egui_context) = camera_query.single_mut() {
        let ctx = egui_context.get_mut();
        configure_segoe_ui_font(ctx);
        fonts_configured.0 = true;
    }
}

fn configure_segoe_ui_font(ctx: &egui::Context) {
    use egui::{FontFamily, TextStyle};
    use std::sync::Arc;

    let mut fonts = egui::FontDefinitions::default();

    // Load Segoe UI from assets folder (embedded in binary)
    let segoe_ui_font_data = include_bytes!("../../assets/segoeui.ttf");
    fonts.font_data.insert(
        "segoe_ui".to_owned(),
        Arc::new(egui::FontData::from_static(segoe_ui_font_data)),
    );

    // Set Segoe UI as the primary font for proportional text
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "segoe_ui".to_owned());

    ctx.set_fonts(fonts);

    // Increase font sizes by 2 points
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        TextStyle::Body,
        egui::FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        egui::FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        egui::FontId::new(11.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        egui::FontId::new(21.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        egui::FontId::new(15.0, FontFamily::Monospace),
    );
    ctx.set_style(style);
}

fn render_menu_bar(
    mut menu_state: ResMut<MenuState>,
    mut camera_query: Query<&mut EguiContext, With<Camera3d>>,
) {
    // Query for the camera with EguiContext directly
    if let Ok(mut egui_context) = camera_query.single_mut() {
        let ctx = egui_context.get_mut();
        render_ui(ctx, &mut menu_state);
    }
}

// In your menu items, close like this:
fn close_popup(ui: &mut egui::Ui, id: egui::Id) {
    egui::Popup::close_id(ui.ctx(), id);
}
fn render_ui(ctx: &mut egui::Context, menu_state: &mut ResMut<MenuState>) {
    let menu_id = egui::Id::new("track_menu");
    let hover_id = egui::Id::new("track_button_hover");

    egui::TopBottomPanel::top("menu_bar")
        .exact_height(28.0)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // Create styled menu button with custom button appearance
                planetarium_menu_button(ui, menu_id, hover_id, "Track", |ui, menu_id| {
                    // Satellite button
                    let satellite_hover_id = egui::Id::new("satellite_button_hover");
                    if planetarium_button(ui, satellite_hover_id, "Satellite", false).clicked() {
                        menu_state.satellite_window_open = true;
                        egui::Popup::close_id(ui.ctx(), menu_id);
                    }

                    // DSO button
                    let dso_hover_id = egui::Id::new("dso_button_hover");
                    if planetarium_button(ui, dso_hover_id, "DSO", false).clicked() {
                        // TODO: Implement DSO tracking
                        egui::Popup::close_id(ui.ctx(), menu_id);
                    }

                    // Planet button
                    let planet_hover_id = egui::Id::new("planet_button_hover");
                    if planetarium_button(ui, planet_hover_id, "Planet", false).clicked() {
                        // TODO: Implement planet tracking
                        egui::Popup::close_id(ui.ctx(), menu_id);
                    }
                });
            });
        });
}
