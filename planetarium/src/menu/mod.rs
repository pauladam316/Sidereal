mod satellite_window;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

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

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .init_resource::<satellite_window::SatelliteSearchState>()
            .init_resource::<satellite_window::SearchResultChannel>()
            .add_systems(Update, render_menu_bar)
            .add_systems(Update, satellite_window::render_satellite_window);
    }
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

fn render_ui(ctx: &mut egui::Context, menu_state: &mut ResMut<MenuState>) {
    egui::TopBottomPanel::top("menu_bar")
        .exact_height(28.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Track", |ui| {
                    if ui.button("Satellite").clicked() {
                        menu_state.satellite_window_open = true;
                        ui.close();
                    }
                    if ui.button("DSO").clicked() {
                        // Do nothing for now
                        ui.close();
                    }
                    if ui.button("Planet").clicked() {
                        // Do nothing for now
                        ui.close();
                    }
                });
            });
        });
}
