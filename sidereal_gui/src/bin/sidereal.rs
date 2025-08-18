use sidereal_gui::app;

use env_logger; // add `env_logger = "0.11"` in Cargo.toml (or similar)

use gstreamer as gst; // add `gstreamer = "0.22"` (or latest) in Cargo.toml

fn main() -> iced::Result {
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();
    std::panic::set_hook(Box::new(|info| eprintln!("PANIC: {info}")));

    gst::init().unwrap();

    let mut settings = iced::Settings::default();
    settings.default_text_size = 14.into();
    settings.antialiasing = true;
    app::MainWindow::run(settings)
}
