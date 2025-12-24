use iced::{
    widget::{container, text_input, text_input::Status, Container, TextInput},
    Background, Border, Theme,
};

use crate::gui::styles;

pub fn sidereal_text_input<'a, Message>(placeholder: &str, value: &str) -> TextInput<'a, Message>
where
    Message: 'a + Clone,
{
    text_input(placeholder, value).style(move |_theme: &Theme, status| {
        let hovered = matches!(status, Status::Hovered);
        let focused = matches!(status, Status::Focused);

        let border_color = if hovered || focused {
            styles::ACCENT_COLOR
        } else {
            styles::ELEMENT_BORDER
        };

        iced::widget::text_input::Style {
            background: Background::Color(styles::BUTTON_COLOR),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 7.0.into(),
            },
            icon: styles::ACCENT_COLOR,
            placeholder: styles::BACKGROUND_TEXT_COLOR,
            value: styles::TEXT_COLOR,
            selection: styles::ACCENT_COLOR,
        }
    })
}

/// A read-only text display that looks like a text input but cannot be edited
/// and doesn't highlight on hover. The value can only be changed programmatically.
pub fn sidereal_readonly_text<'a, Message>(value: &'a str) -> Container<'a, Message>
where
    Message: 'a + Clone,
{
    container(iced::widget::text(value))
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(styles::CONTAINER_LAYER_1)),
            border: Border {
                color: styles::ELEMENT_BORDER, // Always use the same border color (no hover effect)
                width: 1.0,
                radius: 7.0.into(),
            },
            text_color: Some(styles::TEXT_COLOR),
            ..Default::default()
        })
        .padding([6, 6]) // Match typical text input padding
}
