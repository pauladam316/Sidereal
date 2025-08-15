use iced::{
    widget::{container, Container},
    Background, Color, Theme,
};

use crate::gui::styles;

pub enum ContainerLayer {
    Layer1,
    Layer2,
    Layer3,
}
pub fn content_container<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
    layer: ContainerLayer,
) -> Container<'a, Message>
where
    Message: 'a + Clone,
{
    container(content)
        .style(move |_theme: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(match layer {
                ContainerLayer::Layer1 => styles::CONTAINER_LAYER_1,
                ContainerLayer::Layer2 => styles::CONTAINER_LAYER_2,
                ContainerLayer::Layer3 => styles::CONTAINER_LAYER_3,
            })),
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
        .padding(10)
}
