use crate::app::Message as MainMessage;
use crate::gui::styles;
use crate::indi_handler::mount;
use crate::model::SiderealResult;
use iced::widget::{button, image, mouse_area, Button};
use iced::{Background, Border, Color, Element, Length, Task, Theme};
#[derive(Debug, Clone)]
pub enum MountMoveMessage {
    MoveMount(ButtonDirection), // start on mouse-down
    StopMoveMount,              // stop on mouse-up / leave
    Hover(bool),                // optional: for highlight
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonDirection {
    N,
    S,
    E,
    W,
    NE,
    SE,
    NW,
    SW,
    Stop,
}

pub fn mount_steer_button<'a, Message>(
    direction: ButtonDirection,
    hovered: bool,
    pressed: bool,
) -> Button<'a, Message>
where
    Message: 'a + Clone,
{
    let icon_bytes: &'static [u8] = match direction {
        ButtonDirection::N => include_bytes!("../../../assets/N.png").as_slice(),
        ButtonDirection::S => include_bytes!("../../../assets/S.png").as_slice(),
        ButtonDirection::E => include_bytes!("../../../assets/E.png").as_slice(),
        ButtonDirection::W => include_bytes!("../../../assets/W.png").as_slice(),
        ButtonDirection::NE => include_bytes!("../../../assets/NE.png").as_slice(),
        ButtonDirection::SE => include_bytes!("../../../assets/SE.png").as_slice(),
        ButtonDirection::NW => include_bytes!("../../../assets/NW.png").as_slice(),
        ButtonDirection::SW => include_bytes!("../../../assets/SW.png").as_slice(),
        ButtonDirection::Stop => include_bytes!("../../../assets/stop.png").as_slice(),
    };

    let handle = image::Handle::from_bytes(icon_bytes);

    let img = image(handle)
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0));

    button(img)
        .padding([6, 12])
        .style(move |_theme: &Theme, _status| {
            iced::widget::button::Style {
                background: Some(match pressed {
                    false => Background::Color(styles::BUTTON_COLOR),
                    true => Background::Color(styles::CONTAINER_LAYER_3),
                }),

                text_color: if hovered {
                    styles::ACCENT_COLOR
                } else {
                    styles::TEXT_COLOR
                },

                shadow: iced::Shadow {
                    offset: iced::Vector::new(1.0, 1.0),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.2,
                    }, // soft drop shadow
                    blur_radius: 3.0,
                },
                border: Border {
                    color: if hovered {
                        styles::ACCENT_COLOR
                    } else {
                        styles::ELEMENT_BORDER
                    },
                    width: 1.0,
                    radius: 7.0.into(),
                },
            }
        })
}

#[derive(Default)]
pub struct MountSteerButton {
    hovered: bool,
    pressed: bool,
}

impl MountSteerButton {
    pub fn view(&self, direction: ButtonDirection) -> Element<MountMoveMessage> {
        // Your styled button that accepts state flags
        let visual = mount_steer_button(direction, self.hovered, self.pressed)
            .padding(10)
            .width(Length::Shrink);

        mouse_area(visual)
            // fire immediately on mouse DOWN
            .on_press(MountMoveMessage::MoveMount(direction))
            // stop on mouse UP (even if cursor still inside)
            .on_release(MountMoveMessage::StopMoveMount)
            // keep your hover/pressed visuals in sync
            .on_enter(MountMoveMessage::Hover(true))
            .on_exit(MountMoveMessage::Hover(false))
            // .on_mouse_press(|_| Message::Press(true))
            // .on_mouse_release(|_| Message::Press(false))
            .into()
    }
    fn stop_move(&mut self) -> Task<MainMessage> {
        self.pressed = false;
        Task::perform(
            async {
                mount::stop_move().await;
            },
            |_| MainMessage::Noop,
        )
    }
    pub fn update(&mut self, msg: MountMoveMessage) -> Task<MainMessage> {
        match msg {
            MountMoveMessage::Hover(h) => {
                self.hovered = h;
                if !h && self.pressed {
                    return self.stop_move();
                }
            }
            MountMoveMessage::MoveMount(dir) => {
                self.pressed = true;
                return Task::perform(
                    async move {
                        match dir {
                            ButtonDirection::N => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_NORTH".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::S => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_SOUTH".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::E => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_EAST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::W => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_WEST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::NE => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_NORTH".to_string(),
                                )
                                .await?;
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_EAST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::SE => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_SOUTH".to_string(),
                                )
                                .await?;
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_EAST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::NW => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_NORTH".to_string(),
                                )
                                .await?;
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_WEST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::SW => {
                                mount::move_mount(
                                    "TELESCOPE_MOTION_NS".to_string(),
                                    "MOTION_SOUTH".to_string(),
                                )
                                .await?;
                                mount::move_mount(
                                    "TELESCOPE_MOTION_WE".to_string(),
                                    "MOTION_WEST".to_string(),
                                )
                                .await?;
                            }
                            ButtonDirection::Stop => todo!(),
                        }

                        Ok(())
                    },
                    |result: SiderealResult<()>| match result {
                        Ok(_) => MainMessage::Noop,
                        Err(e) => MainMessage::ErrorOccurred(e),
                    },
                );
            }
            MountMoveMessage::StopMoveMount => {
                if self.pressed {
                    return self.stop_move();
                }
            }
        }
        Task::none()
    }
}
