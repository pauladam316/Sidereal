use iced::widget::text;
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct MountState;

impl MountState {
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'static, Message> {
        text("Mount tab").width(Length::Fill).into()
    }
}
