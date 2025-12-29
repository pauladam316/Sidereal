use std::sync::Arc;

use crate::gui::camera_display::{CameraManager, CameraMessage};
use crate::gui::dialogs::add_server;
use crate::gui::dialogs::error::error_dialog;
use crate::gui::styles::button_style::sidereal_button;
use crate::gui::styles::container_style::{content_container, ContainerLayer};
use crate::gui::tabs::setup::{self, BubbleMessagePayload};
use crate::gui::widgets::server_status::{server_status_widget, ServerStatus};
use crate::indi_handler::{device_discovery_watcher, param_watcher, server_disconnect_watcher};
use crate::model::{SiderealError, SiderealResult};
use crate::planetarium_handler::{planetarium_receiver, planetarium_sender};
use crate::{
    config::Config,
    gui::{
        styles::{tab_style::tab_content, SIDEREAL_THEME},
        tabs::{self, MainWindowState, Tab},
    },
};
use iced::futures::SinkExt;
use iced::widget::container;
use iced::widget::{column, row, scrollable, Column, Space};
use iced::window::{self, icon};
use iced::Alignment::{self};
use iced::{stream, Settings, Subscription};
use iced::{widget::text, Element, Length, Task};
use once_cell::sync::OnceCell;
use planetarium_receiver::ForwardedRPC;
use tokio::sync::{mpsc, Mutex};
static RPC_RX: OnceCell<Arc<Mutex<Option<mpsc::UnboundedReceiver<ForwardedRPC>>>>> =
    OnceCell::new();

pub fn set_grpc_receiver(rx: mpsc::UnboundedReceiver<ForwardedRPC>) {
    let _ = RPC_RX.set(Arc::new(Mutex::new(Some(rx))));
}

fn rpc_subscription_worker() -> impl iced::futures::Stream<Item = Message> {
    stream::channel(256, |mut output| async move {
        // If the receiver hasn't been set, just end the worker quietly.
        let Some(holder) = RPC_RX.get().cloned() else {
            return; // no stream; nothing to forward
        };

        let mut rx_opt = holder.lock().await.take();

        if let Some(ref mut rx) = rx_opt {
            while let Some(evt) = rx.recv().await {
                if output.send(Message::ForwardedRPC(evt)).await.is_err() {
                    break; // UI dropped
                }
            }
        }
    })
}

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
    Telescope(tabs::telescope::Message),
    ConfigLoaded(Config),
    ErrorOccurred(SiderealError),
    ErrorCleared(),
    LaunchPlanetarium,
    ServerStatus(ServerStatus),
    ConnectedDeviceChange(ConnectedDevices),
    IndiError(String),
    ModifyCameras(CameraMessage),
    AddServer(add_server::Message),
    ForwardedRPC(ForwardedRPC),
}
#[derive(Debug, Clone, Default)]
pub struct ConnectedDevices {
    pub mount: Option<String>,
    pub camera: Option<String>,
    pub focuser: Option<String>,
    pub telescope_controller: Option<String>,
    pub roof_controller: Option<String>,
}

#[derive(Default)]
pub struct MainWindow {
    state: MainWindowState,
    dialog: Option<DialogType>,
    server_status: ServerStatus,
    connected_devices: ConnectedDevices,
    camera_manager: CameraManager,
}

