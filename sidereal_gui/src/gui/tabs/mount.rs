use iced::widget::{checkbox, column, container, image, row, slider, text, Image, Space};
use iced::{Alignment, Element, Length, Subscription, Task};

use crate::app::Message as MainMessage;
use crate::gui::styles::button_style::{sidereal_button, stop_track_button, track_button};
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::styles::text_input_style::sidereal_text_input;
use crate::model::indi_server_handler::param_watcher;
#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    SetSetPoint(f32),
    MoveMount(ButtonDirection),
    StartTracking,
    StopTracking,
    CoordsUpdated { ra_hours: f64, dec_deg: f64 },
    IndiError(String),
}

#[derive(Default)]
pub struct MountState {
    mount_ra: String,
    mount_dec: String,
}

#[derive(Debug, Clone)]
pub enum ButtonDirection {
    N,
    S,
    E,
    W,
    NE,
    SE,
    NW,
    SW,
    Stop,
}
fn controller_button(direction: ButtonDirection) -> Image {
    let icon_bytes: &'static [u8] = match direction {
        ButtonDirection::N => include_bytes!("../../../assets/N.png").as_slice(),
        ButtonDirection::S => include_bytes!("../../../assets/S.png").as_slice(),
        ButtonDirection::E => include_bytes!("../../../assets/E.png").as_slice(),
        ButtonDirection::W => include_bytes!("../../../assets/W.png").as_slice(),
        ButtonDirection::NE => include_bytes!("../../../assets/NE.png").as_slice(),
        ButtonDirection::SE => include_bytes!("../../../assets/SE.png").as_slice(),
        ButtonDirection::NW => include_bytes!("../../../assets/NW.png").as_slice(),
        ButtonDirection::SW => include_bytes!("../../../assets/SW.png").as_slice(),
        ButtonDirection::Stop => include_bytes!("../../../assets/stop.png").as_slice(),
    };

    let handle = image::Handle::from_bytes(icon_bytes);

    let img = image(handle)
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0));
    return img;
}

pub fn coords_subscription() -> Subscription<Message> {
    Subscription::run_with_id("coords_subscription", param_watcher())
}

impl MountState {
    pub fn subscription(&self) -> iced::Subscription<Message> {
        coords_subscription()
    }
    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::Noop => todo!(),
            Message::SetSetPoint(_) => todo!(),
            Message::MoveMount(_) => todo!(),
            Message::StartTracking => todo!(),
            Message::StopTracking => todo!(),
            Message::CoordsUpdated { ra_hours, dec_deg } => {
                self.mount_ra = ra_hours.to_string();
                self.mount_dec = dec_deg.to_string();
            }
            Message::IndiError(_) => todo!(),
        }
        Task::none()
    }
    pub fn view(&self) -> Element<'static, Message> {
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
                        sidereal_button(controller_button(ButtonDirection::NW))
                            .padding(10)
                            .on_press(Message::MoveMount(ButtonDirection::NW)),
                        sidereal_button(controller_button(ButtonDirection::N)).padding(10),
                        sidereal_button(controller_button(ButtonDirection::NE)).padding(10),
                    ]
                    .spacing(3),
                    row![
                        sidereal_button(controller_button(ButtonDirection::W)).padding(10),
                        sidereal_button(controller_button(ButtonDirection::Stop)).padding(10),
                        sidereal_button(controller_button(ButtonDirection::E)).padding(10),
                    ]
                    .spacing(3),
                    row![
                        sidereal_button(controller_button(ButtonDirection::SW)).padding(10),
                        sidereal_button(controller_button(ButtonDirection::S)).padding(10),
                        sidereal_button(controller_button(ButtonDirection::SE)).padding(10),
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
