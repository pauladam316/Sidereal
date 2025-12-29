mod satellite_window;

use crate::colors;
use bevy::feathers::cursor::EntityCursor;
use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
pub use satellite_window::SatelliteWindow;

#[derive(Component)]
pub struct MenuBar;

#[derive(Component)]
pub struct MenuItem {
    pub action: MenuAction,
}

#[derive(Component)]
pub struct CloseButton;

#[derive(Component)]
pub struct MenuDropdown {
    pub menu_id: usize,
}

#[derive(Component)]
pub struct MenuLabel {
    pub menu_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    TrackSatellite,
    TrackDSO,
    TrackPlanet,
}

#[derive(Resource, Default)]
pub struct MenuState {
    pub satellite_window_open: bool,
    pub active_dropdown: Option<usize>,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(Startup, spawn_menu_bar)
            .add_systems(
                Update,
                (
                    handle_menu_label_interaction,
                    update_dropdown_visibility,
                    handle_menu_item_hover,
                    handle_menu_clicks,
                    handle_window_close,
                    handle_click_outside_menu,
                ),
            );
    }
}

fn spawn_menu_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load Segoe UI font (same as Iced default on Windows)
    let font = asset_server.load("segoeui.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(28.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(colors::MENU_BAR_BACKGROUND),
            ZIndex(1000),
            MenuBar,
        ))
        .with_children(|parent| {
            spawn_menu_group(
                parent,
                "Track",
                0,
                vec![
                    ("Satellite", MenuAction::TrackSatellite),
                    ("DSO", MenuAction::TrackDSO),
                    ("Planet", MenuAction::TrackPlanet),
                ],
                font.clone(),
            );
        });
}

fn spawn_menu_group(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    menu_id: usize,
    items: Vec<(&str, MenuAction)>,
    font: Handle<Font>,
) {
    parent
        .spawn((Node {
            margin: UiRect::horizontal(Val::Px(2.0)),
            ..default()
        },))
        .with_children(|parent| {
            // Menu label button
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::new(
                            Val::Px(10.0),
                            Val::Px(10.0),
                            Val::Px(5.0),
                            Val::Px(5.0),
                        ),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    MenuLabel { menu_id },
                    EntityCursor::System(SystemCursorIcon::Pointer),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(label),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(colors::TEXT_COLOR_NORMAL),
                    ));
                });

            // Dropdown menu
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(26.0),
                        left: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(colors::MENU_DROPDOWN_BACKGROUND),
                    BorderColor::all(colors::MENU_DROPDOWN_BORDER),
                    Visibility::Hidden,
                    ZIndex(1001),
                    MenuDropdown { menu_id },
                ))
                .with_children(|parent| {
                    for (item_label, action) in items {
                        spawn_menu_item(parent, item_label, action, font.clone());
                    }
                });
        });
}

fn spawn_menu_item(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuAction,
    font: Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(5.0), Val::Px(5.0)),
                margin: UiRect::vertical(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            MenuItem { action },
            EntityCursor::System(SystemCursorIcon::Pointer),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(colors::TEXT_COLOR_SECONDARY),
            ));
        });
}

fn handle_menu_label_interaction(
    mut menu_state: ResMut<MenuState>,
    menu_label_query: Query<(&Interaction, &MenuLabel, &Children), Changed<Interaction>>,
    mut text_query: Query<&mut TextColor>,
) {
    let highlight_color = colors::TEXT_COLOR_HIGHLIGHT;
    let normal_color = colors::TEXT_COLOR_NORMAL;

    for (interaction, menu_label, children) in menu_label_query.iter() {
        // Update text color based on interaction
        for child in children {
            if let Ok(mut text_color) = text_query.get_mut(*child) {
                match *interaction {
                    Interaction::Hovered | Interaction::Pressed => {
                        *text_color = TextColor(highlight_color);
                    }
                    Interaction::None => {
                        *text_color = TextColor(normal_color);
                    }
                }
            }
        }

        // Handle menu state changes
        match *interaction {
            Interaction::Pressed => {
                // Toggle dropdown on click
                if menu_state.active_dropdown == Some(menu_label.menu_id) {
                    menu_state.active_dropdown = None;
                } else {
                    menu_state.active_dropdown = Some(menu_label.menu_id);
                }
            }
            Interaction::Hovered => {
                // If any dropdown is already open, switch to this one on hover
                if menu_state.active_dropdown.is_some() {
                    menu_state.active_dropdown = Some(menu_label.menu_id);
                }
            }
            Interaction::None => {}
        }
    }
}

fn update_dropdown_visibility(
    menu_state: Res<MenuState>,
    mut dropdown_query: Query<(&MenuDropdown, &mut Visibility)>,
) {
    for (dropdown, mut visibility) in dropdown_query.iter_mut() {
        if menu_state.active_dropdown == Some(dropdown.menu_id) {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn handle_menu_item_hover(
    menu_item_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<MenuItem>)>,
    mut text_query: Query<&mut TextColor>,
) {
    let highlight_color = colors::TEXT_COLOR_HIGHLIGHT;
    let normal_color = colors::TEXT_COLOR_SECONDARY;

    for (interaction, children) in menu_item_query.iter() {
        for child in children {
            if let Ok(mut text_color) = text_query.get_mut(*child) {
                match *interaction {
                    Interaction::Hovered | Interaction::Pressed => {
                        *text_color = TextColor(highlight_color);
                    }
                    Interaction::None => {
                        *text_color = TextColor(normal_color);
                    }
                }
            }
        }
    }
}

fn handle_click_outside_menu(
    mut menu_state: ResMut<MenuState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    menu_label_query: Query<&Interaction, With<MenuLabel>>,
    menu_item_query: Query<&Interaction, With<MenuItem>>,
    dropdown_query: Query<&Interaction, With<MenuDropdown>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if any menu-related element is being interacted with
        let clicking_label = menu_label_query
            .iter()
            .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);
        let clicking_item = menu_item_query
            .iter()
            .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);
        let clicking_dropdown = dropdown_query
            .iter()
            .any(|i| *i == Interaction::Pressed || *i == Interaction::Hovered);

        if !clicking_label && !clicking_item && !clicking_dropdown {
            menu_state.active_dropdown = None;
        }
    }
}

fn handle_menu_clicks(
    interaction_query: Query<(&Interaction, &MenuItem), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<MenuState>,
    mut commands: Commands,
    satellite_window_query: Query<Entity, With<SatelliteWindow>>,
    asset_server: Res<AssetServer>,
) {
    for (interaction, menu_item) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Close dropdown after selection
            menu_state.active_dropdown = None;

            match menu_item.action {
                MenuAction::TrackSatellite => {
                    if satellite_window_query.is_empty() {
                        menu_state.satellite_window_open = true;
                        let font = asset_server.load("segoeui.ttf");
                        satellite_window::spawn_satellite_window(&mut commands, font);
                    }
                }
                MenuAction::TrackDSO => {
                    // Do nothing for now
                }
                MenuAction::TrackPlanet => {
                    // Do nothing for now
                }
            }
        }
    }
}

fn handle_window_close(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseButton>)>,
    satellite_window_query: Query<Entity, With<SatelliteWindow>>,
    mut menu_state: ResMut<MenuState>,
    mut commands: Commands,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Close satellite window
            for entity in satellite_window_query.iter() {
                commands.entity(entity).despawn();
                menu_state.satellite_window_open = false;
            }
        }
    }
}

