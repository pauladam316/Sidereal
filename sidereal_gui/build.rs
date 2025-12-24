// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this build script if these environment variables change
    println!("cargo:rerun-if-env-changed=GSTREAMER_ROOT");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    
    // On macOS, we need to help the dynamic linker find GStreamer libraries
    if cfg!(target_os = "macos") {
        // Try to find GStreamer installation
        let mut gst_lib_paths: Vec<PathBuf> = vec![
            // Homebrew on Apple Silicon
            PathBuf::from("/opt/homebrew/lib"),
            // Homebrew on Intel
            PathBuf::from("/usr/local/lib"),
        ];
        
        // Add custom installation if specified
        if let Ok(root) = env::var("GSTREAMER_ROOT") {
            gst_lib_paths.push(PathBuf::from(root).join("lib"));
        }
        
        // Filter to only existing paths
        let gst_lib_paths: Vec<PathBuf> = gst_lib_paths
            .into_iter()
            .filter(|path| path.exists())
            .collect();

        // Add rpath for each found GStreamer library path
        for lib_path in &gst_lib_paths {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
        }

        // Also set up library search path
        for lib_path in &gst_lib_paths {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
        }

        if gst_lib_paths.is_empty() {
            println!("cargo:warning=GStreamer library path not found. Please install GStreamer via Homebrew: brew install gstreamer gst-plugins-base gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav");
        }
    }
}

