use iced::window;
use sidereal_gui::app;
fn main() -> iced::Result {
    let mut settings = iced::Settings::default();
    settings.default_text_size = 14.into();
    settings.antialiasing = true;
    // settings.size
    // let font = Font::with_name("MyCustomFont"); // 👈 choose any label
    // Font::from(MY_FONT); // 👈 associate the label with your bytes
    // settings.default_text_size = 12.into();
    // settings.default_font = iced::font::load(MY_FONT).expect("");
    app::MainWindow::run(settings)
}
