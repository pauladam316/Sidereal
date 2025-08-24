use std::net::IpAddr;

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
    AddServer,
}
#[derive(Debug, Clone)]
pub enum Message {
    SelectServer(String),
    SelectCity(String),
    FieldChanged { field: Field, value: String },
    SetLocation,
    ConnectToServer,
    AddServer { ip: String, port: String },
    Bubble(BubbleMessagePayload),
}

fn combine_ip_port(ip: &str, port: &str) -> SiderealResult<String> {
    let ip = ip.trim();
    let port = port.trim();

    // Validate port
    let port_num: u16 = port
        .parse()
        .map_err(|_| SiderealError::FormatError(format!("Invalid port: `{port}`")))?;
    if port_num == 0 {
        return Err(SiderealError::FormatError(
            "Port must be between 1 and 65535.".into(),
        ));
    }

    // Validate IP (strictly IP; not hostname)
    match ip.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => Ok(format!("{}:{}", v4, port_num)),
        Ok(IpAddr::V6(v6)) => Ok(format!("[{}]:{}", v6, port_num)), // bracket IPv6
        Err(_) => Err(SiderealError::FormatError(format!(
            "Invalid IP address: `{ip}`"
        ))),
    }
}

#[derive(Default)]
pub struct SetupState {
    selected_server_ip: Option<String>,
    server_ip_list: Vec<String>,
    favorite_city: Option<String>,
    pub latitude: String,
    pub longitude: String,
    pub altitude: String,
}
impl SetupState {
    pub fn on_config_load(&mut self, config: Config) -> () {
        self.latitude = config.location.latitude.to_string();
        self.longitude = config.location.longitude.to_string();
        self.altitude = config.location.altitude.to_string();
        self.selected_server_ip = config.server.clone();
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

                planetarium_handler::set_site_location().await?;

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
            Message::SelectServer(server_ip) => {
                self.selected_server_ip = Some(server_ip.to_owned())
            }
            Message::SelectCity(_) => todo!(),
            Message::FieldChanged { field, value } => match field {
                Field::Latitude => self.latitude = value,
                Field::Longitude => self.longitude = value,
                Field::Altitude => self.altitude = value,
            },
            Message::SetLocation {} => return self.set_location(),
            Message::ConnectToServer => {
                let ip = self.selected_server_ip.clone();

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
            Message::AddServer { ip, port } => match combine_ip_port(&ip, &port) {
                Ok(ip) => {
                    self.server_ip_list.push(ip.clone());
                    self.selected_server_ip = Some(ip);
                }
                Err(error) => {
                    return Task::done(MainMessage::ErrorOccurred(error));
                }
            },
        }
        Task::none()
    }

    pub fn view<'a>(&'a self, camera_manager: &'a CameraManager) -> Element<'a, Message> {
        let cities: [String; 1] = ["Arlington, VA".to_owned()];

        let pick = sidereal_picklist(
            self.server_ip_list.clone(),
            self.selected_server_ip.clone(),
            |m| Message::SelectServer(m),
        )
        .placeholder("Select server")
        .width(Length::Fill);

        let location_pick = sidereal_picklist(cities.to_vec(), self.favorite_city.clone(), |m| {
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
                        sidereal_button(text("Add"))
                            .on_press(Message::Bubble(BubbleMessagePayload::AddServer)),
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
