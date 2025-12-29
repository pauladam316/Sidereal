use crate::app::Message as MainMessage;
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::styles::text_input_style::sidereal_readonly_text;
use crate::gui::widgets::indicator::{indicator, IndicatorColor};
use crate::gui::widgets::live_plot::{create_live_plot, live_plot, DataPoint, LivePlotData};
use crate::indi_handler::telescope_controller;
use crate::model::SiderealResult;
use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Color, Element, Length, Task};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    TelemetryUpdate {
        ambient_temp: f64,
        heater1_temp: f64,
        heater2_temp: f64,
        heater3_temp: f64,
        lens_cap_open: bool,
        flat_light_on: bool,
        heater1_on: bool,
        heater2_on: bool,
        heater3_on: bool,
        lens_cap_manual_override: bool,
        flat_light_manual_override: bool,
        heater1_manual_override: bool,
        heater2_manual_override: bool,
        heater3_manual_override: bool,
    },
    LensCapOpen,
    LensCapClose,
    FlatLightOn,
    FlatLightOff,
    Heater1Enable,
    Heater1Disable,
    Heater2Enable,
    Heater2Disable,
    Heater3Enable,
    Heater3Disable,
}

pub struct TelescopeState {
    plot: LivePlotData,
    // Series indices for the plot
    ambient_series: usize,
    heater1_series: usize,
    heater2_series: usize,
    heater3_series: usize,
    start_time: SystemTime,
    // Current telemetry values
    ambient_temp: f64,
    heater1_temp: f64,
    heater2_temp: f64,
    heater3_temp: f64,
    lens_cap_open: bool,
    flat_light_on: bool,
    heater1_on: bool,
    heater2_on: bool,
    heater3_on: bool,
    lens_cap_manual_override: bool,
    flat_light_manual_override: bool,
    heater1_manual_override: bool,
    heater2_manual_override: bool,
    heater3_manual_override: bool,
}

impl Default for TelescopeState {
    fn default() -> Self {
        // 30 minutes of data at ~1 update per second = ~1800 points, use 2000 to be safe
        let mut plot = create_live_plot(2000, 20.0);

        // Add temperature series for telescope telemetry
        // Primary heater (heater1), secondary heater (heater2), and ambient
        let ambient_series = plot.add_series("Ambient", Color::from_rgb(0.3, 0.7, 1.0));
        let heater1_series = plot.add_series("Primary Heater", Color::from_rgb(1.0, 0.3, 0.3));
        let heater2_series = plot.add_series("Secondary Heater", Color::from_rgb(1.0, 0.6, 0.3));
        let heater3_series = plot.add_series("Heater 3", Color::from_rgb(0.3, 1.0, 0.3)); // Keep for compatibility but won't be displayed

        Self {
            plot,
            ambient_series,
            heater1_series,
            heater2_series,
            heater3_series,
            start_time: SystemTime::now(),
            ambient_temp: 0.0,
            heater1_temp: 0.0,
            heater2_temp: 0.0,
            heater3_temp: 0.0,
            lens_cap_open: false,
            flat_light_on: false,
            heater1_on: false,
            heater2_on: false,
            heater3_on: false,
            lens_cap_manual_override: false,
            flat_light_manual_override: false,
            heater1_manual_override: false,
            heater2_manual_override: false,
            heater3_manual_override: false,
        }
    }
}

