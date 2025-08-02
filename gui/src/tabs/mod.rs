use iced::widget::{column, row, text};
use iced::{Element, Font, Length};

pub mod capture;
pub mod focus;
pub mod guide;
pub mod mount;
pub mod observatory;
pub mod plate_solve;
pub mod setup;

use crate::styles::tab_style::tab_button;

use self::capture::CaptureState;
use self::focus::FocusState;
use self::guide::GuideState;
use self::mount::MountState;
use self::observatory::ObservatoryState;
use self::plate_solve::PlateSolveState;
use self::setup::SetupState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Mount,
    Setup,
    Observatory,
    PlateSolve,
    Guide,
    Focus,
    Capture,
}

impl Default for Tab {
    fn default() -> Self {
        Tab::Setup
    }
}

#[derive(Default)]
pub struct MainWindowState {
    pub active: Tab,
    pub mount: MountState,
    pub setup: SetupState,
    pub observatory: ObservatoryState,
    pub plate_solve: PlateSolveState,
    pub guide: GuideState,
    pub focus: FocusState,
    pub capture: CaptureState,
}

pub fn header<F, M>(active: Tab, on_select: F) -> Element<'static, M>
where
    F: 'static + Copy + Fn(Tab) -> M,
    M: Clone + 'static,
{
    let tab_button = |label: &'static str, tab: Tab| -> iced::widget::Button<'static, M> {
        let is_active = tab == active;

        //let label_widget = text(label).width(Length::Fill).size(16).center();

        tab_button(label, is_active)
            .padding([7, 13])
            .on_press(on_select(tab))
    };

    column![row![
        tab_button("Setup", Tab::Setup),
        tab_button("Mount", Tab::Mount),
        tab_button("Observatory", Tab::Observatory),
        tab_button("Plate Solve", Tab::PlateSolve),
        tab_button("Focus", Tab::Focus),
        tab_button("Capture", Tab::Capture),
        tab_button("Guide", Tab::Guide)
    ]
    .spacing(5)
    .width(Length::Fill),]
    .spacing(4)
    .width(Length::Fill)
    .into()
}
