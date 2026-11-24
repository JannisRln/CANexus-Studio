# DEV-024 Craft desktop proof

Run the isolated workspace with Rust 1.100.0.

```sh
cargo run --locked --manifest-path examples/craft-desktop/Cargo.toml
cargo test --locked --manifest-path examples/craft-desktop/Cargo.toml
```

The window contains a fixed CAN fixture table and a sampled plot in two Craft dock panels. It offers the egui theme control and an in-memory layout snapshot. The layout test checks the serialized split and panel identities. With `--smoke`, the window closes after drawing three UI frames.

The dependency experiment used the revision in [the ecosystem review](../../docs/development/CRAFT-ECOSYSTEM.md). The unpatched egui 0.37.0 build fails on Craft's `egui::ime_context`, `Style::right_to_left`, and `IMEOutput` calls. Using Storytold's pinned egui family with registry winit also fails, including at `winit::platform::latin_layout` and `WindowEvent::ImeResetCompleted`. The manifest pins the matched egui and winit forks and locks resolution for repeatable checks. It enables Glow, X11, Wayland, default egui fonts, and AccessKit.

Rustfmt, Clippy, build, and the layout test passed locally on Linux. The local sandbox did not allow Xvfb to open its Unix listening socket. The isolated workflow checks Windows and Linux builds and runs an X11 window smoke test on Linux. Windows native interaction, Wayland launch, screen-reader use, release performance, and complete dependency license review are separate DEV-024 evidence. The fixture has no live Core connection.

The pinned Craft, egui, and winit projects publish their license and notice files in their source repositories. See the [Craft notice](https://github.com/storytold/craft-libs/blob/249a41bd9306a1b34330ac94e596a1a8b881378a/NOTICE), [egui license information](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a), and [winit license information](https://github.com/storytold/winit/tree/a517b3846ee8457d84ee4975110bfda6797c44dc).
