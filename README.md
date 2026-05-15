# OpenSpace

OpenSpace is a macOS-first desktop AI assistant for project work. It is not a desktop chat wrapper. It is designed around three primary work modes: Terminal, Chat, and Editor. Each mode can become the main workspace surface for a Session, while the surrounding shell keeps project, context, permission, model, and task state visible.

The first release target is Milestone M1: Private Alpha. It should be usable by the project owner for lightweight real work on macOS before any public beta effort begins.

## Current status

OpenSpace is currently in product and architecture definition. The first implementation step will be a thin Rust workspace skeleton after `docs/DESIGN.md` is reviewed.

## Design source of truth

Read `docs/DESIGN.md` for the product requirements, v1 cutline, acceptance criteria, technical guardrails, implementation milestones, and future scope.

## Architecture summary

OpenSpace will use:

- Rust as the core engine.
- Iced as the desktop interface engine.
- A feature-first Cargo workspace.
- A stable app shell with a mode-switching center surface.
- Long-lived feature runtimes driven by a shared async runtime.
- A coarse app event/command bus with private high-frequency streams inside features.
- SQLite, human-editable config files, and a cache directory for persistence.
- OS keychain-backed secrets with environment-variable fallback for development.
- User-visible audit/action history for important AI, permission, terminal, file, git, and storage events.

## Release target

V1 means Private Alpha, not public beta.

The v1 target is a macOS build that supports:

- Terminal Mode with PTY input/output, tabs, splits, and basic session restore.
- Chat Mode with streaming AI, ContextPack inspection, action cards, and approvals.
- Editor Mode with file open/edit/save, syntax highlighting, diagnostics, and AI review flow.
- Project file tree, basic search, git status/diff/stage/commit, and AI commit-message proposal.
- Permission profiles, keychain secrets, audit log, and SQLite-backed restore.

Linux and Windows are kept behind platform adapter boundaries, but they are not v1 release blockers.

## Build and run

Build and run instructions will be added after the Phase 0 Cargo workspace skeleton exists.
