# tauri-desktop-shell-sidecar-template — Architecture

## High-Level Overview

Tauri v2 desktop application template demonstrating the **shell-sidecar** integration pattern. The app has three cooperating processes:

1. **WebView UI** — React 19 + MUI v9 + TypeScript, served by Vite in dev and bundled to static assets (`dist/`) in release. Calls into the Rust core via `@tauri-apps/api` `invoke()`.
2. **Rust core** (`src-tauri/`) — Tauri host process. Exposes `#[tauri::command]` handlers (`get_app_version`, `run_sidecar`) and owns the `tauri-plugin-shell` instance.
3. **Sidecar binary** (`sidecar-src/`) — a standalone CLI compiled per target triple, bundled alongside the app, and invoked on demand by the Rust core through `tauri-plugin-shell`'s sidecar API.

Invocation flow for a sidecar call:

```
React (invoke "run_sidecar", args)
  -> Rust #[tauri::command] run_sidecar(app, args)
       -> app.shell().sidecar("example-sidecar").args(args).spawn()
            -> OS spawns binaries/example-sidecar-<target-triple>
            <- CommandEvent::{Stdout, Stderr, Terminated} stream
  <- Result<String, String> (stdout concatenated)
<- JS Promise resolves
```

The sidecar is **on-demand, one-shot**: spawned per invocation, stdout/stderr captured, exit code checked. The template's `setup()` hook intentionally does not start a long-running sidecar; `src-tauri/src/lib.rs` documents how to do so (`app.manage(child)`) if needed.

Permissioning is enforced two ways: `tauri.conf.json` declares the binary under `bundle.externalBin` and `plugins.shell.scope`, and `src-tauri/capabilities/default.json` grants `shell:allow-execute` to that exact `name` with `sidecar: true`. Both must agree or the spawn is rejected at runtime.

## Key Directories

- `src/` — React frontend. Entry `main.tsx`, root `App.tsx`, plus `components/`, `pages/`, and `test/` (Vitest + Testing Library).
- `src-tauri/` — Rust host crate (Tauri v2).
  - `src/` — `main.rs` (entry shim), `lib.rs` (Tauri builder, commands).
  - `binaries/` — output directory; `build.rs` drops compiled sidecars here as `example-sidecar-<target-triple>[.exe]`. Gitignored; populated at build time.
  - `capabilities/` — Tauri v2 capability manifests (`default.json` declares shell permissions).
  - `icons/` — bundled app icons (PNG/ICNS/ICO).
- `sidecar-src/` — separate Rust crate (`example-sidecar`) producing the sidecar CLI. Self-contained `Cargo.toml`, source under `src/main.rs`.
- `.github/workflows/` — `build.yml` (CI), `release-official.yml`, `release-beta.yml`.

## Important Files

- `src-tauri/tauri.conf.json` — single source of truth for app version. Declares:
  - `bundle.externalBin: ["binaries/example-sidecar"]` — names the binary Tauri must bundle. The bundler resolves `binaries/example-sidecar-<target-triple>` per platform.
  - `plugins.shell.scope` — allow-list entry for `binaries/example-sidecar` with `sidecar: true, args: true`.
- `src-tauri/capabilities/default.json` — capability manifest. Grants `shell:allow-open` and `shell:allow-execute` for the same sidecar name. Required by Tauri v2's permission model; absence (or mismatch) blocks the spawn.
- `src-tauri/build.rs` — the build wrapper. Two responsibilities:
  1. Read `version` from `tauri.conf.json` and emit it as `APP_VERSION` (with `[DEV]` suffix unless `TAURI_RELEASE=true`) so `env!("APP_VERSION")` resolves at compile time.
  2. Invoke `cargo build --release --target <TARGET> --manifest-path ../sidecar-src/Cargo.toml`, then copy the resulting executable into `src-tauri/binaries/example-sidecar-<target-triple>[.exe]`, chmod 0755 on Unix. Skips rebuild when the destination is fresher than `sidecar-src/src/main.rs` (avoids `tauri dev` rebuild loops).
- `src-tauri/src/lib.rs` — Tauri app entry. Registers `tauri-plugin-shell`, defines `get_app_version` and `run_sidecar` commands, drives the `CommandEvent` stream and exit-code handling.
- `sidecar-src/src/main.rs` — the sidecar CLI itself. Parses `--greet <name>` / `--version`, prints to stdout, returns non-zero on unknown args. Replace with real CLI logic per project.
- `sidecar-src/Cargo.toml` — release profile is `lto = true, codegen-units = 1, opt-level = "s", strip = true, panic = "abort"` for minimal binary size.
- `package.json` — frontend scripts (`dev`, `build`, `test`) and `tauri:dev` / `tauri:build` proxies. Version is mirrored from `tauri.conf.json`.

## Build & Release Flow

Per-target build (`cargo build` / `tauri build` triggered by Tauri CLI or CI):

1. Cargo invokes `src-tauri/build.rs` for the host crate.
2. `build.rs` shells out to `cargo build --release --target <TARGET>` against `sidecar-src/Cargo.toml`, producing `sidecar-src/target/<TARGET>/release/example-sidecar[.exe]`.
3. The binary is copied to `src-tauri/binaries/example-sidecar-<TARGET>[.exe]` — the exact filename Tauri's bundler expects given `externalBin: ["binaries/example-sidecar"]`.
4. `tauri_build::build()` then runs the standard Tauri build, which embeds the platform-matching sidecar into the bundle (DMG / NSIS / DEB / AppImage per `bundle.targets`).
5. At runtime, `tauri-plugin-shell` resolves `"example-sidecar"` to the bundled binary, gated by the `default.json` capability.

Release CI (`.github/workflows/release-official.yml`):

- Matrix across `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`.
- A `prepare` job validates the resolved tag against strict semver before any artifact is produced. Resolution priority: `inputs.tag` → `refs/tags/v*` ref → `version` field in `src-tauri/tauri.conf.json`. The workflow deliberately **does not** fall back to `github.ref_name` (would yield a `vmain` release on dispatch from `main`; guard documented in-workflow).
- Each matrix leg runs `npm ci || npm install --no-fund --prefer-offline`, installs Linux deps on Ubuntu, then `tauri-apps/tauri-action@v0` with `TAURI_RELEASE=true` so `APP_VERSION` is stamped without the `[DEV]` suffix.
- Build artifacts (per-platform installers carrying the embedded sidecar) are attached to a draft release; `end-release` finalizes it.
- `release-beta.yml` mirrors the official flow with beta-tagged outputs.
