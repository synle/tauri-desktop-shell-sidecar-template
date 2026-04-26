# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Project Overview

A **skeleton template** for cross-platform desktop apps built with **Tauri v2** + a **shell-binary sidecar** (a native CLI shipped alongside the app) + **React 19** (TypeScript) + **MUI v9** + **Vite 6**. The Tauri Rust shell launches the bundled CLI on demand via `tauri-plugin-shell`'s sidecar API and captures its stdout.

This template includes an example sidecar in `sidecar-src/` (a Rust CLI), built and staged by `src-tauri/build.rs`. Replace it with your real CLI (Rust, Go, anything that produces a single binary) — or swap the build script for a download-from-GitHub-releases step like [display-dj-cli](https://github.com/synle/display-dj-cli).

Sister templates (in adjacent directories):

- `tauri-desktop-raw-template` — plain Tauri (no sidecar)
- `tauri-desktop-node-sidecar-template` — Tauri + Node.js/Express sidecar

## Build commands

```bash
npm install                # JS dependencies
npx tauri dev              # Builds the sidecar + runs the app in dev mode
npm run dev                # Vite frontend only (browser mode)
npm run build              # Production frontend build
npx tauri build            # Production desktop build
npm test                   # Vitest (run once)
cd src-tauri && cargo test     # Rust tests for the Tauri shell
cd sidecar-src && cargo test   # Rust tests for the sidecar
```

## Architecture

Three layers:

- **`src/` (React + TS)** — UI built with MUI v9. Routes via React Router (`HashRouter`). Calls `invoke('run_sidecar', { args })` to execute the bundled CLI and read its stdout.
- **`sidecar-src/` (CLI source)** — the example is a Rust crate that produces `example-sidecar`. Tauri requires platform-specific binary names (e.g. `example-sidecar-x86_64-apple-darwin`); `src-tauri/build.rs` handles the rename.
- **`src-tauri/` (Tauri shell)** — declares the sidecar in `tauri.conf.json` → `bundle.externalBin` and `plugins.shell.scope`. The capability `shell:allow-execute` allow-lists running it with arbitrary args. The `run_sidecar` Tauri command uses `app.shell().sidecar(name).args(...).spawn()` and consumes `CommandEvent`s for stdout/stderr/exit.

### Sidecar build flow

1. `src-tauri/build.rs::build_sidecar()` runs every `cargo build` for the Tauri shell.
2. It runs `cargo build --release --target <triple> --manifest-path ../sidecar-src/Cargo.toml`.
3. Copies the output to `src-tauri/binaries/example-sidecar-<triple>(.exe)`.
4. Sets `+x` on Unix targets.
5. Skips the rebuild if the destination is newer than `sidecar-src/src/main.rs` (avoids `tauri dev` infinite watch loop).

## Versioning

The single source of truth is **`src-tauri/tauri.conf.json` → `version`**. `build.rs` exposes it as `APP_VERSION`. Dev builds append `[DEV]`; CI release builds set `TAURI_RELEASE=true`.

## Conventions

- All Rust structs sent to the frontend use `#[serde(rename_all = "camelCase")]`.
- Tauri commands are `snake_case` in Rust, called with `snake_case` strings from `invoke()`.
- The `sidecar-src/` crate must keep `[[bin]].name = "example-sidecar"` (or update every reference; see README).
- Always add tests for new code: components get `*.test.tsx`, Rust modules get `#[cfg(test)] mod tests`.

## CI / Release Workflows

- **`build.yml`** — runs on every push/PR to `main`, runs `npm test` and `cargo test` then builds the Tauri bundle (which builds the sidecar via `build.rs`). Posts a PR comment with artifact download links.
- **`release-official.yml`** — `v*` tag pushes or manual `workflow_dispatch`.
- **`release-beta.yml`** — manual `workflow_dispatch` only.

Use the `/release-official` and `/release-beta` slash commands to trigger interactively.

## What to update when adapting this template

1. `package.json` → `name`, `description`
2. `src-tauri/Cargo.toml` → `[package].name`, `[lib].name` (and `src-tauri/src/main.rs` to match)
3. `src-tauri/tauri.conf.json` → `productName`, `identifier`, `windows[].title`, `version`, and the `binaries/example-sidecar` strings if renaming
4. `src-tauri/capabilities/default.json` → matching sidecar name
5. `src-tauri/icons/` → replace
6. `sidecar-src/` → replace with your CLI source (keep `[[bin]].name` matching the sidecar name)
7. `.github/workflows/release-*.yml` → `project_name`

## GitHub Raw File URLs

Always use the `?raw=1` blob URL format: `https://github.com/{owner}/{repo}/blob/head/{path}?raw=1`.

Do NOT use `api.github.com/repos/.../contents/` or `raw.githubusercontent.com`.

## Git / PR Merge Policy

- Always use **squash and merge** for PRs.
- **Always rebase before pushing** (`git pull --rebase` before `git push`).
