use crate::colors;
use crate::starfield::StarfieldState;
use crate::ui::widgets::{content_container_frame, planetarium_button, planetarium_text_input};
use bevy::prelude::*;
use bevy_egui::egui;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use overpass_planner::{get_overpasses, get_satellite_name, ObserverLocation, Overpass};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

#[derive(Resource)]
pub struct SatelliteSearchState {
    pub norad_id_input: String,
    pub norad_id: Option<u32>,
    pub satellite_name: Option<String>,
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
            satellite_name: None,
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
    Success {
        overpasses: Vec<Overpass>,
        satellite_name: Option<String>,
    },
    Error {
        message: String,
    },
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
    mut menu_state: ResMut<crate::ui::MenuState>,
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
                SearchResult::Success {
                    overpasses,
                    satellite_name,
                } => {
                    search_state.overpasses = overpasses;
                    search_state.satellite_name = satellite_name;
                    search_state.search_error = None;
                }
                SearchResult::Error { message } => {
                    search_state.search_error = Some(message);
                    search_state.overpasses.clear();
                    search_state.satellite_name = None;
                }
            }
        }
    }

    // Only show window if it's supposed to be open
    if !menu_state.satellite_window_open {
        return;
    }

    // Customize window frame with background color and padding
    let mut window_frame = egui::Frame::window(&ctx.style());
    window_frame.fill = colors::egui::WINDOW_BACKGROUND;
    // Set 4pt padding so containers have 8pt total spacing from window edges
    // (4pt window padding + 4pt container outer margin = 8pt total)
    window_frame.inner_margin = egui::Margin {
        left: 4,
        right: 4,
        top: 4,
        bottom: 4,
    };

    egui::Window::new(
        egui::RichText::new("Satellite Tracking")
            .size(14.0)
            .color(colors::egui::WINDOW_TITLE_COLOR),
    )
    .collapsible(false)
    .resizable(true)
    .default_size([400.0, 1000.0]) // 30% taller: 600 * 1.3 = 780
    .frame(window_frame)
    .open(&mut menu_state.satellite_window_open)
    .show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width());
            // Search section container
            let w = ui.available_width();
            ui.allocate_ui(egui::Vec2::new(w, 0.0), |ui| {
                content_container_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Search Satellite")
                                .size(14.0)
                                .color(colors::egui::WINDOW_TITLE_COLOR),
                        );
                        let row_h = 24.0;
                        ui.spacing_mut().interact_size.y = row_h;

                        // Use the text input height as the row height
                        let text_input_height = row_h - 2.0;
                        ui.horizontal(|ui| {
                            // Label
                            ui.add_sized(
                                egui::vec2(0.0, text_input_height),
                                egui::Label::new(
                                    egui::RichText::new("NORAD ID:")
                                        .size(12.0)
                                        .color(colors::egui::WINDOW_TITLE_COLOR),
                                ),
                            );

                            // Text input: shorter height
                            planetarium_text_input(
                                ui,
                                &mut search_state.norad_id_input,
                                150.0,
                                text_input_height,
                            );

                            // Button: same height as text input
                            let button_resp =
                                planetarium_button(ui, "Search", 80.0, text_input_height);

                            if button_resp.clicked() {
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
                                                // Fetch satellite name and overpasses in parallel
                                                let (overpasses_result, name_result) = tokio::join!(
                                                    get_overpasses(norad_id, location, time_window),
                                                    get_satellite_name(norad_id)
                                                );

                                                match overpasses_result {
                                                    Ok(overpasses) => {
                                                        let satellite_name = name_result.ok();
                                                        let _ = sender.send(SearchResult::Success {
                                                            overpasses,
                                                            satellite_name,
                                                        });
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
                                        search_state.search_error =
                                            Some("Invalid NORAD ID".to_string());
                                    }
                                }
                            }
                        });

                        // Show satellite name if found
                        if let Some(name) = &search_state.satellite_name {
                            ui.label(
                                egui::RichText::new(format!("Found satellite: {}", name))
                                    .size(12.0)
                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                            );
                        }

                        // Show error if any
                        if let Some(error) = &search_state.search_error {
                            ui.label(
                                egui::RichText::new(format!("Error: {}", error))
                                    .size(12.0)
                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                            );
                        }
                    });
                });
            });

            // Overpasses section container
            // Spacing is handled by container outer margins (4pt top + 4pt bottom = 8pt total)
            let w = ui.available_width();
            ui.allocate_ui(egui::Vec2::new(w, 0.0), |ui| {
                content_container_frame().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new("Upcoming Overpasses (Next 24 Hours)")
                                .size(14.0)
                                .color(colors::egui::WINDOW_TITLE_COLOR),
                        );

                        // Show site location
                        ui.label(
                            egui::RichText::new(format!(
                                "Site: {:.4}°N, {:.4}°E, {:.0}m",
                                starfield_state.lat_deg,
                                starfield_state.lon_deg,
                                0.0
                            ))
                            .size(12.0)
                            .color(colors::egui::WINDOW_TITLE_COLOR),
                        );

                        // Note about timezone
                        ui.label(
                            egui::RichText::new("All times shown in EST (UTC-5)")
                                .size(11.0)
                                .color(colors::egui::WINDOW_TITLE_COLOR),
                        );

                        if search_state.search_in_progress {
                            ui.label(
                                egui::RichText::new("Searching...")
                                    .size(12.0)
                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                            );
                        } else if search_state.overpasses.is_empty()
                            && search_state.search_error.is_none()
                        {
                            ui.label(
                                egui::RichText::new(
                                    "No overpasses found. Enter a NORAD ID and click Search.",
                                )
                                .size(12.0)
                                .color(colors::egui::WINDOW_TITLE_COLOR),
                            );
                        } else {
                            // Table using Grid layout
                            // Allocate dynamic height: use available space or minimum 100px
                            let available_height = ui.available_height();
                            let scroll_height = available_height.max(140.0);
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), scroll_height),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false]) // Don't shrink to content
                                        .show(ui, |ui| {
                                            // Add padding to prevent scrollbar from overlapping data
                                            ui.set_width(ui.available_width() - 20.0); // Reserve space for scrollbar
                                            let overpasses = search_state.overpasses.clone();
                                            let selected = search_state.selected_overpass;

                                            egui::Grid::new("overpasses_table")
                                                .spacing(egui::vec2(8.0, 4.0))
                                                .show(ui, |ui| {
                                                    // Header row
                                                    ui.strong(
                                                        egui::RichText::new("Date")
                                                            .size(12.0)
                                                            .color(
                                                                colors::egui::WINDOW_TITLE_COLOR,
                                                            ),
                                                    );
                                                    ui.strong(
                                                        egui::RichText::new("Start Time")
                                                            .size(12.0)
                                                            .color(
                                                                colors::egui::WINDOW_TITLE_COLOR,
                                                            ),
                                                    );
                                                    ui.strong(
                                                        egui::RichText::new("End Time")
                                                            .size(12.0)
                                                            .color(
                                                                colors::egui::WINDOW_TITLE_COLOR,
                                                            ),
                                                    );
                                                    ui.strong(
                                                        egui::RichText::new("Duration")
                                                            .size(12.0)
                                                            .color(
                                                                colors::egui::WINDOW_TITLE_COLOR,
                                                            ),
                                                    );
                                                    ui.strong(
                                                        egui::RichText::new("Max Elevation")
                                                            .size(12.0)
                                                            .color(
                                                                colors::egui::WINDOW_TITLE_COLOR,
                                                            ),
                                                    );
                                                     ui.strong(
                                                         egui::RichText::new("Midpoint")
                                                             .size(12.0)
                                                             .color(
                                                                 colors::egui::WINDOW_TITLE_COLOR,
                                                             ),
                                                     );
                                                     ui.strong(
                                                         egui::RichText::new("Night")
                                                             .size(12.0)
                                                             .color(
                                                                 colors::egui::WINDOW_TITLE_COLOR,
                                                             ),
                                                     );
                                                     ui.strong(
                                                         egui::RichText::new("Lit")
                                                             .size(12.0)
                                                             .color(
                                                                 colors::egui::WINDOW_TITLE_COLOR,
                                                             ),
                                                     );
                                                     ui.strong(
                                                         egui::RichText::new("").size(12.0).color(
                                                             colors::egui::WINDOW_TITLE_COLOR,
                                                         ),
                                                     ); // Empty header for Track button column
                                                     ui.end_row();

                                                    // Data rows
                                                    for (index, overpass) in
                                                        overpasses.iter().enumerate()
                                                    {
                                                        let is_selected = selected == Some(index);
                                                        let row_start_rect =
                                                            ui.available_rect_before_wrap();

                                                        // Date column
                                                        ui.label(
                                                            egui::RichText::new(format_date(
                                                                overpass.start_time,
                                                            ))
                                                            .size(12.0)
                                                            .color(colors::egui::WINDOW_TITLE_COLOR),
                                                        );

                                                        // Start time column - make first item non-selectable
                                                        let response = if index == 0 {
                                                            // First item: use label instead of selectable_label
                                                            ui.label(
                                                                egui::RichText::new(format_time(
                                                                    overpass.start_time,
                                                                ))
                                                                .size(12.0)
                                                                .color(colors::egui::WINDOW_TITLE_COLOR),
                                                            )
                                                        } else {
                                                            // Other items: selectable
                                                            ui.selectable_label(
                                                                is_selected,
                                                                egui::RichText::new(format_time(
                                                                    overpass.start_time,
                                                                ))
                                                                .size(12.0)
                                                                .color(colors::egui::WINDOW_TITLE_COLOR),
                                                            )
                                                        };
                                                        if index > 0 && response.clicked() {
                                                            search_state.selected_overpass =
                                                                Some(index);
                                                        }

                                                        ui.label(
                                                    egui::RichText::new(format_time(
                                                        overpass.end_time,
                                                    ))
                                                    .size(12.0)
                                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                                                );

                                                        let duration_min = (overpass.end_time
                                                            - overpass.start_time)
                                                            .num_minutes();
                                                        ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:.1} min",
                                                        duration_min
                                                    ))
                                                    .size(12.0)
                                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                                                );

                                                        ui.label(
                                                    egui::RichText::new(format!(
                                                        "{:.2}°",
                                                        overpass.max_elevation
                                                    ))
                                                    .size(12.0)
                                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                                                );

                                                        ui.label(
                                                    egui::RichText::new(format_time(
                                                        overpass.midpoint_time,
                                                    ))
                                                    .size(12.0)
                                                    .color(colors::egui::WINDOW_TITLE_COLOR),
                                                );

                                                        // Night column
                                                        ui.label(
                                                            egui::RichText::new(if overpass.is_night {
                                                                "Yes"
                                                            } else {
                                                                "No"
                                                            })
                                                            .size(12.0)
                                                            .color(colors::egui::WINDOW_TITLE_COLOR),
                                                        );

                                                        // Lit column
                                                        ui.label(
                                                            egui::RichText::new(if overpass.is_lit {
                                                                "Yes"
                                                            } else {
                                                                "No"
                                                            })
                                                            .size(12.0)
                                                            .color(colors::egui::WINDOW_TITLE_COLOR),
                                                        );

                                                        // Track button for this row
                                                        let track_button_height = 20.0;
                                                        if planetarium_button(
                                                            ui,
                                                            "Track",
                                                            60.0,
                                                            track_button_height,
                                                        )
                                                        .clicked()
                                                        {
                                                            // TODO: Implement tracking
                                                            println!(
                                                                "Tracking overpass: {:?}",
                                                                overpass
                                                            );
                                                        }

                                                        let row_end_rect =
                                                            ui.available_rect_before_wrap();
                                                        ui.end_row();

                                                        // Draw selection highlight for the entire row
                                                        if is_selected {
                                                            let row_rect = egui::Rect::from_min_max(
                                                                row_start_rect.min,
                                                                egui::pos2(
                                                                    row_end_rect.max.x,
                                                                    row_start_rect.min.y
                                                                        + row_start_rect.height(),
                                                                ),
                                                            );
                                                            ui.painter().rect_filled(
                                                        row_rect,
                                                        0.0,
                                                        egui::Color32::from_rgba_unmultiplied(
                                                            50, 100, 150, 50,
                                                        ),
                                                    );
                                                        }
                                                    }
                                                });
                                        });
                                },
                            );
                        }
                    });
                });
            });
        });
    });
}

fn format_time(dt: DateTime<Utc>) -> String {
    // Convert UTC to EST (UTC-5)
    let est_offset = FixedOffset::east_opt(-5 * 3600).unwrap();
    let est_time = dt.with_timezone(&est_offset);
    est_time.format("%H:%M:%S").to_string()
}

fn format_date(dt: DateTime<Utc>) -> String {
    // Convert UTC to EST (UTC-5)
    let est_offset = FixedOffset::east_opt(-5 * 3600).unwrap();
    let est_time = dt.with_timezone(&est_offset);
    est_time.format("%Y-%m-%d").to_string()
}
