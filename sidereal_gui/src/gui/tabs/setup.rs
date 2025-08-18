use iced::widget::{column, row, text};
use iced::{Alignment, Element, Length, Task};

use crate::app::Message as MainMessage;
use crate::config::Config;
use crate::gui::camera_display::{CameraManager, CameraMessage};
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::styles::picklist_style::sidereal_picklist;
use crate::gui::styles::text_input_style::sidereal_text_input;
use crate::gui::widgets::server_status::ServerStatus;

use crate::model::{indi_server_handler, planetarium_handler, SiderealError, SiderealResult};

#[derive(Debug, Clone)]
pub enum Field {
    Latitude,
    Longitude,
    Altitude,
}

//bubbled messages are ones emitted by the setup tab that are to be handled by the main app
#[derive(Debug, Clone)]
pub enum BubbleMessagePayload {
    Camera(CameraMessage),
}
#[derive(Debug, Clone)]
pub enum Message {
    SelectServer(&'static str),
    SelectCity(&'static str),
    FieldChanged { field: Field, value: String },
    SetLocation,
    ConnectToServer,
    Bubble(BubbleMessagePayload),
}

#[derive(Default)]
pub struct SetupState {
    server_ip: Option<&'static str>,
    favorite_city: Option<&'static str>,
    pub latitude: String,
    pub longitude: String,
    pub altitude: String,
}
impl SetupState {
    pub fn on_config_load(&mut self, config: Config) -> () {
        self.latitude = config.location.latitude.to_string();
        self.longitude = config.location.longitude.to_string();
        self.altitude = config.location.altitude.to_string();
    }

    pub fn set_location(&mut self) -> Task<MainMessage> {
        // Clone the strings outside the async block
        let latitude = self.latitude.clone();
        let longitude = self.longitude.clone();
        let altitude = self.altitude.clone();

        Task::perform(
            async move {
                let lat = latitude
                    .parse::<f32>()
                    .map_err(|_| SiderealError::ParseError("Invalid latitude".to_string()))?;
                let lon = longitude
                    .parse::<f32>()
                    .map_err(|_| SiderealError::ParseError("Invalid longitude".to_string()))?;
                let alt = altitude
                    .parse::<f32>()
                    .map_err(|_| SiderealError::ParseError("Invalid altitude".to_string()))?;

                crate::config::Config::set_location(lat, lon, alt).await?;

                planetarium_handler::set_location().await?;

                Ok(())
            },
            |result: SiderealResult<()>| match result {
                Ok(()) => MainMessage::Noop,
                Err(e) => MainMessage::ErrorOccurred(SiderealError::ConfigError(e.to_string())),
            },
        )
    }

    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::SelectServer(server_ip) => self.server_ip = Some(server_ip),
            Message::SelectCity(_) => todo!(),
            Message::FieldChanged { field, value } => match field {
                Field::Latitude => self.latitude = value,
                Field::Longitude => self.longitude = value,
                Field::Altitude => self.altitude = value,
            },
            Message::SetLocation {} => return self.set_location(),
            Message::ConnectToServer => {
                let ip = self.server_ip.clone();

                let announce_connecting =
                    Task::done(MainMessage::ServerStatus(ServerStatus::Connecting));

                let do_connect = Task::perform(
                    async move {
                        indi_server_handler::connect_to_server(ip.ok_or("No server IP selected")?)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| match result {
                        Ok(_) => MainMessage::ServerStatus(ServerStatus::Connected),
                        Err(e) => MainMessage::ErrorOccurred(SiderealError::ServerConnectionError(
                            e.to_string(),
                        )),
                    },
                );

                return Task::batch(vec![announce_connecting, do_connect]);
            }
            Message::Bubble(_) => {}
        }
        Task::none()
    }

    pub fn view<'a>(&'a self, camera_manager: &'a CameraManager) -> Element<'a, Message> {
        let server_ips: [&'static str; 2] = ["127.0.0.1:7624", "test"];
        let cities: [&'static str; 1] = ["Arlington, VA"];

        let pick = sidereal_picklist(server_ips.to_vec(), self.server_ip, |m| {
            Message::SelectServer(m)
        })
        .placeholder("Select server")
        .width(Length::Fill);

        let location_pick = sidereal_picklist(cities.to_vec(), self.favorite_city, |m| {
            Message::SelectCity(m)
        })
        .placeholder("Select city")
        .width(Length::Fill);

        let layout =
            column![
                content_container(
                    row![
                        text("Server"),
                        pick,
                        sidereal_button(text("Add")).on_press(Message::SelectServer("placeholder")),
                        sidereal_button(text("Connect")).on_press(Message::ConnectToServer)
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                    ContainerLayer::Layer1,
                )
                .padding(10),
                content_container(
                    column![
                        text("Site Setup"),
                        row![text("Location"), location_pick,]
                            .align_y(Alignment::Center)
                            .spacing(10),
                        row![
                            text("Latitude"),
                            sidereal_text_input("latitude", &self.latitude).on_input(|v| {
                                Message::FieldChanged {
                                    field: Field::Latitude,
                                    value: v,
                                }
                            }),
                            text("Longitude"),
                            sidereal_text_input("longitude", &self.longitude).on_input(|v| {
                                Message::FieldChanged {
                                    field: Field::Longitude,
                                    value: v,
                                }
                            }),
                            text("Altitude"),
                            sidereal_text_input("altitude", &self.altitude).on_input(|v| {
                                Message::FieldChanged {
                                    field: Field::Altitude,
                                    value: v,
                                }
                            }),
                            sidereal_button("Apply").on_press(Message::SetLocation)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(10),
                    ]
                    .spacing(10),
                    ContainerLayer::Layer1
                )
                .padding(10),
                content_container(
                    column![
                        text("Cameras"),
                        camera_manager
                            .view_camera_setup()
                            .map(|m| Message::Bubble(BubbleMessagePayload::Camera(m))),
                        sidereal_button("Add Camera").width(Length::Fill).on_press(
                            Message::Bubble(BubbleMessagePayload::Camera(CameraMessage::AddCamera))
                        )
                    ]
                    .spacing(10),
                    ContainerLayer::Layer1
                )
            ]
            .spacing(10);
        layout.into()
    }
}
