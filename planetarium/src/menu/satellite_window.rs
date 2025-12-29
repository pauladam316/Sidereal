use crate::colors;
use bevy::feathers::cursor::EntityCursor;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

use super::CloseButton;

#[derive(Component)]
pub struct SatelliteWindow;

#[derive(Component)]
struct SearchButton;

#[derive(Component)]
struct RefreshButton;

#[derive(Component)]
struct CatalogSelector {
    selected: usize,
}

#[derive(Component)]
struct AutoUpdateToggle {
    enabled: bool,
}

pub fn spawn_satellite_window(commands: &mut Commands, font: Handle<Font>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(colors::MODAL_OVERLAY),
            ZIndex(2000),
            SatelliteWindow,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(650.0),
                        min_height: Val::Px(500.0),
                        max_height: Val::Percent(85.0),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors::MODAL_BACKGROUND),
                    BorderColor::all(colors::MODAL_BORDER),
                ))
                .with_children(|parent| {
                    // Title bar
                    spawn_title_bar(parent, font.clone());

                    // Content area with scroll
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0,
                                padding: UiRect::all(Val::Px(20.0)),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BackgroundColor(colors::MENU_BAR_BACKGROUND),
                        ))
                        .with_children(|parent| {
                            spawn_satellite_content(parent, font.clone());
                        });
                });
        });
}

fn spawn_title_bar(parent: &mut ChildSpawnerCommands, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(14.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(colors::MODAL_TITLE_BAR),
            BorderColor::all(colors::MODAL_TITLE_BORDER),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Satellite Tracking"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_BRIGHT),
            ));

            // Close button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::CLOSE_BUTTON),
                    CloseButton,
                    EntityCursor::System(SystemCursorIcon::Pointer),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("×"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn spawn_satellite_content(parent: &mut ChildSpawnerCommands, font: Handle<Font>) {
    // Section: Search
    spawn_section_header(parent, "Search Satellite", font.clone());

    // Search input row
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|parent| {
            // Text input field (simulated with a button for now, as Bevy doesn't have native text input in UI)
            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(32.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                    BorderColor::all(Color::srgb(0.3, 0.3, 0.35)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Enter satellite name or NORAD ID..."),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });

            // Search button
            spawn_action_button(parent, "Search", SearchButton, font.clone());
        });

    // Section: Catalog Selection
    spawn_section_header(parent, "TLE Catalog", font.clone());

    // Radio button group (simulated with buttons)
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|parent| {
            spawn_radio_option(parent, "Active Satellites", true, font.clone());
            spawn_radio_option(parent, "Space Stations", false, font.clone());
            spawn_radio_option(parent, "Weather Satellites", false, font.clone());
            spawn_radio_option(parent, "Amateur Radio", false, font.clone());
        });

    // Section: Options
    spawn_section_header(parent, "Options", font.clone());

    // Checkbox option
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },))
        .with_children(|parent| {
            spawn_checkbox(parent, "Auto-update position", true, font.clone());
            spawn_checkbox(parent, "Show ground track", false, font.clone());
            spawn_checkbox(parent, "Show orbit", true, font.clone());
        });

    // Slider section
    spawn_section_header(parent, "Update Interval", font.clone());

    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|parent| {
            // Slider (simulated with a progress bar)
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.35)),
                ))
                .with_children(|parent| {
                    // Filled portion (60%)
                    parent.spawn((
                        Node {
                            width: Val::Percent(60.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.6, 0.9)),
                    ));
                });

            // Value label
            parent.spawn((
                Text::new("Update every 3 seconds"),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_SECONDARY),
            ));
        });

    // Section: Status
    spawn_section_header(parent, "Status", font.clone());

    // Status information
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
        BorderColor::all(Color::srgb(0.25, 0.25, 0.3)),
        ))
        .with_children(|parent| {
            spawn_status_row(parent, "TLE Age:", "2 days", font.clone());
            spawn_status_row(parent, "Next Pass:", "In 45 minutes", font.clone());
            spawn_status_row(parent, "Altitude:", "420 km", font.clone());
            spawn_status_row(parent, "Velocity:", "7.66 km/s", font.clone());
        });

    // Progress bar
    spawn_section_header(parent, "Download Progress", font.clone());

    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(24.0),
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
        BorderColor::all(Color::srgb(0.3, 0.3, 0.35)),
        ))
        .with_children(|parent| {
            // Progress bar background
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },))
                .with_children(|parent| {
                    // Filled portion (75%)
                    parent.spawn((
                        Node {
                            width: Val::Percent(75.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
                    ));
                });

            // Progress text (overlay)
            parent.spawn((
                Text::new("75% Complete"),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_BRIGHT),
                ZIndex(1),
            ));
        });

    // Action buttons at the bottom
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            column_gap: Val::Px(10.0),
            margin: UiRect::top(Val::Px(10.0)),
            ..default()
        },))
        .with_children(|parent| {
            spawn_action_button(parent, "Refresh TLE", RefreshButton, font.clone());
            spawn_secondary_button(parent, "Clear", font.clone());
        });
}

fn spawn_section_header(parent: &mut ChildSpawnerCommands, title: &str, font: Handle<Font>) {
    parent.spawn((
        Text::new(title),
        TextFont {
            font,
            font_size: 13.0,
            ..default()
        },
        TextColor(colors::TEXT_COLOR_BRIGHT),
        Node {
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        },
    ));
}

fn spawn_radio_option(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    selected: bool,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            // Radio circle
            parent
                .spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(if selected {
                        Color::srgb(0.2, 0.6, 0.9)
                    } else {
                        Color::srgb(0.4, 0.4, 0.45)
                    }),
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    if selected {
                        parent.spawn((
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 0.9)),
                        ));
                    }
                });

            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_NORMAL),
            ));
        });
}

fn spawn_checkbox(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    checked: bool,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            // Checkbox square
            parent
                .spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(if checked {
                        Color::srgb(0.2, 0.6, 0.9)
                    } else {
                        Color::srgb(0.4, 0.4, 0.45)
                    }),
                    BackgroundColor(if checked {
                        Color::srgb(0.2, 0.6, 0.9)
                    } else {
                        Color::NONE
                    }),
                ))
                .with_children(|parent| {
                    if checked {
                        parent.spawn((
                            Text::new("✓"),
                            TextFont {
                                font: font.clone(),
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    }
                });

            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_NORMAL),
            ));
        });
}

fn spawn_action_button<T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: T,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.5, 0.8)),
            BorderColor::all(Color::srgb(0.3, 0.6, 0.9)),
            marker,
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_secondary_button(parent: &mut ChildSpawnerCommands, label: &str, font: Handle<Font>) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.28)),
            BorderColor::all(Color::srgb(0.35, 0.35, 0.4)),
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_NORMAL),
            ));
        });
}

fn spawn_status_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    font: Handle<Font>,
) {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_SECONDARY),
            ));

            parent.spawn((
                Text::new(value),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_BRIGHT),
            ));
        });
}

