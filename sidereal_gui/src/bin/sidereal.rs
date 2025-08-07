use sidereal_gui::app;
fn main() -> iced::Result {
    let mut settings = iced::Settings::default();
    settings.default_text_size = 14.into();
    settings.antialiasing = true;
    app::MainWindow::run(settings)
}
