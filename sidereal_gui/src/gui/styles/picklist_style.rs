use iced::{
    widget::{pick_list, pick_list::Status, PickList},
    Background, Border, Length, Theme,
};
use std::fmt::Display;

use crate::gui::styles;

pub fn sidereal_picklist<'a, T, Message>(
    items: Vec<T>, // owned
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a + Clone,
) -> PickList<'a, T, Vec<T>, T, Message, Theme>
where
    T: Clone + Display + PartialEq + 'a,
    Message: 'a + Clone,
{
    pick_list(items, selected.clone(), on_select)
        .padding([6, 12])
        .width(Length::Fill)
        .style(move |_: &Theme, status: Status| {
            let hovered = matches!(status, Status::Hovered);
            let is_highlighted = hovered;

            iced::widget::pick_list::Style {
                background: Background::Color(styles::BUTTON_COLOR),
                text_color: styles::TEXT_COLOR,
                border: Border {
                    color: if is_highlighted {
                        styles::ACCENT_COLOR
                    } else {
                        styles::ELEMENT_BORDER
                    },
                    width: 1.0,
                    radius: 7.0.into(),
                },
                placeholder_color: styles::BACKGROUND_TEXT_COLOR,
                handle_color: if is_highlighted {
                    styles::ACCENT_COLOR
                } else {
                    styles::TEXT_COLOR
                },
            }
        })
}
