# TermiSSH

TermiSSH is a Rust-based desktop SSH client focused on operational workflows.  
It provides a modern GUI (`iced`), multi-tab connections, integrated terminal rendering, and a relay process for SSH session transport.

## Core Capabilities

- Multi-host SSH management with local persistence
- Multi-tab terminal sessions in a single window
- Embedded terminal bridge (`termissh-relay`) for interactive shells
- Theme/language settings and API-based host synchronization
- Basic remote structure listing for connected hosts
- Cross-platform release packaging via GitHub Actions

## Architecture

- `termissh`: GUI application and session orchestration
- `termissh-relay`: SSH relay binary used by GUI tabs
- `src/ui/*`: toolbar, sidebar, tabs, status bar, dialogs
- `src/terminal/*`: relay discovery and process bridge

## Requirements

- Rust stable toolchain (`cargo`, `rustc`)
- Windows or Linux
- Optional for Linux static build: `musl-tools`

## Development

```bash
cargo run --bin termissh
```

## Build

```bash
cargo build --release --bin termissh --bin termissh-relay
```

Main outputs:

- `target/release/termissh(.exe)`
- `target/release/termissh-relay(.exe)`

## Configuration

- Optional `.env`:

```env
API_URL=https://termissh.org
```

- Runtime settings (hosts, API key, language, theme) are stored in `config.json` under the OS-specific application config directory resolved by `directories::ProjectDirs`.

## Release Automation

This repository is configured to publish releases from Git tags.

- Workflow: `.github/workflows/build.yml`
- Trigger: push tags matching `v*` (example: `v0.2.1`)
- Build matrix: Linux (`x86_64-unknown-linux-musl`) and Windows (`x86_64-pc-windows-msvc`)
- Artifacts: `.tar.gz` / `.zip` + `SHA256SUMS.txt`
- Release publication: automatic GitHub Release asset upload

You can use:

```bash
bash release.sh v0.2.1
```

`release.sh` creates and pushes the tag, then the workflow builds and publishes release assets automatically.

## Contact

- Website: https://termissh.org
- Email: hacimertgokhan@gmail.com
