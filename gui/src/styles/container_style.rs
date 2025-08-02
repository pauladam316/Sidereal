use iced::{
    widget::{button, container, Button, Container},
    Background, Border, Color, Theme,
};

use crate::styles;

pub fn content_container<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> Container<'a, Message>
where
    Message: 'a + Clone,
{
    container(content).style(|_theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(styles::CONTAINER_COLOR)),
        border: iced::Border {
            radius: 10.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: iced::Shadow {
            offset: iced::Vector::new(0.0, 2.0),
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.5,
            },
            blur_radius: 2.0,
        },
        text_color: None,
    })
}
