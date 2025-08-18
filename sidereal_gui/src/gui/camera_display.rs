use std::fmt;

use crate::gui::{
    styles::{
        button_style::sidereal_button,
        container_style::{content_container, ContainerLayer},
        picklist_style::sidereal_picklist,
        text_input_style::sidereal_text_input,
    },
    widgets::video::{IpCamera, IpCameraMessage},
};
use iced::{
    widget::{column, row, text},
    Subscription,
};
use iced::{Alignment, Element};

#[derive(Debug, Clone)]
pub enum CameraField {
    Url,
}

#[derive(Debug, Clone)]
pub enum CameraMessageType {
    IpCamera(IpCameraMessage),
}

#[derive(Debug, Clone)]
pub enum CameraMessage {
    AddCamera,
    SetCameraType {
        camera_index: usize,
        camera_type: CameraType,
    },
    SetCameraField {
        camera_index: usize,
        field: CameraField,
        value: String,
    },
    RemoveCamera(usize),
    UpdateCamera {
        camera_index: usize,
        message: CameraMessageType,
    },
    ConnectCamera(usize),
}
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RTSPCameraSettings {
    pub url: String,
    pub username: String,
    pub password: String,
    pub camera: IpCamera,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AllSkyCameraSettings {
    pub url: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum CameraType {
    RTSP(IpCamera),
    AllSky(AllSkyCameraSettings),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub camera_type: CameraType,
}

#[derive(Default)]
pub struct CameraManager {
    pub cameras: Vec<Camera>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            camera_type: CameraType::RTSP(IpCamera::default()),
        }
    }
}

impl fmt::Display for CameraType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CameraType::RTSP(_) => write!(f, "RTSP"),
            CameraType::AllSky(_) => write!(f, "AllSky"),
        }
    }
}

impl CameraManager {
    pub fn subscription(&self) -> Subscription<CameraMessage> {
        let subs = self
            .cameras
            .iter()
            .enumerate()
            .map(|(i, cam)| match &cam.camera_type {
                CameraType::RTSP(camera) => camera.subscription_with_index(i),
                _ => Subscription::none(),
            });
        Subscription::batch(subs)
    }
    pub fn handle_message(&mut self, message: CameraMessage) {
        match message {
            CameraMessage::AddCamera => self.cameras.push(Camera::default()),
            CameraMessage::SetCameraType {
                camera_index,
                camera_type,
            } => {
                self.cameras[camera_index].camera_type = camera_type;
            }
            CameraMessage::SetCameraField {
                camera_index,
                field,
                value,
            } => {
                if let Some(cam) = self.cameras.get_mut(camera_index) {
                    match &mut cam.camera_type {
                        CameraType::RTSP(camera) => match field {
                            CameraField::Url => camera.url = value,
                        },
                        CameraType::AllSky(_all_sky_camera_settings) => {
                            // TODO
                        }
                    }
                }
            }
            CameraMessage::RemoveCamera(camera_index) => {
                self.cameras.remove(camera_index);
            }
            CameraMessage::UpdateCamera {
                camera_index,
                message,
            } => match message {
                CameraMessageType::IpCamera(ip_camera_message) => {
                    match self.cameras.get_mut(camera_index) {
                        Some(cam) => match cam.camera_type {
                            CameraType::RTSP(ref mut camera) => {
                                camera.update(ip_camera_message);
                            }
                            _ => panic!("sending an IP camera message to a non RTSP camera, something went very wrong!"),
                        },
                        None => panic!("no camera with that index, something went very wrong!"),
                    }
                }
            },
            CameraMessage::ConnectCamera(camera_index) => {
                if let Some(cam) = self.cameras.get_mut(camera_index) {
                    match &mut cam.camera_type {
                        CameraType::RTSP(camera) => {
                            camera.connect();
                        }
                        CameraType::AllSky(_all_sky_camera_settings) => {
                            // TODO
                        }
                    }
                }
            }
        }
    }

    pub fn view_cameras(&self) -> Element<CameraMessage> {
        let mut col = column![].spacing(10);
        for (i, camera) in self.cameras.iter().enumerate() {
            if let CameraType::RTSP(camera) = &camera.camera_type {
                col = col.push(camera.view().map({
                    let i = i;
                    move |ip_msg: IpCameraMessage| CameraMessage::UpdateCamera {
                        camera_index: i,
                        message: CameraMessageType::IpCamera(ip_msg),
                    }
                }));
            }
        }
        col.into()
    }

    pub fn view_camera_setup(&self) -> Element<CameraMessage> {
        let camera_types = vec![
            CameraType::RTSP(IpCamera::default()),
            CameraType::AllSky(AllSkyCameraSettings::default()),
        ];
        let mut col = column![].spacing(10);
        for (i, camera) in self.cameras.iter().enumerate() {
            col = col.push(content_container(
                column![
                    row![
                        text("Camera type: "),
                        sidereal_picklist(
                            camera_types.to_vec(),
                            Some(camera.camera_type.clone()),
                            move |camera_type| {
                                CameraMessage::SetCameraType {
                                    camera_index: i,
                                    camera_type,
                                }
                            }
                        ),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    match &camera.camera_type {
                        CameraType::RTSP(rtspcamera_settings) => {
                            row![
                                text("URL: "),
                                sidereal_text_input("url", &rtspcamera_settings.url).on_input(
                                    move |v| {
                                        CameraMessage::SetCameraField {
                                            camera_index: i,
                                            field: CameraField::Url,
                                            value: v,
                                        }
                                    }
                                ),
                                sidereal_button("Connect")
                                    .on_press(CameraMessage::ConnectCamera(i)),
                                sidereal_button("Remove").on_press(CameraMessage::RemoveCamera(i))
                            ]
                            .spacing(10)
                            .align_y(Alignment::Center)
                        }
                        CameraType::AllSky(all_sky_camera_settings) => row![
                            text("URL: "),
                            sidereal_text_input("url", &all_sky_camera_settings.url).on_input(
                                move |v| {
                                    CameraMessage::SetCameraField {
                                        camera_index: i,
                                        field: CameraField::Url,
                                        value: v,
                                    }
                                }
                            ),
                            sidereal_button("Connect"),
                            sidereal_button("Remove").on_press(CameraMessage::RemoveCamera(i))
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    }
                ]
                .spacing(10),
                ContainerLayer::Layer2,
            ))
        }
        col.into()
    }
}
