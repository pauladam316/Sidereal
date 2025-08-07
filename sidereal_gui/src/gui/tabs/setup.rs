use iced::widget::{column, row, text};
use iced::{Alignment, Element, Length, Task};

use crate::app::Message as MainMessage;
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::content_container;
use crate::gui::styles::picklist_style::sidereal_picklist;
use crate::gui::styles::text_input_style::sidereal_text_input;
use crate::gui::widgets::dialog::dialog;

#[derive(Debug, Clone)]
pub enum Field {
    Latitude,
    Longitude,
    Altitude,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectServer(&'static str),
    SelectCity(&'static str),
    FieldChanged { field: Field, value: String },
    SetLocation,
}

#[derive(Default)]
pub struct SetupState {
    favorite: Option<&'static str>,
    favorite_city: Option<&'static str>,
    pub latitude: String,
    pub longitude: String,
    pub altitude: String,
    error_message: Option<String>,
}
impl SetupState {
    pub fn on_config_load(&mut self) -> () {
        let guard = crate::config::GLOBAL_CONFIG.read().unwrap();
        self.latitude = (&guard.location.latitude).to_string();
        self.longitude = (&guard.location.longitude).to_string();
        self.altitude = (&guard.location.altitude).to_string();
    }

    pub fn update(&mut self, message: Message) -> Task<MainMessage> {
        match message {
            Message::SelectServer(_) => todo!(),
            Message::SelectCity(_) => todo!(),
            Message::FieldChanged { field, value } => match field {
                Field::Latitude => self.latitude = value,
                Field::Longitude => self.longitude = value,
                Field::Altitude => self.altitude = value,
            },
            Message::SetLocation {} => {
                if let Err(msg) = self
                    .latitude
                    .parse::<f32>()
                    .map_err(|_| "Invalid latitude")
                    .and_then(|lat| {
                        self.longitude
                            .parse::<f32>()
                            .map_err(|_| "Invalid longitude")
                            .and_then(|lon| {
                                self.altitude
                                    .parse::<f32>()
                                    .map_err(|_| "Invalid altitude")
                                    .and_then(|alt| {
                                        self.error_message = None;
                                        crate::config::Config::set_location(lat, lon, alt)
                                            .map_err(|_| "Failed to write location to config")
                                    })
                            })
                    })
                {
                    return Task::perform(
                        async move { msg.to_string() },
                        MainMessage::ErrorOccurred,
                    );
                }
            }
        }
        Task::none()
    }
    pub fn view(&self) -> Element<Message> {
        let server_ips: [&'static str; 1] = ["192.168.5.1"];
        let cities: [&'static str; 1] = ["Arlington, VA"];

        let pick = sidereal_picklist(server_ips.to_vec(), self.favorite, |m| {
            Message::SelectServer(m)
        })
        .placeholder("Select server")
        .width(Length::Fill);

        let location_pick = sidereal_picklist(cities.to_vec(), self.favorite_city, |m| {
            Message::SelectCity(m)
        })
        .placeholder("Select city")
        .width(Length::Fill);

        let mut layout = column![
            content_container(
                row![
                    text("Server"),
                    pick,
                    sidereal_button(text("Add")).on_press(Message::SelectServer("placeholder")),
                    sidereal_button(text("Connect")).on_press(Message::SelectServer("placeholder"))
                ]
                .align_y(Alignment::Center)
                .spacing(10),
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
                .spacing(10)
            )
            .padding(10)
        ]
        .spacing(10);
        layout.into()
    }
}