impl TelescopeState {
    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::Noop => Task::none(),
            Message::TelemetryUpdate {
                ambient_temp,
                heater1_temp,
                heater2_temp,
                heater3_temp,
                lens_cap_open,
                flat_light_on,
                heater1_on,
                heater2_on,
                heater3_on,
                lens_cap_manual_override,
                flat_light_manual_override,
                heater1_manual_override,
                heater2_manual_override,
                heater3_manual_override,
            } => {
                self.ambient_temp = ambient_temp;
                self.heater1_temp = heater1_temp;
                self.heater2_temp = heater2_temp;
                self.heater3_temp = heater3_temp;
                self.lens_cap_open = lens_cap_open;
                self.flat_light_on = flat_light_on;
                self.heater1_on = heater1_on;
                self.heater2_on = heater2_on;
                self.heater3_on = heater3_on;
                self.lens_cap_manual_override = lens_cap_manual_override;
                self.flat_light_manual_override = flat_light_manual_override;
                self.heater1_manual_override = heater1_manual_override;
                self.heater2_manual_override = heater2_manual_override;
                self.heater3_manual_override = heater3_manual_override;

                // Update plot with new temperature data
                // Show last 30 minutes (1800 seconds) of data
                let timestamp = self.start_time.elapsed().unwrap_or_default().as_secs_f64();

                // Add data points for ambient, primary heater, and secondary heater
                self.plot.add_data_point(
                    self.ambient_series,
                    DataPoint {
                        timestamp,
                        value: ambient_temp,
                    },
                );
                self.plot.add_data_point(
                    self.heater1_series,
                    DataPoint {
                        timestamp,
                        value: heater1_temp,
                    },
                );
                self.plot.add_data_point(
                    self.heater2_series,
                    DataPoint {
                        timestamp,
                        value: heater2_temp,
                    },
                );

                Task::none()
            }
            Message::LensCapOpen => Task::perform(
                async { telescope_controller::set_lens_cap(true).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::LensCapClose => Task::perform(
                async { telescope_controller::set_lens_cap(false).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::FlatLightOn => Task::perform(
                async { telescope_controller::set_flat_light(true).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::FlatLightOff => Task::perform(
                async { telescope_controller::set_flat_light(false).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater1Enable => Task::perform(
                async { telescope_controller::set_heater1(true).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater1Disable => Task::perform(
                async { telescope_controller::set_heater1(false).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater2Enable => Task::perform(
                async { telescope_controller::set_heater2(true).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater2Disable => Task::perform(
                async { telescope_controller::set_heater2(false).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater3Enable => Task::perform(
                async { telescope_controller::set_heater3(true).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
            Message::Heater3Disable => Task::perform(
                async { telescope_controller::set_heater3(false).await },
                |result: SiderealResult<()>| match result {
                    Ok(_) => MainMessage::Noop,
                    Err(e) => MainMessage::ErrorOccurred(e),
                },
            ),
        }
    }
    pub fn view(&self) -> Element<'static, Message> {
        let lens_cap_state_text = if self.lens_cap_open { "Open" } else { "Closed" };
        let flat_light_state_text = if self.flat_light_on { "On" } else { "Off" };
        let heater1_state_text = if self.heater1_on { "On" } else { "Off" };
        let heater2_state_text = if self.heater2_on { "On" } else { "Off" };
        let heater3_state_text = if self.heater3_on { "On" } else { "Off" };
        let heater1_status_text = if self.heater1_on {
            "Enabled"
        } else {
            "Disabled"
        };
        let heater2_status_text = if self.heater2_on {
            "Enabled"
        } else {
            "Disabled"
        };
        let heater3_status_text = if self.heater3_on {
            "Enabled"
        } else {
            "Disabled"
        };

        let layout = column![
            content_container(
                column![
                    text("Lens Cap"),
                    row![
                        sidereal_button(
                            container(text("Open"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                            Some(Message::LensCapOpen),
                            true,
                        )
                        .width(Length::Fixed(80.0)),
                        sidereal_button(
                            container(text("Close"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                            Some(Message::LensCapClose),
                            true,
                        )
                        .width(Length::Fixed(80.0)),
                        Space::with_width(Length::Fill),
                        text("Open:"),
                        indicator(if self.lens_cap_open {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                        text("Manual Override:"),
                        indicator(if self.lens_cap_manual_override {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .width(Length::Fill)
                ]
                .spacing(10),
                ContainerLayer::Layer1
            )
            .width(Length::Fill),
            content_container(
                column![
                    text("Flat Light"),
                    row![
                        sidereal_button(
                            container(text("On"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                            Some(Message::FlatLightOn),
                            true,
                        )
                        .width(Length::Fixed(80.0)),
                        sidereal_button(
                            container(text("Off"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                            Some(Message::FlatLightOff),
                            true,
                        )
                        .width(Length::Fixed(80.0)),
                        Space::with_width(Length::Fill),
                        text("On:"),
                        indicator(if self.flat_light_on {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                        text("Manual Override:"),
                        indicator(if self.flat_light_manual_override {
                            IndicatorColor::Green
                        } else {
                            IndicatorColor::Red
                        }),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10)
                    .width(Length::Fill)
                ]
                .spacing(10),
                ContainerLayer::Layer1
            )
            .width(Length::Fill),
            content_container(
                column![
                    text("Heaters"),
                    content_container(
                        column![
                            text("Heater 1"),
                            row![
                                sidereal_button(
                                    container(text("Enable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater1Enable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                sidereal_button(
                                    container(text("Disable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater1Disable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                Space::with_width(Length::Fill),
                                text("Enabled:"),
                                indicator(if self.heater1_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Heating:"),
                                indicator(if self.heater1_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Manual Override:"),
                                indicator(if self.heater1_manual_override {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .width(Length::Fill)
                        ]
                        .spacing(10),
                        ContainerLayer::Layer2
                    ),
                    content_container(
                        column![
                            text("Heater 2"),
                            row![
                                sidereal_button(
                                    container(text("Enable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater2Enable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                sidereal_button(
                                    container(text("Disable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater2Disable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                Space::with_width(Length::Fill),
                                text("Enabled:"),
                                indicator(if self.heater2_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Heating:"),
                                indicator(if self.heater2_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Manual Override:"),
                                indicator(if self.heater2_manual_override {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .width(Length::Fill)
                        ]
                        .spacing(10),
                        ContainerLayer::Layer2
                    ),
                    content_container(
                        column![
                            text("Heater 3"),
                            row![
                                sidereal_button(
                                    container(text("Enable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater3Enable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                sidereal_button(
                                    container(text("Disable"))
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center),
                                    Some(Message::Heater3Disable),
                                    true,
                                )
                                .width(Length::Fixed(80.0)),
                                Space::with_width(Length::Fill),
                                text("Enabled:"),
                                indicator(if self.heater3_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Heating:"),
                                indicator(if self.heater3_on {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                                text("Manual Override:"),
                                indicator(if self.heater3_manual_override {
                                    IndicatorColor::Green
                                } else {
                                    IndicatorColor::Red
                                }),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(10)
                            .width(Length::Fill)
                        ]
                        .spacing(10),
                        ContainerLayer::Layer2
                    ),
                    live_plot(&self.plot)
                        .width(Length::Fill)
                        .height(Length::Fixed(300.0))
                ]
                .spacing(10),
                ContainerLayer::Layer1
            )
            .width(Length::Fill),
        ]
        .spacing(10)
        .into();
        layout
    }
}
