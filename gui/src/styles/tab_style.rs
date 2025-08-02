use iced::{
    widget::{button, button::Status, container, text, Button, Container},
    Background, Border, Color, Length, Theme,
};

use crate::styles;

pub fn tab_content<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> Container<'a, Message>
where
    Message: 'a + Clone,
{
    container(content).style(|_theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(styles::TAB_BACKGROUND_COLOR)),
        border: iced::Border {
            radius: 2.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: iced::Shadow {
            offset: iced::Vector::new(2.0, 2.0),
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

pub fn tab_button<'a, Message>(label: &'a str, active: bool) -> Button<'a, Message>
where
    Message: 'a + Clone,
{
    button(text(label).width(Length::Fill).size(14).center()).style(
        move |_theme: &Theme, status| {
            let hovered = matches!(status, Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(match active {
                    true => styles::TAB_BACKGROUND_COLOR,
                    false => styles::INACTIVE_TAB_COLOR,
                })),

                text_color: match active || hovered {
                    true => styles::ACCENT_COLOR,
                    false => styles::TEXT_COLOR,
                },
                shadow: iced::Shadow {
                    offset: iced::Vector::new(2.0, 2.0),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    }, // soft drop shadow
                    blur_radius: 2.0,
                },
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 2.0.into(),
                },
            }
        },
    )
}
