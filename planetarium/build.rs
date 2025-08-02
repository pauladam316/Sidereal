// build.rs
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Recursively copy a directory.
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

/// Walk upward from `start` looking for a Cargo.toml containing a `[workspace]` table.
/// Returns the directory containing that file, or `None` if not found.
fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            if let Ok(contents) = std::fs::read_to_string(&candidate) {
                // crude check for `[workspace]` header; sufficient for typical manifests
                if contents.lines().any(|l| l.trim_start().starts_with("[workspace]")) {
                    return Some(dir.to_path_buf());
                }
            }
        }
        current = dir.parent();
    }
    None
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_assets = manifest_dir.join("assets");
    if !src_assets.exists() {
        println!(
            "cargo:warning=source assets directory does not exist: {}",
            src_assets.display()
        );
        return Ok(());
    }

    // Determine profile ("debug" or "release")
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());

    // Determine shared workspace target directory if possible
    let dest_base = if let Some(workspace_root) = find_workspace_root(&manifest_dir) {
        // Check if workspace root has a .cargo/config.toml overriding target-dir
        let mut target_dir = workspace_root.join("target");
        let cargo_config = workspace_root.join(".cargo").join("config.toml");
        if cargo_config.is_file() {
            if let Ok(contents) = std::fs::read_to_string(&cargo_config) {
                for line in contents.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("target-dir") {
                        // rudimentary parse like: target-dir = "some/path"
                        if let Some(eq) = trimmed.find('=') {
                            let value = trimmed[eq + 1..].trim().trim_matches('"');
                            target_dir = workspace_root.join(value);
                        }
                    }
                }
            }
        }
        target_dir
    } else {
        // Fallback to the local target directory semantics (honor CARGO_TARGET_DIR if set)
        let default_target = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
        PathBuf::from(default_target)
    };

    let dest_assets = dest_base.join(&profile).join("assets");

    println!(
        "cargo:warning=copying assets from {} to {}",
        src_assets.display(),
        dest_assets.display()
    );

    copy_dir_all(&src_assets, &dest_assets)?;
    Ok(())
}
