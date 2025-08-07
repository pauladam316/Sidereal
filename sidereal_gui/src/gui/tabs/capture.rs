use iced::widget::text;
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct CaptureState;

impl CaptureState {
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'static, Message> {
        text("Setup tab").width(Length::Fill).into()
    }
}
