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

    // On macOS, help GStreamer find its plugins
    #[cfg(target_os = "macos")]
    {
        use std::path::PathBuf;
        let plugin_paths = vec![
            "/opt/homebrew/lib/gstreamer-1.0",  // Apple Silicon Homebrew
            "/usr/local/lib/gstreamer-1.0",     // Intel Homebrew
        ];
        
        for path in plugin_paths {
            if PathBuf::from(path).exists() {
                if let Ok(current) = std::env::var("GST_PLUGIN_PATH") {
                    std::env::set_var("GST_PLUGIN_PATH", format!("{}:{}", path, current));
                } else {
                    std::env::set_var("GST_PLUGIN_PATH", path);
                }
                break;
            }
        }
    }

    gst::init().unwrap_or_else(|e| {
        eprintln!("Failed to initialize GStreamer: {}", e);
        eprintln!("On macOS, make sure GStreamer is installed via Homebrew:");
        eprintln!("  brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav");
        std::process::exit(1);
    });

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
