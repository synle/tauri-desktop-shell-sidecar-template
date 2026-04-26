//! Example sidecar binary.
//!
//! Tauri's shell plugin invokes this binary with platform-specific arguments
//! and captures its stdout/stderr. Replace this with your real CLI logic.
//!
//! Try: `example-sidecar --greet world`

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--greet" => {
                let name = args.get(i + 1).cloned().unwrap_or_else(|| "world".to_string());
                println!("Hello, {name}! (from example-sidecar)");
                i += 2;
            }
            "--version" => {
                println!("example-sidecar {}", env!("CARGO_PKG_VERSION"));
                i += 1;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }
    ExitCode::SUCCESS
}
