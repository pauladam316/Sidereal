use iced::{
    widget::{text_input, text_input::Status, TextInput},
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
