# OpenSpace Architecture

Status: v0.1 architecture guardrails
Primary source of truth: `DESIGN.md`
Target release: M1 Private Alpha
Primary platform for v1: macOS

This document summarizes the architecture guardrails for OpenSpace. The detailed product requirements, user stories, acceptance criteria, release cutline, and implementation milestones live in `DESIGN.md`.

## Architecture intent

OpenSpace is a desktop AI assistant for project work. It is built around three main workspace modes: Terminal, Chat, and Editor. Each mode can become the main CenterSurface for a Session while the surrounding shell remains stable.

The architecture must support real project workflows, not only conversational UI. Terminal processes, chat streams, editor buffers, file watchers, language-server processes, AI tasks, and git state must survive mode switching.

## Main shell model

OpenSpace uses one stable app shell:

- TopBar
- LeftPanel
- CenterSurface
- RightPanel
- StatusBar

The CenterSurface renders the active mode for the active Session:

- Terminal Mode renders a terminal workspace.
- Chat Mode renders an AI workflow surface.
- Editor Mode renders an editing workspace.

V1 does not support arbitrary mixed main panes where Terminal, Chat, and Editor compete as equal center panes. Supporting panels may expose cross-feature context, but the main workspace is owned by the active mode.

## Session model

A Session owns:

- active mode
- permission profile
- mode-specific surface descriptors
- context state
- restoration descriptors

Mode is not a global app toggle. Permission is not a global app toggle. Both are Session-scoped so different work sessions can restore their own behavior.

## Workspace crate direction

OpenSpace uses a feature-first Rust workspace. The app shell composes features; feature crates do not import each other directly. Within each feature crate, code is organised along Clean Architecture lines so domain logic stays independent of Iced and platform adapters:

```
domain          // pure value types and contracts (no Iced, no I/O)
application     // use cases, reducers, command catalogues
infrastructure  // OS adapters, persistence, transports
presenter       // Iced view + canvas programs
```

Layer folders stay short (`domain`, `application`, `infrastructure`, `presenter`) — the crate name already supplies the feature scope. Files inside each layer keep the feature prefix (`welcome_state.rs`, `home_audit.rs`, `terminal_commands.rs`) so the file layout stays discoverable from `find` / IDE navigation. Sub-feature files inside the home shell follow the same rule with the sub-feature prefix.

Crate map:

- `openspace-core`: shared IDs, contracts, events, commands, permissions, context, actions, audit, runtime status. Files use the `core_*` prefix.
- `openspace-theme`: semantic theme tokens and style helpers (`theme_palette`, `theme_tokens`).
- `openspace-platform`: platform capability detection and policies.
- `openspace-storage`: SQLite persistence, config, cache, migrations (`storage_*`).
- `openspace-secrets`: keychain, environment fallback, memory adapter, redaction (`secrets_*`).
- `openspace-welcome`: first-run welcome window. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-home`: post-welcome workspace shell — top bar, panels, center surface, command palette, app router, runtime manager, audit sinks. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-app`: composition root. Mounts the welcome → home routing pipeline and runs the Iced event loop. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-terminal`: PTY, terminal emulator state, tabs, splits, snapshots. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-chat`: chat workflow state, messages, streaming lifecycle, action cards. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-ai`: provider abstraction, streaming, model registry, ContextPack assembly, action proposal mapping. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-fs`: project tree, watcher, indexing, search metadata. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-git`: git status, diff, stage, unstage, commit, commit-message context. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-editor`: buffers, file tabs, syntax highlighting, dirty state, AI review surface. Layered: `domain`, `application`, `infrastructure`, `presenter`.
- `openspace-lsp`: language-server process lifecycle, diagnostics, completion and code-intelligence boundaries. Layered: `domain`, `application`, `infrastructure`, `presenter`.

Composition flows top-down:

```
openspace-app
 ├── openspace-welcome   (first-run welcome window)
 └── openspace-home      (post-welcome workspace shell)
      └── sub-features (terminal, chat, editor, ai, fs, git, lsp)
```

Shared crates sit beside this tree and are pulled in by whichever feature needs them.

## Dependency rules

- Feature modules may depend on core contracts.
- The app shell may depend on feature modules as the composition root.
- Feature modules must not depend laterally on each other.
- Cross-feature coordination flows through core contracts and app routing.
- Long-lived feature state stays in the owning feature module.
- Platform-specific decisions go through platform policy boundaries.
- Storage and secrets behavior is routed through storage/secrets boundaries, not scattered across feature internals.

## File and module naming rules

