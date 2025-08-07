use iced::{
    widget::button::Status,
    widget::{button, Button},
    Background, Border, Color, Theme,
};

use crate::gui::styles;

pub fn sidereal_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> Button<'a, Message>
where
    Message: 'a + Clone,
{
    button(content)
        .padding([6, 12])
        .style(move |_theme: &Theme, status| {
            let hovered = matches!(status, Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(styles::BUTTON_COLOR)),

                text_color: if hovered {
                    styles::ACCENT_COLOR
                } else {
                    styles::TEXT_COLOR
                },

                shadow: iced::Shadow {
                    offset: iced::Vector::new(1.0, 1.0),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.2,
                    }, // soft drop shadow
                    blur_radius: 3.0,
                },
                border: Border {
                    color: if hovered {
                        styles::ACCENT_COLOR
                    } else {
                        styles::ELEMENT_BORDER
                    },
                    width: 1.0,
                    radius: 7.0.into(),
                },
            }
        })
}
