use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::styles::button_style::sidereal_button;
use crate::styles::container_style::content_container;
use crate::styles::picklist_style::sidereal_picklist;
use crate::styles::text_input_style::sidereal_text_input;

#[derive(Debug, Clone)]
pub enum Message {
    SelectServer(&'static str),
    SelectCity(&'static str),
    LatitudeChanged(String),
    LongitudeChanged(String),
    AltitudeChanged(String),
}

#[derive(Default)]
pub struct SetupState {
    favorite: Option<&'static str>,
    favorite_city: Option<&'static str>,
    pub latitude: String,
    pub longitude: String,
    pub altitude: String,
}
impl SetupState {
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'static, Message> {
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
        column![
            content_container(
                row![
                    text("Server"),
                    pick,
                    sidereal_button(text("Add")).on_press(Message::SelectServer("placeholder")),
                    sidereal_button(text("Connect")).on_press(Message::SelectServer("placeholder"))
                ]
                .align_y(Alignment::Center) // aligns children vertically
                .spacing(10),
            ) // centers the whole row vertically within the container
            .padding(10),
            content_container(
                column![
                    text("Site Setup"),
                    row![text("Location"), location_pick,]
                        .align_y(Alignment::Center)
                        .spacing(10),
                    row![
                        text("Latitude"),
                        sidereal_text_input("latitude", "test").on_input(Message::LatitudeChanged),
                        text("Longitude"),
                        sidereal_text_input("longitude", "test")
                            .on_input(Message::LongitudeChanged),
                        text("Altitude"),
                        sidereal_text_input("altitude", "test").on_input(Message::AltitudeChanged)
                    ]
                    .align_y(Alignment::Center)
                    .spacing(10),
                ]
                .spacing(10)
            ) // centers the whole row vertically within the container
            .padding(10)
        ]
        .spacing(10)
        .into()
    }
}
