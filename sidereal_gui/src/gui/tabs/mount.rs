use iced::widget::{checkbox, column, container, row, slider, text, Space};
use iced::{Alignment, Element, Length, Task};

use crate::app::Message as MainMessage;
use crate::gui::styles::button_style::{sidereal_button, stop_track_button, track_button};
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::styles::text_input_style::sidereal_text_input;
use crate::gui::widgets::mount_steer_button::{
    ButtonDirection, MountMoveMessage, MountSteerButton,
};
use crate::planetarium_handler::planetarium_sender;
#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    SetSetPoint(f32),
    StartTracking,
    StopTracking,
    CoordsUpdated {
        ra_hours: f64,
        dec_deg: f64,
    },
    MountMove {
        index: usize,
        message: MountMoveMessage,
    },
}

pub struct MountState {
    mount_ra: String,
    mount_dec: String,
    mount_steer_buttons: Vec<MountSteerButton>,
}

impl Default for MountState {
    fn default() -> Self {
        Self {
            mount_ra: Default::default(),
            mount_dec: Default::default(),
            mount_steer_buttons: (0..9).map(|_| MountSteerButton::default()).collect(),
        }
    }
}

impl MountState {
    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::Noop => {}
            Message::SetSetPoint(_) => todo!(),
            Message::StartTracking => todo!(),
            Message::StopTracking => todo!(),
            Message::CoordsUpdated { ra_hours, dec_deg } => {
                self.mount_ra = ra_hours.to_string();
                self.mount_dec = dec_deg.to_string();
                return Task::perform(
                    async move {
                        planetarium_sender::set_mount_position(ra_hours as f32, dec_deg as f32)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| match result {
                        Ok(_) => MainMessage::Noop, // or whatever message you want on success
                        Err(e) => {
                            println!("failed to send mount position to planetarium: {}", e);
                            MainMessage::Noop
                        }
                    },
                );
            }
            Message::MountMove { index, message } => {
                return self.mount_steer_buttons[index].update(message);
            }
        }
        Task::none()
    }
    pub fn view(&self) -> Element<Message> {
        let layout = row![
            column![
                row![content_container(
                    column![
                        text("Status"),
                        content_container(
                            column![
                                row![
                                    text("Mount Status:"),
                                    content_container(row![text("ONLINE")], ContainerLayer::Layer3),
                                    Space::with_width(Length::Fill),
                                    text("Mount State:"),
                                    content_container(
                                        row![text("SLEWING")],
                                        ContainerLayer::Layer3
                                    ),
                                    Space::with_width(Length::Fill),
                                ]
                                .align_y(Alignment::Center)
                                .spacing(30)
                                .width(Length::Fill),
                                content_container(
                                    column![
                                        text("Position"),
                                        row![
                                            text("RA:"),
                                            sidereal_text_input("TEST", &self.mount_ra)
                                                .width(Length::Fill),
                                            text("DEC:"),
                                            sidereal_text_input("TEST", &self.mount_dec)
                                                .width(Length::Fill)
                                        ]
                                        .align_y(Alignment::Center)
                                        .spacing(10)
                                        .width(Length::Fill)
                                    ],
                                    ContainerLayer::Layer3
                                )
                            ]
                            .spacing(10)
                            .padding([5, 1]),
                            ContainerLayer::Layer2
                        )
                    ]
                    .spacing(10),
                    ContainerLayer::Layer1
                ),]
                .height(Length::Shrink),
                content_container(
                    column![
                        text("Tracking"),
                        row![content_container(
                            column![
                                text("Tracking Settings"),
                                row![
                                    checkbox("Leapfrog Target", false).width(Length::Fixed(90.0)),
                                    content_container(
                                        row![
                                            text("Distance"),
                                            sidereal_text_input("TEST", "TEST").width(Length::Fill)
                                        ]
                                        .spacing(10)
                                        .align_y(Alignment::Center),
                                        ContainerLayer::Layer3
                                    )
                                    .width(Length::Fill),
                                    checkbox("Pause at Horizon", false).width(Length::Fixed(90.0)),
                                    content_container(
                                        row![
                                            text("Distance"),
                                            sidereal_text_input("TEST", "TEST").width(Length::Fill)
                                        ]
                                        .spacing(10)
                                        .align_y(Alignment::Center),
                                        ContainerLayer::Layer3
                                    )
                                ]
                                .spacing(10)
                                .align_y(Alignment::Center),
                                content_container(
                                    column![
                                        text("Target"),
                                        row![
                                            text("RA:"),
                                            sidereal_text_input("TEST", "TEST").width(Length::Fill),
                                            text("DEC:"),
                                            sidereal_text_input("TEST", "TEST").width(Length::Fill)
                                        ]
                                        .align_y(Alignment::Center)
                                        .spacing(10)
                                        .width(Length::Fill)
                                    ]
                                    .spacing(10),
                                    ContainerLayer::Layer3
                                ),
                            ]
                            .spacing(10)
                            .padding([5, 1]),
                            ContainerLayer::Layer2
                        )
                        .width(Length::FillPortion(3))]
                        .spacing(10)
                        .padding([10, 1]),
                        row![
                            track_button(
                                container(text("Track Target"))
                                    .width(Length::Fill)
                                    .align_x(Alignment::Center)
                                    .align_y(Alignment::Center)
                            )
                            .width(Length::Fill)
                            .on_press(Message::StartTracking),
                            stop_track_button(
                                container(text("Abort Tracking"))
                                    .width(Length::Fill)
                                    .align_x(Alignment::Center)
                                    .align_y(Alignment::Center)
                            )
                            .width(Length::Fill)
                            .on_press(Message::StartTracking),
                        ]
                        .spacing(10)
                    ],
                    ContainerLayer::Layer1
                )
                .height(Length::Shrink)
            ]
            .spacing(10),
            content_container(
                column![
                    text("Manual Slew"),
                    row![
                        self.mount_steer_buttons[0]
                            .view(ButtonDirection::NW)
                            .map(|m| Message::MountMove {
                                index: 0,
                                message: m
                            }),
                        self.mount_steer_buttons[1]
                            .view(ButtonDirection::N)
                            .map(|m| Message::MountMove {
                                index: 1,
                                message: m
                            }),
                        self.mount_steer_buttons[2]
                            .view(ButtonDirection::NE)
                            .map(|m| Message::MountMove {
                                index: 2,
                                message: m
                            }),
                    ]
                    .spacing(3),
                    row![
                        self.mount_steer_buttons[3]
                            .view(ButtonDirection::W)
                            .map(|m| Message::MountMove {
                                index: 3,
                                message: m
                            }),
                        self.mount_steer_buttons[4]
                            .view(ButtonDirection::Stop)
                            .map(|m| Message::MountMove {
                                index: 4,
                                message: m
                            }),
                        self.mount_steer_buttons[5]
                            .view(ButtonDirection::E)
                            .map(|m| Message::MountMove {
                                index: 5,
                                message: m
                            }),
                    ]
                    .spacing(3),
                    row![
                        self.mount_steer_buttons[6]
                            .view(ButtonDirection::SW)
                            .map(|m| Message::MountMove {
                                index: 6,
                                message: m
                            }),
                        self.mount_steer_buttons[7]
                            .view(ButtonDirection::S)
                            .map(|m| Message::MountMove {
                                index: 7,
                                message: m
                            }),
                        self.mount_steer_buttons[8]
                            .view(ButtonDirection::SE)
                            .map(|m| Message::MountMove {
                                index: 8,
                                message: m
                            }),
                    ]
                    .spacing(3),
                    column![
                        text("Movement Speed"),
                        slider(0.0..=1.0, 0.5, |_| Message::Noop).width(Length::Fill),
                        sidereal_button(
                            container(text("Park Scope"))
                                .width(Length::Fill)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                        ),
                        sidereal_button(
                            container(text("Unpark Scope"))
                                .width(Length::Fill)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                        )
                    ]
                    .spacing(10)
                ]
                .spacing(3)
                .width(Length::Shrink),
                ContainerLayer::Layer1
            )
            .padding(10)
        ]
        .spacing(10);

        layout.into()
    }
}
