# Initial Rust workspace boundaries

DEV-002 establishes a deliberately small workspace, without prematurely fixing HAL, event bus, UI or plugin ABIs.

- `canexus-core`: transport-neutral shared primitives and application service foundations. Must not depend on GUI or hardware SDKs.
- `canexus-cli`: headless executable, currently only a bootstrap smoke entry point. Depends on core.
- Future transport-specific CAN types, HAL adapters, recorder and Slint UI should become separate modules/crates after their interfaces are validated in subsequent tasks.

Dependency direction: executable -> core. Core must not import CLI, Slint or vendor SDKs. No asynchronous runtime or native GUI dependency is selected in DEV-002.

## Local verification

Install Rust via rustup (toolchain version pinned in `rust-toolchain.toml`) and run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --all-targets
cargo test --workspace --all-targets
cargo run -p canexus-cli
```

GitHub Actions runs the checks on Ubuntu and Windows. No hardware, Slint or performance guarantees are implied by these checks.
