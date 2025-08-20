use crate::gui::camera_display::{CameraManager, CameraMessage};
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::tabs::setup::BubbleMessagePayload;
use crate::gui::widgets::dialog::dialog;
use crate::gui::widgets::server_status::{server_status_widget, ServerStatus};
use crate::model::indi_server_handler::param_watcher;
use crate::model::{planetarium_handler, SiderealError};
use crate::{
    config::Config,
    gui::{
        styles::{tab_style::tab_content, SIDEREAL_THEME},
        tabs::{self, MainWindowState, Tab},
    },
};
use iced::widget::container;
use iced::widget::{column, row, scrollable, Column, Space};
use iced::Alignment::{self};
use iced::Subscription;
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
    ErrorOccurred(SiderealError),
    ErrorCleared(),
    LaunchPlanetarium,
    ServerStatus(ServerStatus),
    ConnectedDeviceChange(ConnectedDevices),
    IndiError(String),
    ModifyCameras(CameraMessage),
}
#[derive(Debug, Clone, Default)]
pub struct ConnectedDevices {
    pub mount: Option<String>,
    pub camera: Option<String>,
    pub focuser: Option<String>,
}

#[derive(Default)]
pub struct MainWindow {
    state: MainWindowState,
    error_message: Option<String>,
    server_status: ServerStatus,
    connected_devices: ConnectedDevices,
    camera_manager: CameraManager,
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

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            // your existing stream
            Subscription::run_with_id("coords_subscription", param_watcher()),
            // the camera stream
            self.camera_manager
                .subscription()
                .map(|m| Message::ModifyCameras(m)),
            // self.state.mount.subscription().map(|m| Message::Mount(m)),
        ])
    }

    pub fn run(settings: iced::Settings) -> iced::Result {
        let result = iced::application("Sidereal GUI", Self::update, Self::view)
            .subscription(|app: &MainWindow| app.subscription())
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
            Message::Setup(msg) => match msg {
                tabs::setup::Message::Bubble(bubble_message) => match bubble_message {
                    BubbleMessagePayload::Camera(camera_message) => {
                        return Task::done(Message::ModifyCameras(camera_message))
                    }
                },
                other => return self.state.setup.update(other),
            },
            Message::Mount(msg) => {
                return self.state.mount.update(msg);
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
                self.error_message = Some(err.to_string());
                if let SiderealError::ServerConnectionError(_) = err {
                    self.server_status = ServerStatus::Disconnected;
                }
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
                        Err(e) => {
                            Message::ErrorOccurred(SiderealError::PlanetariumError(e.to_string()))
                        }
                    },
                );
            }
            Message::ServerStatus(status) => {
                self.server_status = status;
            }
            Message::Noop => {}
            Message::ConnectedDeviceChange(connected_devices) => {
                self.connected_devices = connected_devices;
            }
            Message::IndiError(err) => self.error_message = Some(err.to_string()),
            Message::ModifyCameras(camera_message) => {
                self.camera_manager.handle_message(camera_message);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let header = tabs::header(self.state.active, |t| Message::Tab(t));

        let inner_content: Element<_> = match self.state.active {
            Tab::Setup => self
                .state
                .setup
                .view(&self.camera_manager)
                .map(Message::Setup),
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
            column![content_container(
                scrollable(
                    column![
                        content_container(
                            row![
                                text("Server Status:"),
                                Space::with_width(Length::Fill),
                                server_status_widget(&self.server_status)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(10),
                            ContainerLayer::Layer2
                        )
                        .width(Length::Fill),
                        container(
                            self.camera_manager
                                .view_cameras()
                                .map(Message::ModifyCameras)
                        )
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                        sidereal_button(
                            container(text("Launch Planetarium"))
                                .width(Length::Fill)
                                .align_x(Alignment::Center)
                        )
                        .width(Length::Fill)
                        .on_press(Message::LaunchPlanetarium),
                        text("Connected Devices"),
                        match &self.connected_devices.mount {
                            Some(mount) => column![content_container(
                                row![text("Mount:"), Space::with_width(Length::Fill), text(mount)],
                                ContainerLayer::Layer2
                            )],
                            None => Column::new(), // renders nothing
                        },
                        match &self.connected_devices.camera {
                            Some(camera) => column![content_container(
                                row![
                                    text("Camera:"),
                                    Space::with_width(Length::Fill),
                                    text(camera)
                                ],
                                ContainerLayer::Layer2
                            )],
                            None => Column::new(), // renders nothing
                        },
                        match &self.connected_devices.focuser {
                            Some(focuser) => column![content_container(
                                row![
                                    text("Focuser:"),
                                    Space::with_width(Length::Fill),
                                    text(focuser)
                                ],
                                ContainerLayer::Layer2
                            )],
                            None => Column::new(), // renders nothing
                        },
                    ]
                    .spacing(10) // .padding(iced::Padding {
                                 //     top: 0.0,
                                 //     right: 22.0,
                                 //     bottom: 0.0,
                                 //     left: 0.0,
                                 // })
                )
                .spacing(10),
                // .direction(scrollable::Direction::Vertical(
                //     Properties::new()
                //         .width(16) // reserve gutter width
                //         .scroller_width(8), // actual scrollbar thickness
                // )),
                ContainerLayer::Layer1
            )
            .width(Length::Fill),]
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
