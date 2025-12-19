use sidereal_gui::app;

use env_logger;
use sidereal_gui::planetarium_handler::planetarium_receiver; // add `env_logger = "0.11"` in Cargo.toml (or similar)

use crate::app::set_grpc_receiver;
use gstreamer as gst; // add `gstreamer = "0.22"` (or latest) in Cargo.toml
use sidereal_gui::planetarium_handler::planetarium_receiver::ForwardedRPC;
use tokio::sync::mpsc;
fn main() -> iced::Result {
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();
    std::panic::set_hook(Box::new(|info| eprintln!("PANIC: {info}")));

    gst::init().unwrap();

    let (tx, rx) = mpsc::unbounded_channel::<ForwardedRPC>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        rt.block_on(async move {
            if let Err(e) = planetarium_receiver::run(tx).await {
                eprintln!("gRPC server error: {e}");
            }
        });
    });

    set_grpc_receiver(rx);

    let mut settings = iced::Settings::default();
    settings.default_text_size = 14.into();
    settings.antialiasing = true;

    app::MainWindow::run(settings)
}
