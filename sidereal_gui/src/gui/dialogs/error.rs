use iced::{Alignment, Element};

use crate::gui::{dialogs::dialog::dialog, styles::button_style::sidereal_button};
use iced::widget::{column, row, text};
pub fn error_dialog<'a, Message>(
    background_content: impl Into<Element<'a, Message>> + 'a,
    error_string: String,
    on_clear: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    dialog(
        background_content,
        column![
            text("Error").size(28),
            text(error_string),
            row![sidereal_button("Dismiss", Some(on_clear), true)]
                .spacing(10)
                .align_y(Alignment::Center),
        ]
        .spacing(20)
        .padding(20)
        .align_x(Alignment::Center),
    )
}
