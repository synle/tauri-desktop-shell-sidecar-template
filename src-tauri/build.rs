use std::path::PathBuf;
use std::process::Command;

/// Build script:
///   1. Exposes `APP_VERSION` from `tauri.conf.json` to Rust at compile time.
///   2. Builds the sidecar binary in `../sidecar-src` for the current target
///      and copies it to `binaries/example-sidecar-<target-triple>(.exe)`
///      where Tauri's `externalBin` config expects to find it.
fn main() {
    expose_app_version();
    build_sidecar();
    tauri_build::build();
}

/// Read the version from `tauri.conf.json` (the single source of truth) and
/// expose it as the compile-time env var `APP_VERSION`.
fn expose_app_version() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let conf_path = manifest_dir.join("tauri.conf.json");
    let conf: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&conf_path).expect("failed to read tauri.conf.json"),
    )
    .expect("failed to parse tauri.conf.json");
    let version = conf["version"].as_str().expect("version missing in tauri.conf.json");

    let is_release = std::env::var("TAURI_RELEASE").unwrap_or_default() == "true";
    let app_version = if is_release {
        version.to_string()
    } else {
        format!("{version} [DEV]")
    };
    println!("cargo:rustc-env=APP_VERSION={app_version}");
    println!("cargo:rerun-if-changed=tauri.conf.json");
}

/// Build the sidecar in `../sidecar-src` and place the resulting binary
/// under `binaries/` with the target-triple suffix Tauri expects.
fn build_sidecar() {
    let target = std::env::var("TARGET").expect("TARGET not set by Cargo");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sidecar_src = manifest_dir.join("..").join("sidecar-src");
    let binaries_dir = manifest_dir.join("binaries");
    std::fs::create_dir_all(&binaries_dir).expect("failed to create binaries dir");

    let exe_suffix = if target.contains("windows") { ".exe" } else { "" };
    let dest = binaries_dir.join(format!("example-sidecar-{target}{exe_suffix}"));

    // Skip rebuild if the destination is fresher than the source (avoids
    // triggering an infinite watch loop in `tauri dev`).
    let needs_build = match (
        std::fs::metadata(&dest),
        std::fs::metadata(sidecar_src.join("src/main.rs")),
    ) {
        (Ok(d), Ok(s)) => match (d.modified(), s.modified()) {
            (Ok(dm), Ok(sm)) => sm > dm,
            _ => true,
        },
        _ => true,
    };
    if !needs_build {
        println!("Sidecar binary up-to-date: {}", dest.display());
        println!("cargo:rerun-if-changed=../sidecar-src/src/main.rs");
        return;
    }

    println!("Building sidecar for {target}...");
    let status = Command::new("cargo")
        .args(["build", "--release", "--target", &target, "--manifest-path"])
        .arg(sidecar_src.join("Cargo.toml"))
        .status()
        .expect("failed to invoke cargo for sidecar build");
    if !status.success() {
        panic!("sidecar build failed");
    }

    let built = sidecar_src
        .join("target")
        .join(&target)
        .join("release")
        .join(format!("example-sidecar{exe_suffix}"));
    std::fs::copy(&built, &dest)
        .unwrap_or_else(|e| panic!("failed to copy {} -> {}: {e}", built.display(), dest.display()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms).unwrap();
    }

    println!("Sidecar binary: {}", dest.display());
    println!("cargo:rerun-if-changed=../sidecar-src/src/main.rs");
    println!("cargo:rerun-if-changed=../sidecar-src/Cargo.toml");
}