pub enum DialogType {
    Error(String),
    AddServer(add_server::AddServerDialog),
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
            Subscription::run_with_id("param_watcher", param_watcher()),
            Subscription::run_with_id("device_discovery", device_discovery_watcher()),
            Subscription::run_with_id("server_disconnect", server_disconnect_watcher()),
            self.camera_manager
                .subscription()
                .map(Message::ModifyCameras),
            // NEW: gRPC → mpsc → Iced
            Subscription::run_with_id("grpc-forwarded-rpc", rpc_subscription_worker()),
        ])
    }

    pub fn run(settings: Settings) -> iced::Result {
        // Build window settings (size + optional icon)
        let mut win = window::Settings {
            size: iced::Size::new(1200.0, 900.0),
            ..Default::default()
        };

        // Prefer PNG if you can. ICO may work depending on enabled decoders.
        if let Ok(bytes) = std::fs::read("assets/icon.ico") {
            if let Ok(ic) = icon::from_file_data(&bytes, None) {
                win.icon = Some(ic);
            }
        }

        iced::application("Sidereal GUI", Self::update, Self::view)
            .subscription(|app: &MainWindow| app.subscription())
            .theme(|_| SIDEREAL_THEME.clone())
            .settings(settings)
            .window(win)
            .run_with(Self::new)
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
                    BubbleMessagePayload::AddServer => {
                        self.dialog =
                            Some(DialogType::AddServer(add_server::AddServerDialog::default()));
                    }
                },
                other => return self.state.setup.update(other),
            },
            Message::Mount(msg) => {
                return self.state.mount.update(msg);
            }
            Message::Observatory(msg) => {
                return self.state.observatory.update(msg);
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
            Message::Telescope(msg) => {
                return self.state.telescope.update(msg);
            }
            Message::ConfigLoaded(config) => {
                self.state.setup.on_config_load(config.clone());
                self.camera_manager.load_from_config(config.cameras);
                // Automatically connect all cameras that were loaded from config
                for camera_index in 0..self.camera_manager.cameras.len() {
                    self.camera_manager
                        .handle_message(CameraMessage::ConnectCamera(camera_index));
                }
            }
            Message::ErrorOccurred(err) => {
                self.dialog = Some(DialogType::Error(err.to_string()));

                if let SiderealError::ServerConnectionError(_) = err {
                    self.server_status = ServerStatus::Disconnected;
                }
            }
            Message::ErrorCleared() => self.dialog = None,
            Message::LaunchPlanetarium => {
                return Task::perform(
                    async {
                        planetarium_sender::launch_planetarium()
                            .await
                            .map_err(|e| e.to_string())?;
                        planetarium_sender::set_site_location()
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
                // Clear connected devices when server connection is lost or disconnected
                if matches!(
                    status,
                    ServerStatus::ConnectionLost | ServerStatus::Disconnected
                ) {
                    self.connected_devices = ConnectedDevices::default();
                }
                self.server_status = status;
            }
            Message::Noop => {}
            Message::ConnectedDeviceChange(connected_devices) => {
                self.connected_devices = connected_devices;
            }
            Message::IndiError(err) => self.dialog = Some(DialogType::Error(err.to_string())),
            Message::ModifyCameras(camera_message) => {
                // Only save cameras when configuration changes, not on streaming/connection updates
                let should_save = matches!(
                    camera_message,
                    CameraMessage::AddCamera
                        | CameraMessage::RemoveCamera(_)
                        | CameraMessage::SetCameraType { .. }
                        | CameraMessage::SetCameraField { .. }
                );

                self.camera_manager.handle_message(camera_message);

                if should_save {
                    // Save cameras to config after configuration modification
                    let cameras_config = self.camera_manager.to_config_cameras();
                    return Task::perform(
                        async move {
                            crate::config::Config::update_cameras(cameras_config).await?;
                            Ok(())
                        },
                        |result: SiderealResult<()>| match result {
                            Ok(()) => Message::Noop,
                            Err(e) => {
                                Message::ErrorOccurred(SiderealError::ConfigError(e.to_string()))
                            }
                        },
                    );
                }
            }
            Message::AddServer(child) => {
                if let Some(DialogType::AddServer(ref mut dialog)) = self.dialog {
                    match child.clone() {
                        add_server::Message::Cancel => {
                            self.dialog = None;
                            return Task::none();
                        }
                        add_server::Message::Submit { ip, port } => {
                            self.dialog = None;
                            return self
                                .state
                                .setup
                                .update(setup::Message::AddServer { ip, port });
                        }
                        _ => {
                            return dialog.update(child).map(Message::AddServer);
                        }
                    }
                }
            }
            Message::ForwardedRPC(_rpc) => {
                println!("test message received");
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
            Tab::Telescope => self.state.telescope.view().map(Message::Telescope),
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
                                .align_x(Alignment::Center),
                            Some(Message::LaunchPlanetarium),
                            true,
                        )
                        .width(Length::Fill),
                        content_container(
                            column![
                                text("Connected Devices"),
                                match &self.connected_devices.mount {
                                    Some(mount) => column![content_container(
                                        row![
                                            text("Mount:"),
                                            Space::with_width(Length::Fill),
                                            text(mount)
                                        ],
                                        ContainerLayer::Layer3
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
                                        ContainerLayer::Layer3
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
                                        ContainerLayer::Layer3
                                    )],
                                    None => Column::new(), // renders nothing
                                },
                                match &self.connected_devices.telescope_controller {
                                    Some(telescope_controller) => column![content_container(
                                        row![
                                            text("Telescope Controller:"),
                                            Space::with_width(Length::Fill),
                                            text(telescope_controller)
                                        ],
                                        ContainerLayer::Layer3
                                    )],
                                    None => Column::new(), // renders nothing
                                },
                                match &self.connected_devices.roof_controller {
                                    Some(roof_controller) => column![content_container(
                                        row![
                                            text("Roof Controller:"),
                                            Space::with_width(Length::Fill),
                                            text(roof_controller)
                                        ],
                                        ContainerLayer::Layer3
                                    )],
                                    None => Column::new(), // renders nothing
                                },
                            ]
                            .spacing(5),
                            ContainerLayer::Layer2
                        )
                        .width(Length::Fill),
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

        let view = match &self.dialog {
            Some(dialog) => match dialog {
                DialogType::Error(error_message) => {
                    error_dialog(layout, error_message.to_string(), Message::ErrorCleared())
                }
                DialogType::AddServer(dialog) => dialog.view(layout, Message::AddServer),
            },
            None => layout.into(),
        };

        view.into()
    }
}
