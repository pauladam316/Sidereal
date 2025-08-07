use crate::gui::styles::button_style::sidereal_button;
use crate::gui::widgets::dialog::dialog;
use crate::model::planetarium_handler;
use crate::{
    config::Config,
    gui::{
        styles::{tab_style::tab_content, SIDEREAL_THEME},
        tabs::{self, MainWindowState, Tab},
    },
};
use iced::widget::{column, row};
use iced::widget::{container, image};
use iced::Alignment::{self};
use iced::ContentFit;
use iced::{widget::text, Element, Length, Task};

#[derive(Debug, Clone)]
pub enum Message {
    Noop,
    Tab(Tab),
    Setup(tabs::setup::Message),
    Mount(tabs::mount::Message),
    Observatory(tabs::observatory::Message),
    PlateSolve(tabs::plate_solve::Message),
    Capture(tabs::capture::Message),
    Focus(tabs::focus::Message),
    Guide(tabs::guide::Message),
    ConfigLoaded(Config),
    ErrorOccurred(String),
    ErrorCleared(),
    LaunchPlanetarium,
}

#[derive(Default)]
pub struct MainWindow {
    state: MainWindowState,
    error_message: Option<String>,
}

impl MainWindow {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self::default();

        let config_load_task = Task::perform(
            async {
                Config::initialize().await.expect("Failed to load config");
                Ok::<_, String>(Config::get().await)
            },
            |config| Message::ConfigLoaded(config.expect("failed to get config")),
        );
        (app, config_load_task)
    }

    pub fn run(settings: iced::Settings) -> iced::Result {
        let result = iced::application("Sidereal GUI", Self::update, Self::view)
            .theme(|_| SIDEREAL_THEME.clone())
            .settings(settings)
            .window_size(iced::Size::new(1200.0, 900.0))
            .run_with(Self::new);

        result
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tab(tab) => {
                self.state.active = tab;
            }
            Message::Setup(msg) => {
                return self.state.setup.update(msg);
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
            Message::ConfigLoaded(config) => {
                self.state.setup.on_config_load(config);
            }
            Message::ErrorOccurred(err) => {
                self.error_message = Some(err);
            }
            Message::ErrorCleared() => self.error_message = None,
            Message::LaunchPlanetarium => {
                return Task::perform(
                    async {
                        planetarium_handler::launch_planetarium()
                            .await
                            .map_err(|e| e.to_string())?;
                        planetarium_handler::set_location()
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| match result {
                        Ok(_) => Message::Noop, // or whatever message you want on success
                        Err(e) => Message::ErrorOccurred(e),
                    },
                );
            }
            Message::Noop => {}
        }
        Task::none()
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

        let layout = row![
            column![
                image("assets/placeholder.png")
                    .height(Length::Shrink)
                    .content_fit(ContentFit::Contain),
                image("assets/placeholder.png")
                    .height(Length::Shrink)
                    .content_fit(ContentFit::Contain),
                sidereal_button(
                    container(text("Launch Planetarium"))
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                )
                .width(Length::Fill)
                .on_press(Message::LaunchPlanetarium)
            ]
            .width(Length::FillPortion(1))
            .spacing(10),
            column![header, content]
                .spacing(-1.0)
                .width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .padding(10);

        // Wrap in dialog if there's an error
        let view = dialog(
            self.error_message.is_some(),
            layout,
            text(
                self.error_message
                    .as_deref()
                    .unwrap_or("An unknown error occurred"),
            ),
            sidereal_button("Dismiss").on_press(Message::ErrorCleared()),
        );
        view.into()
    }
}
