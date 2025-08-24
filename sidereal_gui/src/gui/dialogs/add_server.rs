use iced::widget::{column, row, text};
use iced::{Alignment, Element, Task};

use crate::gui::styles::{button_style::sidereal_button, text_input_style::sidereal_text_input};

#[derive(Debug, Clone)]
pub enum Message {
    IpChanged(String),
    PortChanged(String),
    Cancel,
    Submit { ip: String, port: String },
}

#[derive(Default, Debug, Clone)]
pub struct AddServerDialog {
    ip: String,
    port: String,
}

impl AddServerDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::IpChanged(ip) => {
                self.ip = ip;
            }
            Message::PortChanged(port) => {
                self.port = port;
            }
            _ => {}
        }
        Task::none()
    }

    pub fn view<'a, ParentMessage>(
        &'a self,
        background: impl Into<Element<'a, ParentMessage>> + 'a,
        map: impl Fn(Message) -> ParentMessage + 'a + Clone,
    ) -> Element<'a, ParentMessage>
    where
        ParentMessage: Clone + 'a, // <-- add this bound
    {
        super::dialog::dialog(
            background,
            column![
                column![
                    row![
                        text("IP Address"),
                        sidereal_text_input("127.0.0.1", &self.ip).on_input({
                            let map = map.clone();
                            move |s| map(Message::IpChanged(s))
                        })
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        text("Port"),
                        sidereal_text_input("7624", &self.port).on_input({
                            let map = map.clone();
                            move |s| map(Message::PortChanged(s))
                        })
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(10),
                row![
                    sidereal_button("Add").on_press({
                        let map = map.clone();
                        map(Message::Submit {
                            ip: self.ip.clone(),
                            port: self.port.clone(),
                        })
                    }),
                    sidereal_button("Cancel").on_press({
                        let map = map.clone();
                        map(Message::Cancel)
                    }),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center),
        )
    }
}
