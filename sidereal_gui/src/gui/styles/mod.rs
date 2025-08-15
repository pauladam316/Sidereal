use iced::{
    theme::{Custom, Palette},
    Color, Theme,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
pub mod button_style;
pub mod container_style;
pub mod picklist_style;
pub mod tab_style;
pub mod text_input_style;

pub const TAB_BACKGROUND_COLOR: Color = Color::from_rgb(0.184, 0.184, 0.184);
pub const CONTAINER_LAYER_1: Color = Color::from_rgb(0.25, 0.25, 0.25);
pub const CONTAINER_LAYER_2: Color = Color::from_rgb(0.3, 0.3, 0.3);
pub const CONTAINER_LAYER_3: Color = Color::from_rgb(0.4, 0.4, 0.4);
pub const INACTIVE_TAB_COLOR: Color = Color::from_rgb(0.129, 0.129, 0.129);
pub const ACCENT_COLOR: Color = Color::from_rgb(0.918, 0.878, 0.349);
pub const TEXT_COLOR: Color = Color::from_rgb(0.875, 0.875, 0.875);
pub const BACKGROUND_TEXT_COLOR: Color = Color::from_rgb(0.675, 0.675, 0.675);
pub const BUTTON_COLOR: Color = Color::from_rgb(0.302, 0.302, 0.302);
pub const TRACK_BUTTON_COLOR: Color = Color::from_rgb(0.302, 0.42, 0.302);
pub const STOP_TRACK_BUTTON_COLOR: Color = Color::from_rgb(0.42, 0.302, 0.302);
pub const BACKGROUND_COLOR: Color = Color::from_rgb(0.129, 0.129, 0.129);
pub const ELEMENT_BORDER: Color = Color::from_rgb(0.7, 0.7, 0.7);
pub const TRACK_BUTTON_BORDER: Color = Color::from_rgb(0.7, 0.86, 0.7);
pub const STOP_TRACK_BUTTON_BORDER: Color = Color::from_rgb(0.86, 0.7, 0.7);

pub static SIDEREAL_THEME: Lazy<Theme> = Lazy::new(|| {
    Theme::Custom(Arc::new(Custom::new(
        "Sidereal".to_owned(),
        Palette {
            background: BACKGROUND_COLOR,
            text: TEXT_COLOR,
            primary: Color::from_rgb(0.23, 0.6, 0.95),
            success: Color::from_rgb(0.2, 0.7, 0.4),
            danger: Color::from_rgb(0.9, 0.3, 0.3),
        },
    )))
});
