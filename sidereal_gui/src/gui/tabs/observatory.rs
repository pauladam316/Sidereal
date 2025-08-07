use iced::{Element, Length};
use iced::widget::text;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct ObservatoryState;

impl ObservatoryState {
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'static, Message> {
        text("Setup tab").width(Length::Fill).into()
    }
}
