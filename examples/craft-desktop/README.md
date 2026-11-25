# CANexus Studio virtual analyzer

This desktop preview captures virtual Classic CAN and CAN FD in RAM. Start a session with **Start**, stop and drain it with **Stop**, and select a sequence in **Trace** to inspect its full payload. Sort with the column headings, filter by hexadecimal CAN ID or frame format, and toggle **Follow** to browse older rows. **Freeze view** holds the visible history while capture continues. The latest 512 rows are retained, and the status shows accepted frames, RAM admission rejections, receiver loss and history eviction separately.

## Install

Download the appropriate artifact from a successful **Virtual desktop app** run in GitHub Actions. Extract its ZIP.

- Windows x64 with Direct3D 12: run the setup executable. It installs per user and creates a Start menu entry. Preview installers are unsigned.
- Ubuntu 24.04 amd64: run `sudo apt install ./CANexus-Studio-0.1.0-linux-amd64.deb`, then open **CANexus Studio** from the desktop menu.

## Build and test from source

```sh
cargo run --locked --manifest-path examples/craft-desktop/Cargo.toml --release
cargo test --locked --manifest-path examples/craft-desktop/Cargo.toml
```

Use the repository's Rust 1.100.0 toolchain. Linux needs an X11 or Wayland desktop and a working OpenGL stack. The workflow builds native packages and runs `--smoke` on the packaged app. This mode starts virtual capture, checks Classic and FD frames, stops the session and exits. The normal app starts with capture stopped.

This preview uses the pinned Craft, egui and winit family from DEV-024. The crate remains isolated from the legacy GUI workspace while native Windows interaction, Wayland and accessibility acceptance for the production shell are tracked separately. The preview scope is virtual analysis and RAM-only retention.
