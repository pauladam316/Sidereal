use std::sync::Arc;

use crate::{
    styles::{tab_style::tab_content, SIDEREAL_THEME},
    tabs::{self, MainWindowState, Tab},
};
use iced::{Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    Tab(Tab),
    Setup(tabs::setup::Message),
    Mount(tabs::mount::Message),
    Observatory(tabs::observatory::Message),
    PlateSolve(tabs::plate_solve::Message),
    Capture(tabs::capture::Message),
    Focus(tabs::focus::Message),
    Guide(tabs::guide::Message),
}

#[derive(Default)]
pub struct MainWindow {
    state: MainWindowState,
}

impl MainWindow {
    pub fn run(settings: iced::Settings) -> iced::Result {
        iced::application("Sidereal GUI", Self::update, Self::view)
            .theme(|_| SIDEREAL_THEME.clone())
            .settings(settings)
            .run()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tab(tab) => {
                self.state.active = tab;
            }
            Message::Setup(msg) => {
                self.state.setup.update(msg);
            }
            Message::Mount(msg) => {
                self.state.mount.update(msg);
            }
            Message::Observatory(msg) => {
                self.state.observatory.update(msg);
            }
            Message::PlateSolve(msg) => {
                self.state.plate_solve.update(msg);
            }
            Message::Guide(msg) => {
                self.state.guide.update(msg);
            }
            Message::Focus(msg) => {
                self.state.focus.update(msg);
            }
            Message::Capture(msg) => {
                self.state.capture.update(msg);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let header = tabs::header(self.state.active, |t| Message::Tab(t));

        let inner_content: Element<_> = match self.state.active {
            Tab::Setup => self.state.setup.view().map(Message::Setup),
            Tab::Mount => self.state.mount.view().map(Message::Mount),
            Tab::Observatory => self.state.observatory.view().map(Message::Observatory),
            Tab::PlateSolve => self.state.plate_solve.view().map(Message::PlateSolve),
            Tab::Guide => self.state.guide.view().map(Message::Guide),
            Tab::Focus => self.state.focus.view().map(Message::Focus),
            Tab::Capture => self.state.capture.view().map(Message::Capture),
        };

        let content = tab_content(inner_content)
            .padding([30, 10])
            .width(Length::Fill)
            .height(Length::Fill);

        iced::widget::column![header, content]
            .spacing(-1.0)
            .padding(10)
            .into()
    }
}
