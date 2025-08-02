use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

fn spawn(path: &str) -> Child {
    Command::new(path)
        .spawn()
        .expect(&format!("failed to spawn {}", path))
}

fn main() {
    // Assuming the binaries have already been built and are in target/debug/
    let bevy_path = "target/debug/planetarium";
    let iced_path = "target/debug/gui";

    println!("Launching Bevy app...");
    let mut bevy_child = spawn(bevy_path);

    println!("Launching Iced app...");
    let mut iced_child = spawn(iced_path);

    // Optional: wait and monitor. Here we just wait until either exits.
    loop {
        if let Some(status) = bevy_child.try_wait().expect("failed to poll bevy") {
            println!("Bevy app exited with: {status}");
            break;
        }
        if let Some(status) = iced_child.try_wait().expect("failed to poll iced") {
            println!("Iced app exited with: {status}");
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }

    // Clean up: kill remaining child if still running.
    let _ = bevy_child.kill();
    let _ = iced_child.kill();
}
