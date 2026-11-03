# Dockable Plugin Panels and Configurable Commands

**Revision:** 0.7 · **Status:** Accepted product decisions and proposed design

## Accepted requirements

| ID | Proposed priority | Status | Requirement |
|---|---|---|---|
| REQ-PLG-012 | P1 | Accepted | Public UI plugin SDK enables independent extensions to contribute their own dockable Slint panels to pbSlintDock workspaces. |
| REQ-CMD-001 | P1 | Accepted | Provide a fully configurable command system: core and plugin commands, keyboard shortcuts, macros, context-menu actions and toolbar customization. |

## Proposed requirements

| ID | Status | Requirement |
|---|---|---|
| REQ-PLG-013 | Proposed | UI plugin panel contracts expose stable panel IDs, state serialization, lifecycle notifications and capability limits. |
| REQ-PLG-014 | Proposed | Docked/floating panel re-instantiation does not own or reset protocol session state; missing plugin panels leave recoverable placeholders. |
| REQ-CMD-002 | Proposed | Commands carry stable IDs, argument schemas, availability conditions, user-visible descriptions and permission requirements. |
| REQ-CMD-003 | Proposed | Keybindings resolve conflicts predictably across scopes and provide an editor showing collisions. |
| REQ-CMD-004 | Proposed | Macros compose existing commands asynchronously and support cancellation/timeout and logs; privilege checks apply to each action. |
| REQ-CMD-005 | Proposed | Local command/UI customization is user-specific; explicitly shared presets may be stored inside the single portable project file. |
| REQ-CMD-006 | Proposed | Command palette, toolbar, menus, context menus, scripts and future remote APIs call the same command dispatcher instead of reimplementing actions. |

## Acceptance criteria

- [ ] Build an external Slint example plugin, add its dockable panel, move to floating and back, close and reopen; its persistent state and data subscriptions remain coherent.
- [ ] Disable an installed UI plugin; workspace reload shows a recoverable missing-panel placeholder rather than destroying saved layout.
- [ ] Bind a shortcut to a built-in command and a plugin command, detect a conflict, resolve it and verify it persists locally without modifying the ECU Git checkout.
- [ ] Create a macro invoking commands from two modules and verify order, authorization, cancellation and diagnostic output.
- [ ] Attempt to invoke a flashing or transmitting command without permission through a shortcut/macro; dispatcher rejects it, just as it would from the original UI.

## Conflicts and open implementation decisions

- Slint dynamic component model and plugin ABI may limit runtime third-party component loading; confirm with a working proof of concept before finalizing SDK contracts.
- Trusted native UI plugins can crash the host; choose isolation/trust policy and document inability to guarantee in-process crash containment.
- Hotkeys conflict with text input and OS shortcuts; require scoped precedence rules and platform tests.
- Fully customizable does not mean arbitrary unmanaged code execution; expose controlled command objects and typed UI extensions.
- Define minimum command macro expressiveness versus Python automation; avoid competing, independently implemented schedulers.
