use crate::starfield::StarfieldState;
use bevy::prelude::*;
use bevy_egui::egui;
use chrono::{DateTime, Duration, Utc};
use overpass_planner::{get_overpasses, ObserverLocation, Overpass};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

#[derive(Resource)]
pub struct SatelliteSearchState {
    pub norad_id_input: String,
    pub norad_id: Option<u32>,
    pub overpasses: Vec<Overpass>,
    pub selected_overpass: Option<usize>,
    pub search_in_progress: bool,
    pub search_error: Option<String>,
}

impl Default for SatelliteSearchState {
    fn default() -> Self {
        Self {
            norad_id_input: String::new(),
            norad_id: None,
            overpasses: Vec::new(),
            selected_overpass: None,
            search_in_progress: false,
            search_error: None,
        }
    }
}

// Channel for async search results
#[derive(Resource)]
pub struct SearchResultChannel {
    pub sender: Mutex<Sender<SearchResult>>,
    pub receiver: Mutex<Receiver<SearchResult>>,
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Success { overpasses: Vec<Overpass> },
    Error { message: String },
}

impl Default for SearchResultChannel {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            sender: Mutex::new(tx),
            receiver: Mutex::new(rx),
        }
    }
}

pub fn render_satellite_window(
    mut search_state: ResMut<SatelliteSearchState>,
    starfield_state: Res<StarfieldState>,
    search_channel: Res<SearchResultChannel>,
    mut menu_state: ResMut<crate::menu::MenuState>,
    mut camera_query: Query<&mut bevy_egui::EguiContext, With<bevy::prelude::Camera3d>>,
) {
    let Ok(mut egui_context) = camera_query.single_mut() else {
        return;
    };
    let ctx = egui_context.get_mut();

    // Handle search results
    if let Ok(receiver) = search_channel.receiver.lock() {
        while let Ok(result) = receiver.try_recv() {
            search_state.search_in_progress = false;
            match result {
                SearchResult::Success { overpasses } => {
                    search_state.overpasses = overpasses;
                    search_state.search_error = None;
                }
                SearchResult::Error { message } => {
                    search_state.search_error = Some(message);
                    search_state.overpasses.clear();
                }
            }
        }
    }

    // Only show window if it's supposed to be open
    if !menu_state.satellite_window_open {
        return;
    }

    egui::Window::new("Satellite Tracking")
        .collapsible(false)
        .resizable(true)
        .default_size([800.0, 600.0])
        .open(&mut menu_state.satellite_window_open)
        .show(ctx, |ui| {
            // Search section
            ui.vertical(|ui| {
                ui.heading("Search Satellite");
                ui.horizontal(|ui| {
                    ui.label("NORAD ID:");
                    ui.text_edit_singleline(&mut search_state.norad_id_input);
                    if ui.button("Search").clicked() {
                        // Parse NORAD ID
                        match search_state.norad_id_input.trim().parse::<u32>() {
                            Ok(norad_id) => {
                                search_state.norad_id = Some(norad_id);
                                search_state.search_in_progress = true;
                                search_state.search_error = None;
                                search_state.overpasses.clear();
                                search_state.selected_overpass = None;

                                // Spawn async task to fetch overpasses
                                let location = ObserverLocation {
                                    latitude: starfield_state.lat_deg,
                                    longitude: starfield_state.lon_deg,
                                    altitude: 0.0, // Sea level
                                };
                                let time_window = Duration::hours(24);
                                // Clone the sender from the Mutex
                                let sender = {
                                    let guard = search_channel.sender.lock().unwrap();
                                    guard.clone()
                                };

                                std::thread::spawn(move || {
                                    let rt = tokio::runtime::Runtime::new().unwrap();
                                    rt.block_on(async move {
                                        match get_overpasses(norad_id, location, time_window).await
                                        {
                                            Ok(overpasses) => {
                                                let _ = sender
                                                    .send(SearchResult::Success { overpasses });
                                            }
                                            Err(e) => {
                                                let _ = sender.send(SearchResult::Error {
                                                    message: format!("{}", e),
                                                });
                                            }
                                        }
                                    });
                                });
                            }
                            Err(_) => {
                                search_state.search_error = Some("Invalid NORAD ID".to_string());
                            }
                        }
                    }
                });

                // Show error if any
                if let Some(error) = &search_state.search_error {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 100, 100),
                        format!("Error: {}", error),
                    );
                }

                ui.separator();

                // Overpasses table
                ui.heading("Upcoming Overpasses (Next 24 Hours)");

                if search_state.search_in_progress {
                    ui.label("Searching...");
                } else if search_state.overpasses.is_empty() && search_state.search_error.is_none()
                {
                    ui.label("No overpasses found. Enter a NORAD ID and click Search.");
                } else {
                    // Table header
                    ui.horizontal(|ui| {
                        ui.strong("Start Time");
                        ui.strong("End Time");
                        ui.strong("Duration");
                        ui.strong("Max Elevation");
                        ui.strong("Midpoint");
                    });
                    ui.separator();

                    // Table rows
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            let overpasses = search_state.overpasses.clone();
                            let selected = search_state.selected_overpass;

                            for (index, overpass) in overpasses.iter().enumerate() {
                                let is_selected = selected == Some(index);

                                ui.horizontal(|ui| {
                                    let response = ui.selectable_label(
                                        is_selected,
                                        format_time(overpass.start_time),
                                    );
                                    if response.clicked() {
                                        search_state.selected_overpass = Some(index);
                                    }

                                    ui.label(format_time(overpass.end_time));

                                    let duration_min =
                                        (overpass.end_time - overpass.start_time).num_minutes();
                                    ui.label(format!("{:.1} min", duration_min));

                                    ui.label(format!("{:.2}°", overpass.max_elevation));

                                    ui.label(format_time(overpass.midpoint_time));
                                });

                                if is_selected {
                                    ui.painter().rect_filled(
                                        ui.available_rect_before_wrap(),
                                        0.0,
                                        egui::Color32::from_rgba_unmultiplied(50, 100, 150, 50),
                                    );
                                }
                            }
                        });
                }

                ui.separator();

                // Track button
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let enabled = search_state.selected_overpass.is_some();
                        if ui
                            .add_enabled(enabled, egui::Button::new("Track"))
                            .clicked()
                        {
                            if let Some(index) = search_state.selected_overpass {
                                if index < search_state.overpasses.len() {
                                    let overpass = &search_state.overpasses[index];
                                    // TODO: Implement tracking
                                    println!("Tracking overpass: {:?}", overpass);
                                }
                            }
                        }
                    });
                });
            });
        });
}

fn format_time(dt: DateTime<Utc>) -> String {
    dt.format("%H:%M:%S").to_string()
}
