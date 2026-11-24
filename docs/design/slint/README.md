# CANexus Studio · Slint Fluent design foundation

**Status:** Prototype / proposed. This directory does not yet form a built Rust or pbSlintDock application.

## Architecture

- `theme.slint` owns runtime theme **preference** and exposes color/spacing aliases backed by the standard `Palette` and `StyleMetrics`.
- `components/` contains first small reusable CANexus-specific components.
- `theme-preview.slint` demonstrates switching without replacing the Slint widget style.

**Build with the `fluent` style**, not `fluent-light` or `fluent-dark`, so the runtime `Palette.color-scheme` override can be used. Slint 1.16+ uses Fluent by default on all target platforms. Explicitly configure `fluent` in the future Rust build pipeline for reproducibility.

Run the preview with a compatible Slint viewer, for example:

```sh
SLINT_STYLE=fluent slint-viewer docs/design/slint/theme-preview.slint
```

A Slint viewer and Rust toolchain are **not** currently provided by this repository; the preview has not been compiler-verified here.

## Theme behavior and persistence contract

Preference values are **0 System**, **1 Light**, **2 Dark**. `CxTheme.set-preference(mode)` sets the built-in `Palette.color-scheme` to `unknown`, `light`, or `dark` respectively, then emits `preference-changed(mode)`.

- The host Rust application will load a **user-local** preference on startup, validate it (unknown values -> System), and call `set-preference`.
- When the callback fires after user interaction, the host persists `theme_preference` **outside** every ECU/worktree repository.
- In System mode, Slint follows the OS color scheme. A manual override remains in force until System is selected again.
- No window recreation, capture/recording restart, or ECU operation is part of switching themes.
- Components should prefer `Palette` and `StyleMetrics` for background/text/borders. CANexus semantic status/plot colors are supplementary and must be contrast-tested in both modes.
- Plugins must use the host's published theme contract, not install global palette overrides. The public plugin SDK/lifecycle is still subject to the existing feasibility work.

## Planned integration

DEV-002 (Rust workspace) and DEV-003 (Slint/pbSlintDock integration) still precede production integration. On both Windows and Linux verify:
1. System -> Light -> Dark -> System changes standard Fluent controls and CANexus panels in-place.
2. The initial OS color scheme and OS changes in System mode are reflected.
3. User-local persistence survives restart and never changes the ECU Git checkout.
4. Text, semantic statuses, focus rings, high-DPI and keyboard access remain legible.
5. Existing capture/recorder/measurement state remains unchanged by the theme switch.

**Sources:** [Slint Widget Styles](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/), [Slint Palette](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/globals/palette/).

The existing `docs/design/tokens.json` and HTML prototype remain historical visual-reference artifacts, **not** the runtime source of truth. The Fluent-backed .slint tokens are the proposed implementation direction.
