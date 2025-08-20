use std::fmt;

use crate::gui::styles::{
    self,
    container_style::{content_container, ContainerLayer},
};
use iced::{
    theme::Theme,
    widget::{container, text},
    Alignment, Background, Border, Length,
}; // adjust if located elsewhere

#[derive(Debug, Clone)]
pub enum ServerStatus {
    Disconnected,
    Connecting,
    Connected,
    ConnectionLost,
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerStatus::ConnectionLost => write!(f, "Connection Lost"),
            ServerStatus::Disconnected => write!(f, "Disconnected"),
            ServerStatus::Connecting => write!(f, "Connecting"),
            ServerStatus::Connected => write!(f, "Connected"),
        }
    }
}

impl Default for ServerStatus {
    fn default() -> Self {
        ServerStatus::Disconnected
    }
}

pub fn server_status_widget<'a, Message>(status: &ServerStatus) -> iced::Element<'a, Message>
where
    Message: Clone + 'a,
{
    // Choose colors per state (tweak to match your theme)
    let (bg, fg, border) = match status {
        ServerStatus::Disconnected => (
            styles::RED_BUTTON_COLOR,
            styles::RED_TEXT,
            styles::RED_BUTTON_BORDER,
        ),
        ServerStatus::ConnectionLost => (
            styles::RED_BUTTON_COLOR,
            styles::RED_TEXT,
            styles::RED_BUTTON_BORDER,
        ),
        ServerStatus::Connecting => (
            styles::AMBER_BUTTON_COLOR,
            styles::AMBER_TEXT,
            styles::AMBER_BUTTON_BORDER,
        ),
        ServerStatus::Connected => (
            styles::GREEN_BUTTON_COLOR,
            styles::GREEN_TEXT,
            styles::GREEN_BUTTON_BORDER,
        ),
    };

    // Create the pill-like container
    let inner = text(status.to_string()).size(14).line_height(1.2);

    content_container(inner, ContainerLayer::Layer3)
        .padding([6, 12])
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Length::Shrink)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(fg),
            border: Border {
                color: border,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
}
