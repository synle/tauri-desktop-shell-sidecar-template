# Tauri Desktop Shell Sidecar Template

Skeleton template for a cross-platform desktop app using **Tauri v2** + a **shell binary sidecar** + **React 19** (TypeScript) + **MUI v9** + **Vite 6** + **Vitest 4**. The Tauri Rust shell launches a bundled native CLI binary on demand via `tauri-plugin-shell`'s sidecar API and captures its stdout.

Use this template when you have a CLI you want to ship alongside your app — your own Rust CLI (like the [display-dj-cli](https://github.com/synle/display-dj-cli) pattern), a Go binary, a `ffmpeg`-style tool, etc.

This template ships with an example Rust CLI in `sidecar-src/` that supports `--greet <name>` and `--version`. `src-tauri/build.rs` builds it for the current target and copies it to `src-tauri/binaries/example-sidecar-<target-triple>(.exe)` so Tauri's `externalBin` config can find it.

Two starter pages — **Home** (calls `run_sidecar(['--greet', 'world'])`) and **Settings** — wired up via React Router (HashRouter).

## Requirements

| Tool | Version | Notes |
|------|---------|-------|
| Node.js | 20+ | Use `fnm` / `nvm` to pin |
| npm | 10+ | Ships with Node |
| Rust | stable | `rustup default stable` (also builds the sidecar) |
| Tauri prereqs | — | See [tauri.app prerequisites](https://tauri.app/start/prerequisites/) |

Platform-specific extras:

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft C++ Build Tools, WebView2 (preinstalled on Win11)
- **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev libssl-dev`

## Getting started

```bash
git clone <this-repo>
cd tauri_desktop_shell_sidecar_template
npm install            # JS dependencies
npx tauri dev          # Builds the sidecar + runs the desktop app
```

The first `tauri dev` build will compile the example sidecar — that takes a minute. Subsequent builds skip the sidecar rebuild unless `sidecar-src/` changes.

Useful scripts:

```bash
npm run dev            # Vite dev server only (browser at http://localhost:1420)
npm run build          # Production frontend build
npm test               # Vitest run
npm run typecheck      # tsc --noEmit
npm run tauri:build    # Production desktop build (.dmg/.exe/.deb/.AppImage)
cd src-tauri && cargo test     # Rust tests for the Tauri shell
cd sidecar-src && cargo test   # Rust tests for the sidecar binary
```

## Project layout

```
.
├── src/                       # React frontend
│   ├── components/NavBar.tsx
│   ├── pages/                 # HomePage (calls run_sidecar), SettingsPage
│   ├── test/setup.ts          # Vitest setup (mocks Tauri APIs)
│   ├── App.tsx
│   └── main.tsx
├── sidecar-src/               # Example sidecar binary (Rust)
│   ├── Cargo.toml
│   └── src/main.rs            # CLI: --greet <name> | --version
├── src-tauri/                 # Tauri Rust shell
│   ├── src/lib.rs             # `run_sidecar` Tauri command
│   ├── build.rs               # Builds + stages the sidecar binary
│   ├── binaries/              # Build output: example-sidecar-<target>
│   ├── capabilities/          # Allows shell:allow-execute on the sidecar
│   └── tauri.conf.json
├── vite.config.ts
└── .github/workflows/         # build, release-official, release-beta
```

## How the sidecar wires up

1. **Build** (`src-tauri/build.rs`): when `cargo build` runs (called by `tauri dev` / `tauri build`), the build script invokes `cargo build --release --target <triple>` inside `sidecar-src/`, then copies the resulting binary to `src-tauri/binaries/example-sidecar-<triple>(.exe)`.
2. **Bundle** (`src-tauri/tauri.conf.json` → `bundle.externalBin`): Tauri ships the binary alongside the app, picking the one whose suffix matches the build target.
3. **Invoke** (`src-tauri/src/lib.rs`): the `run_sidecar` Tauri command resolves the binary via `app.shell().sidecar("example-sidecar")`, runs it with the supplied args, and streams `stdout` / `stderr` via `CommandEvent`.
4. **Permissions** (`src-tauri/capabilities/default.json`): `shell:allow-execute` allows-listing only `binaries/example-sidecar` (with `sidecar: true` and `args: true`).

## Replacing the example sidecar

Two paths:

- **Keep `sidecar-src/`** — edit it to your CLI's source, keep `[[bin]].name = "example-sidecar"` (or rename it everywhere — see "What to change" below).
- **Pre-built sidecar** — delete `sidecar-src/`, replace `build.rs::build_sidecar()` with a download-from-GitHub-releases step (see `display-dj/src-tauri/build.rs` for an example), and check binaries into `src-tauri/binaries/` as a network-failure fallback.

## Versioning & release

The version lives in **`src-tauri/tauri.conf.json` → `version`**. `build.rs` exposes it as `APP_VERSION`. Dev builds append `[DEV]`; CI release builds set `TAURI_RELEASE=true`.

- **Build CI** (`.github/workflows/build.yml`) — runs on every push/PR to `main`. Tests + builds on macOS (ARM + Intel), Windows, Linux.
- **Official release** (`.github/workflows/release-official.yml`) — `v*` tag or `workflow_dispatch`.
- **Beta release** (`.github/workflows/release-beta.yml`) — manual `workflow_dispatch` only.

## What to change after cloning

1. Rename in `package.json` (`name`, `description`).
2. Rename in `src-tauri/Cargo.toml` (`[package].name`, `[lib].name`) and `src-tauri/src/main.rs` (`app_lib::run()`).
3. Update `src-tauri/tauri.conf.json` (`productName`, `identifier`, `windows[].title`, `version`).
4. Replace icons in `src-tauri/icons/` (`npx tauri icon path/to/icon.png`).
5. Rename the sidecar binary: search-and-replace `example-sidecar` across `tauri.conf.json`, `capabilities/default.json`, `build.rs`, `lib.rs`, and `sidecar-src/Cargo.toml`.
6. Replace `sidecar-src/src/main.rs` with your real CLI.
7. Update `.github/workflows/release-*.yml` `project_name`.

## License

MIT — add a `LICENSE` file if you publish.
