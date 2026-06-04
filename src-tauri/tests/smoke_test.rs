use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
#[ignore] // This test is slow and requires a prior build. Run with `cargo test -- --ignored`.
fn smoke_test_app_launches() {
    // This smoke test assumes the application has been built for release using `cargo tauri build`.
    let profile = "release";
    let exe_name = "nodaysidle-whispering";

    // The executable is located in the `src-tauri/target` directory.
    let mut exe_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    exe_path.push("target");
    exe_path.push(profile);
    exe_path.push(exe_name);

    println!("Attempting to find application binary at: {:?}", exe_path);

    // On macOS, the executable is inside the .app bundle, which is the primary build artifact.
    // We check for the raw executable first, then the bundled one.
    if !exe_path.exists() {
        let mut bundle_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        bundle_path.push("target");
        bundle_path.push(profile);
        bundle_path.push("bundle");
        bundle_path.push("macos");
        bundle_path.push("NoDaysIdle Whispering.app");
        bundle_path.push("Contents");
        bundle_path.push("MacOS");
        bundle_path.push(exe_name);

        if bundle_path.exists() {
            exe_path = bundle_path;
            println!("Found application binary in .app bundle: {:?}", exe_path);
        } else {
            panic!(
                "Application executable not found at {:?} or in the .app bundle. Please run `cargo tauri build` before running this test.",
                exe_path
            );
        }
    }

    let mut child = Command::new(&exe_path)
        .spawn()
        .expect("Failed to start the application process.");

    // Allow the application some time to initialize.
    thread::sleep(Duration::from_secs(5));

    // Check if the application process has exited prematurely.
    match child.try_wait() {
        Ok(Some(status)) => {
            panic!("Application exited unexpectedly with status: {}", status);
        }
        Ok(None) => {
            // The process is still running, which indicates a successful launch.
            println!("Application launched successfully and is still running.");
            child
                .kill()
                .expect("Failed to kill the application process.");
            println!("Application process terminated for test cleanup.");
        }
        Err(e) => {
            panic!(
                "An error occurred while waiting for the application process: {}",
                e
            );
        }
    }
}
