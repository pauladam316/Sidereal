use iced::widget::container::Style;
use iced::widget::{center, opaque};
use iced::widget::{column, container, row, text, Button, Stack};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::gui::styles;

/// Simple error dialog container that overlays `content` with a modal window.
pub fn dialog<'a, Message>(
    show: bool,
    content: impl Into<Element<'a, Message>> + 'a,
    error_text: impl Into<Element<'a, Message>> + 'a,
    dismiss_button: Button<'a, Message>,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let content = content.into();

    if !show {
        return content;
    }

    // Construct the overlay dialog
    let overlay = container(
        column![
            text("Error").size(28),
            error_text.into(),
            row![dismiss_button].spacing(10).align_y(Alignment::Center),
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center),
    )
    .width(Length::from(300))
    .height(Length::Shrink)
    .style(dialog_style)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center);
    let overlay_centered = center(overlay);

    let backdrop = opaque(
        container(column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(background_style),
    );

    // Return a stacked layout: base content + overlay
    Stack::new()
        .push(content)
        .push(backdrop)
        .push(overlay_centered)
        .width(Length::Fill)
        .height(Length::Fill)
        // .style(dialog_style)
        .into()
}

fn dialog_style(_theme: &Theme) -> Style {
    iced::widget::container::Style {
        background: Some(Background::Color(styles::BUTTON_COLOR)),

        text_color: Some(styles::TEXT_COLOR),

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
            color: styles::ELEMENT_BORDER,
            width: 1.0,
            radius: 7.0.into(),
        },
    }
}

fn background_style(_theme: &Theme) -> Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.8))),

        text_color: Some(styles::TEXT_COLOR),

        shadow: iced::Shadow::default(),
        border: Border {
            color: Color::BLACK,
            width: 0.0,
            radius: 0.0.into(),
        },
    }
}
