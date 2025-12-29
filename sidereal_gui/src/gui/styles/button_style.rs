use iced::{
    widget::button::Status,
    widget::{button, container, Button},
    Alignment, Background, Border, Color, Theme,
};

use crate::gui::styles;

pub fn sidereal_button<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
    message: Option<Message>,
    enabled: bool,
) -> Button<'a, Message>
where
    Message: 'a + Clone,
{
    let mut btn = button(
        container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(iced::Length::Fill),
    )
    .padding([6, 12]);

    // Only set on_press if enabled and message is provided
    if enabled {
        if let Some(msg) = message {
            btn = btn.on_press(msg);
        }
    }

    btn.style(move |_theme: &Theme, status| {
        let hovered = matches!(status, Status::Hovered);
        let disabled_color = Color {
            r: 0.3,
            g: 0.3,
            b: 0.3,
            a: 1.0,
        };
        let disabled_text_color = Color {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };

        iced::widget::button::Style {
            background: Some(Background::Color(if enabled {
                styles::BUTTON_COLOR
            } else {
                disabled_color
            })),

            text_color: if !enabled {
                disabled_text_color
            } else if hovered {
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
                    a: if enabled { 0.2 } else { 0.1 },
                }, // soft drop shadow
                blur_radius: 3.0,
            },
            border: Border {
                color: if !enabled {
                    disabled_color
                } else if hovered {
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

pub fn track_button<'a, Message>(
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
                background: Some(Background::Color(styles::GREEN_BUTTON_COLOR)),

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
                        styles::GREEN_BUTTON_BORDER
                    },
                    width: 2.0,
                    radius: 20.0.into(),
                },
            }
        })
}

pub fn stop_track_button<'a, Message>(
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
                background: Some(Background::Color(styles::RED_BUTTON_COLOR)),

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
                        styles::RED_BUTTON_BORDER
                    },
                    width: 2.0,
                    radius: 20.0.into(),
                },
            }
        })
}