- Use feature-first organization. Each feature crate is named `openspace-<feature>`.
- Within each feature crate, source code is split into four short layer modules: `domain`, `application`, `infrastructure`, `presenter`. The crate name supplies the feature scope, so the layer folders themselves stay unprefixed.
- Files inside each layer use the feature prefix (`welcome_state.rs`, `home_audit.rs`, `terminal_commands.rs`).
- Sub-feature files use the sub-feature prefix when the sub-feature lives inside another feature crate (`chat_messages.rs` inside `openspace-chat`).
- Create files by scope when the scope has real content. Avoid large generic catch-all files.
- Public interfaces expose stable contracts outward (re-exports at `lib.rs`); implementation details remain inside the owning layer.
- Shared crates (`openspace-core`, `openspace-theme`, `openspace-storage`, `openspace-secrets`) follow the same file-prefix rule (`core_audit.rs`, `theme_tokens.rs`, `storage_handle.rs`, `secrets_redaction.rs`) but stay flat — they do not impose their own four-layer structure on consumers.

The folder structure is a discoverability tool. It is not a claim that the entire project follows a broad enterprise-layer architecture.

## Event and command model

OpenSpace uses coarse AppEvent and AppCommand routing for shell orchestration and cross-feature coordination.

App-level events are appropriate for:

- workspace changes
- project changes
- Session mode changes
- permission changes
- AI task lifecycle
- chat workflow lifecycle
- terminal lifecycle and layout changes
- editor file and diagnostics changes
- git state changes
- file system project metadata changes
- storage migration and restore events
- audit, notification, and error events

High-frequency internals stay private to features. These should not flood app-level events:

- raw PTY bytes
- raw AI token deltas per token
- cursor blink
- per-frame animation state
- raw file watcher noise
- raw language-server protocol packets
- incremental parser internals

## Runtime model

- Iced owns shell/UI state.
- Feature actors/services own long-lived runtime state.
- A shared async runtime drives feature services.
- The app dispatches commands to feature handles.
- Feature services emit coarse events and snapshots back to the app.
- Rendering reads view models or snapshots, not raw runtime internals.

Avoid putting every subsystem state directly inside the Iced app state. The shell should compose feature snapshots and issue commands; feature runtimes should own their long-lived processes and data streams.

## Terminal architecture guardrails

Terminal Mode is a first-class main workspace mode.

V1 must support:

- PTY input/output on macOS
- terminal emulator state
- keyboard focus and input routing
- terminal tabs
- horizontal and vertical splits
- basic layout descriptors for restore
- runtime survival across mode switches
- audit for workflow-executed commands

Raw PTY data remains inside the terminal feature. The app receives terminal lifecycle, layout, focus, status, and audit-worthy events.

## Chat architecture guardrails

Chat Mode is an AI workflow surface.

V1 must support:

- chat thread creation
- message send lifecycle
- streaming response lifecycle
- ContextPack inspection
- action cards
- approval and rejection states
- audit for AI tasks and decisions

Chat does not call providers directly. Chat uses the AI runtime and routes proposed actions through permission policy.

## Editor architecture guardrails

Editor Mode is lightweight but real.

V1 must support:

- open/edit/save for text files
- file tabs
- dirty state
- rope-based text buffers
- parser-backed syntax highlighting where available
- diagnostics display
- AI review surface
- diff apply/reject flow through permission and audit

Editor Mode is not full IDE parity in v1.

## LSP architecture guardrails

The LSP module owns language-server process lifecycle and protocol detail.

V1 must support basic diagnostics and AI-assisted diagnostic explanation. Raw protocol packets do not become app-level events.

## AI and ContextPack guardrails

The AI runtime owns provider abstraction, streaming lifecycle, model/provider configuration, and action proposal normalization.

ContextPack uses explicit anchors plus safe automatic enrichment. It must be inspectable before use, with source, scope, reason, content kind, freshness, token estimate, and permission decision visible to the user.

AI output proposes actions. OpenSpace gates execution.

## Storage, secrets, and audit guardrails

- SQLite stores dynamic app state.
- Human-editable config stores non-secret settings and secret references.
- Cache stores derived artifacts.
- OS keychain is the default secret store.
- Environment variables are development fallback.
- Child process environments do not automatically inherit provider secrets.
- Important AI, permission, terminal, file, git, editor, storage, and security-sensitive events are audited.
- Raw secrets, raw authorization headers, raw PTY output, raw AI token streams, and noisy protocol traffic are not logged by default.

## Testing guardrails

Tests should verify external behavior and boundary contracts, not private implementation details.

V1 test layers:

- core policy and value-object tests
- feature integration tests with fakes and temp resources
- app routing tests for command/event/permission/audit behavior
- manual macOS smoke checklist

Real AI network calls, real keychain use, full UI automation, GPU snapshots, and cross-platform release matrices are not required for v1 tests.

## Implementation sequence

1. Workspace skeleton and crate boundaries.
2. Shell, Session, and mode skeleton.
3. Terminal risk slice.
4. AI and Chat workflow slice.
5. File system and Git context slice.
6. Lightweight Editor and LSP slice.
7. M1 Private Alpha hardening.
