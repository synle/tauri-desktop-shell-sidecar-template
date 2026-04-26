// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

/// Returns the app version baked in at compile time by `build.rs`.
#[tauri::command]
fn get_app_version() -> &'static str {
    env!("APP_VERSION")
}

/// Spawns the bundled `example-sidecar` binary with the given arguments,
/// waits for it to exit, and returns its captured stdout.
///
/// Tauri's shell plugin resolves the binary by the name declared in
/// `tauri.conf.json` -> `bundle.externalBin`. The actual file on disk is
/// `src-tauri/binaries/example-sidecar-<target-triple>` so the build
/// can pick the right one per platform.
#[tauri::command]
async fn run_sidecar(app: tauri::AppHandle, args: Vec<String>) -> Result<String, String> {
    let cmd = app
        .shell()
        .sidecar("example-sidecar")
        .map_err(|e| format!("failed to resolve sidecar: {e}"))?
        .args(args);

    let (mut rx, _child) = cmd.spawn().map_err(|e| format!("failed to spawn sidecar: {e}"))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => {
                stdout.push_str(&String::from_utf8_lossy(&line));
                stdout.push('\n');
            }
            CommandEvent::Stderr(line) => {
                stderr.push_str(&String::from_utf8_lossy(&line));
                stderr.push('\n');
            }
            CommandEvent::Terminated(payload) => {
                if payload.code.unwrap_or(0) != 0 {
                    return Err(format!(
                        "sidecar exited with code {:?}: {}",
                        payload.code, stderr
                    ));
                }
                break;
            }
            _ => {}
        }
    }
    Ok(stdout.trim_end().to_string())
}

/// Tauri application entry point.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // No production sidecar spawning needed at startup — the sidecar is
            // launched on demand via the `run_sidecar` Tauri command. If your
            // sidecar is a long-running server, spawn it here and store the
            // child handle in `app.manage(...)` (see display-dj for an example).
            let _ = _app.handle();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_app_version, run_sidecar])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_is_non_empty() {
        assert!(!get_app_version().is_empty());
    }
}
