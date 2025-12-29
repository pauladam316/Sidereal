use crate::app::Message as MainMessage;
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::styles::text_input_style::sidereal_readonly_text;
use crate::gui::widgets::indicator::{indicator, IndicatorColor};
use crate::indi_handler::roof_controller;
use crate::model::SiderealResult;
use iced::widget::{column, row, text, Space};
use iced::{Alignment, Element, Length, Task};

const BUTTON_WIDTH: f32 = 120.0;

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    TelemetryUpdate {
        is_armed: bool,
        roof_is_open: bool,
        roof_is_closed: bool,
        roof_position: f64,
        lock_engaged: bool,
        voltage_5v: f64,
        voltage_12v: f64,
        actuator_current: f64,
        limit_u1: bool,
        limit_u2: bool,
        limit_l1: bool,
        limit_l2: bool,
    },
    ArmSystem,
    DisarmSystem,
    OpenRoof,
    CloseRoof,
    StopRoof,
    EngageLock,
    DisengageLock,
    StopLock,
}

#[derive(Default)]
pub struct ObservatoryState {
    is_armed: bool,
    roof_is_open: bool,
    roof_is_closed: bool,
    roof_position: f64,
    lock_engaged: bool,
    voltage_5v: f64,
    voltage_12v: f64,
    actuator_current: f64,
    limit_u1: bool,
    limit_u2: bool,
    limit_l1: bool,
    limit_l2: bool,
}

impl ObservatoryState {
    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::TelemetryUpdate {
                is_armed,
                roof_is_open,
                roof_is_closed,
                roof_position,
                lock_engaged,
                voltage_5v,
                voltage_12v,
                actuator_current,
                limit_u1,
                limit_u2,
                limit_l1,
                limit_l2,
            } => {
                self.is_armed = is_armed;
                self.roof_is_open = roof_is_open;
                self.roof_is_closed = roof_is_closed;
                self.roof_position = roof_position;
                self.lock_engaged = lock_engaged;
                self.voltage_5v = voltage_5v;
                self.voltage_12v = voltage_12v;
                self.actuator_current = actuator_current;
                self.limit_u1 = limit_u1;
                self.limit_u2 = limit_u2;
                self.limit_l1 = limit_l1;
                self.limit_l2 = limit_l2;
                Task::none()
            }
            Message::ArmSystem => Task::perform(
                async { roof_controller::arm_system().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::DisarmSystem => Task::perform(
                async { roof_controller::disarm_system().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::OpenRoof => Task::perform(
                async { roof_controller::open_roof().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::CloseRoof => Task::perform(
                async { roof_controller::close_roof().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::StopRoof => Task::perform(
                async { roof_controller::stop_roof().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::EngageLock => Task::perform(
                async { roof_controller::engage_lock().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::DisengageLock => Task::perform(
                async { roof_controller::disengage_lock().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::StopLock => Task::perform(
                async { roof_controller::stop_lock().await },
                |result: SiderealResult<()>| {
                    if let Err(e) = result {
                        MainMessage::ErrorOccurred(e)
                    } else {
                        MainMessage::Noop
                    }
                },
            ),
            Message::Noop => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'static, Message> {
        let buttons_enabled = self.is_armed;

        // Create buttons outside the macro to avoid temporary value issues
        let arm_btn = sidereal_button(text("Arm System"), Some(Message::ArmSystem), true)
            .width(Length::Fixed(BUTTON_WIDTH));
        let disarm_btn = sidereal_button(text("Disarm System"), Some(Message::DisarmSystem), true)
            .width(Length::Fixed(BUTTON_WIDTH));

        let engage_lock_btn = sidereal_button(
            text("Engage Lock"),
            Some(Message::EngageLock),
            buttons_enabled,
        )
        .width(Length::Fixed(BUTTON_WIDTH));
        let stop_lock_btn = sidereal_button(text("Stop"), Some(Message::StopLock), true)
            .width(Length::Fixed(BUTTON_WIDTH));
        let disengage_lock_btn = sidereal_button(
            text("Disengage Lock"),
            Some(Message::DisengageLock),
            buttons_enabled,
        )
        .width(Length::Fixed(BUTTON_WIDTH));
        let open_roof_btn =
            sidereal_button(text("Open Roof"), Some(Message::OpenRoof), buttons_enabled)
                .width(Length::Fixed(BUTTON_WIDTH));
        let stop_roof_btn = sidereal_button(text("Stop"), Some(Message::StopRoof), true)
            .width(Length::Fixed(BUTTON_WIDTH));
        let close_roof_btn = sidereal_button(
            text("Close Roof"),
            Some(Message::CloseRoof),
            buttons_enabled,
        )
        .width(Length::Fixed(BUTTON_WIDTH));

        column![content_container(
            column![
                text("Roof Control"),
                content_container(
                    row![
                        arm_btn,
                        disarm_btn,
                        Space::with_width(Length::Fill),
                        text("System Armed:"),
                        indicator(if self.is_armed {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    ContainerLayer::Layer2
                ),
                content_container(
                    row![
                        engage_lock_btn,
                        stop_lock_btn,
                        disengage_lock_btn,
                        Space::with_width(Length::Fill),
                        text("Lock Engaged:"),
                        indicator(if self.lock_engaged {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    ContainerLayer::Layer2
                ),
                content_container(
                    row![
                        open_roof_btn,
                        stop_roof_btn,
                        close_roof_btn,
                        Space::with_width(Length::Fill),
                        text("Roof Open:"),
                        indicator(if self.roof_is_open {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    ContainerLayer::Layer2
                ),
                content_container(
                    column![
                        text("Debug"),
                        row![
                            text("Upper Limit Switches:"),
                            content_container(
                                row![
                                    text("Left:"),
                                    indicator(if self.limit_u1 {
                                        IndicatorColor::Green
                                    } else {
                                        IndicatorColor::Red
                                    }),
                                    text("Right:"),
                                    indicator(if self.limit_u2 {
                                        IndicatorColor::Green
                                    } else {
                                        IndicatorColor::Red
                                    }),
                                ]
                                .spacing(10)
                                .align_y(Alignment::Center),
                                ContainerLayer::Layer3
                            ),
                            Space::with_width(Length::Fill),
                            text("Lower Limit Switches:"),
                            content_container(
                                row![
                                    text("Left:"),
                                    indicator(if self.limit_l1 {
                                        IndicatorColor::Green
                                    } else {
                                        IndicatorColor::Red
                                    }),
                                    text("Right:"),
                                    indicator(if self.limit_l2 {
                                        IndicatorColor::Green
                                    } else {
                                        IndicatorColor::Red
                                    }),
                                ]
                                .spacing(10)
                                .align_y(Alignment::Center),
                                ContainerLayer::Layer3
                            ),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                        row![
                            text("5V:"),
                            text(format!("{:.2}", self.voltage_5v)).width(Length::Fill),
                            text("12V:"),
                            text(format!("{:.2}", self.voltage_12v)).width(Length::Fill),
                            text("Actuator Current (A):"),
                            text(format!("{:.2}", self.actuator_current)).width(Length::Fill),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .width(Length::Fill),
                    ]
                    .spacing(10),
                    ContainerLayer::Layer2
                ),
            ]
            .spacing(10),
            ContainerLayer::Layer1
        )
        .width(Length::Fill)]
        .into()
    }
}
