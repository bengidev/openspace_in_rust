# OpenSpace DESIGN

Status: v0.1 source of truth
Format: Product PRD with architecture guardrails
Target release: Milestone M1, Private Alpha
Primary platform for v1: macOS

This document is the initial source of truth for OpenSpace. It captures the product intent, v1 release cutline, user stories, acceptance criteria, technical guardrails, implementation sequence, and future scope.

OpenSpace is still early. This document is intentionally comprehensive so the first Rust workspace skeleton can be built against stable decisions instead of drifting feature by feature. It is not an API reference and it should not freeze every internal type before implementation gives feedback.

## 1. Product overview

OpenSpace is a desktop AI assistant for project work. It is built around three main work modes:

- Terminal Mode
- Chat Mode
- Editor Mode

OpenSpace is not a desktop chat wrapper. Chat is one mode, not the app itself. The app must also support terminal-first workflows, project-aware editing, file and git context, permission-aware AI action proposals, and auditable local work.

The key product idea is that a Session can make Terminal, Chat, or Editor the main workspace surface. The surrounding shell stays stable so the user never loses project, context, model, permission, task, and audit awareness while switching modes.

## 2. Product principles

### 2.1 Three main modes, not mixed main panes

Terminal, Chat, and Editor are primary modes. A Session has one active main mode at a time.

OpenSpace must not become an arbitrary mixed-pane desktop where Terminal, Chat, and Editor all compete as equal main panes in the same center layout. Side panels may show supporting context across features, but the CenterSurface belongs to the active mode.

### 2.2 Terminal can be the main workspace

Terminal Mode must be able to feel like the primary workspace, not like a bottom drawer. Terminal processes must survive mode switching.

### 2.3 Chat is a workflow surface

Chat Mode must support AI conversation, but it must also support workflow concepts: context inspection, action cards, approval states, pending actions, and audit visibility.

### 2.4 Editor is lightweight but real

Editor Mode does not need full IDE parity for v1. It does need real open/edit/save behavior, syntax highlighting, diagnostics, dirty state, and AI review/apply flow.

### 2.5 AI proposes, OpenSpace gates execution

AI output must not directly mutate files, run commands, or change git state. AI may propose actions. OpenSpace routes those proposals through Session permission profiles, approval flow, and audit logging.

### 2.6 Context must be visible and explainable

AI context must not be a hidden magic blob. ContextPack items must be inspectable with source, scope, reason, content kind, freshness, and token estimate.

### 2.7 Safety is a product requirement

Permissions, secrets, audit, redaction, and child-process environment filtering are v1 requirements. They are not polish items.

### 2.8 Feature-first organization

The codebase should be organized around product features and runtime ownership, not around broad horizontal enterprise layers. Scope folders may be used to make files discoverable, but the folder structure is a discovery aid, not a claim that every feature follows a named architecture.

### 2.9 Design tokens before advanced rendering effects

The visual system should start with stable theme tokens and a dark, developer-focused palette. Advanced frosted or glass effects are future work and must not block v1 usability.

## 3. Target users and primary use cases

### 3.1 Target users

OpenSpace v1 is for a developer-owner using macOS who wants an AI assistant that can help with real project work while keeping terminal, editor, files, git, and context visible.

### 3.2 Primary use cases

1. Open a project folder and keep its file, git, and session state available.
2. Use Terminal Mode as the main workspace and run normal shell commands.
3. Switch to Chat Mode to ask for help with the active project context.
4. Inspect what context will be sent before an AI request.
5. Let AI propose terminal commands, file edits, git operations, or explanations without executing them blindly.
6. Approve, reject, or revise AI action cards.
7. Use Editor Mode to open, edit, save, inspect diagnostics, and review AI-proposed diffs.
8. Use git status and diff views to understand project changes.
9. Generate a commit message proposal from staged changes.
10. Restart the app and restore enough state to continue working.

## 4. Release ladder

### 4.1 M0: Technical Preview

M0 is internal only and is not v1.

M0 proves that the architecture can launch, switch modes, run a terminal slice, and run a mock AI/chat workflow. It does not need product polish.

### 4.2 M1: Private Alpha

M1 is the v1 target.

V1 is done when the macOS build is usable by the project owner for lightweight real work. It must include the safety, storage, and audit foundations required for trusted AI-assisted local work.

### 4.3 M2: Public Beta

M2 is public-beta readiness. It adds onboarding, packaging polish, user-facing docs, friendlier provider setup, stronger error handling, and broader feedback readiness.

### 4.4 M3: Production

M3 is production readiness. It adds update strategy, deeper performance hardening, stronger docs and support, broader platform support, and carefully selected advanced features.

## 5. V1 Private Alpha cutline

V1 must include usable vertical slices for:

- Shell, Session, and Mode switching.
- Terminal Mode.
- Chat Mode.
- Editor Mode.
- AI runtime.
- ContextPack.
- File system and project tree.
- Git status, diff, stage, unstage, and commit.
- Storage and session restore.
- Secrets.
- Audit and developer observability.
- Command palette and keybinding foundation.
- macOS platform integration.
- Core tests and manual smoke checklist.

V1 must be safe enough for local project work. It does not need to be public polished.

## 6. V1 non-goals

The following are not v1 goals:

- Plugin system.
- Full IDE/editor parity.
- Full git client parity.
- Arbitrary mixed-mode pane layouts.
- Neovim-like command language.
- Autonomous task board or multi-agent execution system.
- Semantic embeddings or dependency graph retrieval.
- Official Linux or Windows release.
- Production updater.
- Full UI automation suite.
- Advanced glass/frosted rendering as a release blocker.
- Event-sourcing replay system.
- Encrypted portable secret vault.
- Public onboarding polish.

## 7. User experience requirements

### 7.1 Stable app shell

Product requirement: OpenSpace must use a stable app shell. Mode switching changes the CenterSurface, not the whole application frame.

Shell regions:

- TopBar
- LeftPanel
- CenterSurface
- RightPanel
- StatusBar

TopBar should expose workspace, project, Session, active mode, model, permission profile, and command palette access.

LeftPanel should expose project navigation, file tree, and lightweight project shortcuts.

CenterSurface should render the active main mode:

- TerminalWorkspaceView for Terminal Mode
- ChatWorkflowView for Chat Mode
- EditorWorkspaceView for Editor Mode

RightPanel should expose context, AI task/progress, diagnostics, git detail, inspection, and pending actions.

StatusBar should expose git branch/status, current cwd/file/language/server, permission profile, background task count, and important warnings/errors.

User stories:

1. As a user, I can see the same shell frame while switching between Terminal, Chat, and Editor modes, so that I stay oriented.
2. As a user, I can see the active permission profile and AI model from the shell, so that I understand the current risk context.
3. As a user, I can inspect context, pending actions, diagnostics, or git detail without turning those panels into competing main modes.

Done when:

- The app launches into a stable shell.
- TopBar, LeftPanel, CenterSurface, RightPanel, and StatusBar exist as stable regions.
- Switching active mode replaces only the CenterSurface mode view.
- Terminal, Chat, and Editor do not appear as arbitrary mixed main panes in v1.
- Permission, model, and Session state remain visible or directly reachable from the shell.

Technical guardrails:

- `openspace-app` owns shell UI state.
- Feature runtimes own long-lived feature state.
- The shell renders snapshots or view models, not raw feature internals.
- Side panels may show cross-feature supporting context but must not violate the single active main mode model.

## 8. Session, workspace, and mode requirements

Product requirement: A Session represents a working context. Each Session owns its active mode and permission profile.

User stories:

1. As a user, I can open a project Session, so that OpenSpace can track mode, context, permissions, and restore state.
2. As a user, I can switch a Session between Terminal, Chat, and Editor modes, so that I can work in the mode that matches my current task.
3. As a user, I can switch modes without killing terminal processes, AI tasks, file watchers, or LSP processes, so that my work continues in the background.

Done when:

- A Session can be created for an opened project folder.
- A Session stores active mode independently from global app state.
- A Session stores a permission profile independently from global app state.
- Switching mode updates CenterSurface selection.
- Mode switching does not terminate feature runtimes.
- Basic Session descriptors can be persisted and restored.

Technical guardrails:

- Mode is a Session property, not a process-global toggle.
- Permission profile is a Session property.
- Feature runtimes persist independently from CenterSurface visibility.
- The app routes mode changes through AppCommand/AppEvent rather than direct cross-feature mutation.

## 9. Permission requirements

Product requirement: Every Session has a permission profile that controls AI-assisted actions and risky operations.

Expected v1 profiles:

- Default: conservative approvals for mutating actions.
- Auto Review: lower friction for review/explanation flows while preserving gates for mutations.
- Full Access: reduced approval friction but still audited.
- Custom: explicit user-defined policy surface for later expansion.

User stories:

1. As a user, I can see the current Session permission profile, so that I understand the level of automation allowed.
2. As a user, I can approve or reject AI-proposed actions, so that AI does not mutate my project without consent.
3. As a user, I can enable a more permissive profile while still retaining an audit trail.

Done when:

- Each Session has one active permission profile.
- AI-proposed actions are evaluated against the active permission profile.
- Destructive or mutating actions are gated by permission policy.
- Full Access does not disable audit logging.
- Permission changes are recorded in audit history.

Technical guardrails:

- Permission decisions belong in core/shared policy types.
- Feature runtimes may request action execution, but the app/permission router gates execution.
- Permission policy must be testable without UI.

## 10. Workspace and crate direction

Product requirement: The Rust workspace must support feature-first development with explicit boundaries.

Initial crate direction:

- `openspace-core`: shared contracts, IDs, events, permissions, context, actions, audit types, command descriptors, runtime status types.
- `openspace-app`: Iced app shell, routing, command registry, keybinding resolver, runtime manager, snapshot cache.
- `openspace-platform`: OS capability detection and platform-specific policies.
- `openspace-theme`: theme tokens, palette, style helpers, future rendering capability flags.
- `openspace-terminal`: PTY, terminal emulator state, terminal layouts, terminal snapshots.
- `openspace-chat`: chat workflow state, messages, action cards, streaming message lifecycle.
- `openspace-ai`: provider abstraction, streaming, model registry, context assembly, action proposal mapping.
- `openspace-editor`: buffers, file tabs, syntax highlighting, editor snapshots, AI review surface.
- `openspace-lsp`: LSP client processes, diagnostics, completion lifecycle, AI-assisted LSP integration.
- `openspace-fs`: project tree, file watching, indexing, search metadata.
- `openspace-git`: git status, diff, staging, commit operations, commit-message context.
- `openspace-storage`: SQLite persistence, config, cache directories, migrations.
- `openspace-secrets`: keychain, environment fallback, memory adapter, redaction policy.

User stories:

1. As a developer, I can locate feature-owned code quickly, so that development remains fast as the app grows.
2. As a developer, I can work on a feature without importing private internals from another feature, so that boundaries remain stable.
3. As a developer, I can add files by scope without turning the project into a flat dumping ground.

Done when:

- Phase 0 creates a Cargo workspace with the initial crate set.
- Core shared types live in `openspace-core`.
- Feature crates expose public contracts and hide implementation details.
- Feature crates do not depend laterally on each other.
- `openspace-app` composes features and routes commands/events.
- The workspace compiles before feature implementation becomes deep.

Technical guardrails:

- Feature crates may depend on `openspace-core`.
- `openspace-app` may depend on feature crates as the composition root.
- Feature crates must not directly own global shell state.
- Feature crates must not write storage directly unless that behavior is explicitly routed through storage contracts.
- Platform-specific behavior belongs behind `openspace-platform` or feature-specific platform adapters.

## 11. Feature module and file naming rules

Product requirement: File organization must make scope obvious to developers.

Rules:

- Use feature-first modules.
- Public interface files should use the feature prefix.
- Implementation files should use the feature prefix.
- Sub-feature files should use the sub-feature prefix.
- Create new files by scope instead of overloading large generic files.
- Scope folders are allowed for discoverability.
- Scope folders are not a requirement to follow a specific named architecture.

Example pattern:

- `terminal_interface.rs`
- `terminal_types.rs`
- `terminal_events.rs`
- `terminal_commands.rs`
- `terminal_runtime.rs`
- `terminal_snapshot.rs`
- `terminal_layout_descriptor.rs`
- `terminal_split_tree.rs`
- `terminal_pty_adapter.rs`

User stories:

1. As a developer, I can infer a file's feature and scope from its name, so that navigation stays fast.
2. As a developer, I can add a sub-feature without hiding it inside a generic catch-all module.

Done when:

- Phase 0 crate skeleton follows feature prefix naming.
- New files use names that include the feature or sub-feature prefix.
- Public interfaces are separated from implementation details.
- Large generic files are avoided.

Technical guardrails:

- The naming rule is more important than copying a rigid folder taxonomy.
- Interface files expose contracts outward.
- Runtime/application/infrastructure files consume contracts inward.

## 12. AppEvent and AppCommand requirements

Product requirement: OpenSpace needs a unified coarse event/command route without flooding the app with high-frequency internal data.

User stories:

1. As a developer, I can route feature events through a common app-level mechanism, so that cross-feature behavior is understandable.
2. As a user, I can get consistent notifications, errors, pending actions, and audit history across features.

Done when:

- Coarse feature events can be promoted into AppEvent.
- AppCommand can dispatch commands to the appropriate feature runtime.
- Raw high-frequency streams do not enter AppEvent.
- App routing can update shell state, pending actions, notifications, audit, and snapshots.

Technical guardrails:

AppEvent/AppCommand should handle coarse events such as:

- Workspace changes.
- Project changes.
- Session mode changes.
- Permission changes.
- AI task lifecycle.
- Chat workflow lifecycle.
- Terminal lifecycle and layout changes.
- Editor file and diagnostics changes.
- Git status/diff/action changes.
- File system project tree/index changes.
- Storage migration and restore changes.
- Audit, notification, and error events.

Do not route these through AppEvent:

- Raw PTY bytes.
- Raw AI token stream per token.
- Cursor blink.
- Per-frame animation state.
- Raw file watcher noise.
- Raw LSP JSON-RPC packets.
- Tree-sitter incremental parser internals.

## 13. Runtime and concurrency requirements

Product requirement: UI state and long-lived feature runtimes must have clear ownership.

Runtime model:

- Iced owns shell and UI state.
- Feature actors/services own long-lived runtime state.
- A shared async runtime drives feature actors.
- The app dispatches commands to feature runtimes.
- Feature runtimes emit coarse events back to the app.
- Rendering reads snapshots/view models.

User stories:

1. As a user, I can switch modes without killing a terminal, AI task, LSP process, or watcher.
2. As a developer, I can test feature runtimes without launching the whole UI.
3. As a developer, I can reason about who owns state for each subsystem.

Done when:

- `openspace-app` owns shell UI state and focused-surface state.
- Feature runtimes own PTY, AI stream, LSP process, watcher, git, storage, and editor runtime state.
- Runtimes expose command handles and snapshot/status streams.
- Shutdown and restart paths are explicit enough for v1.
- High-frequency internals remain private.

Technical guardrails:

- Avoid putting all runtime state into the Iced AppState.
- Avoid per-feature async runtimes unless a subsystem later proves it needs isolation.
- Prefer feature handles with command channel, snapshot stream, and status stream.
- App routing must be testable without rendering the full UI.

## 14. Command palette and keybinding requirements

Product requirement: Commands and keybindings must be feature-registered and context-aware.

User stories:

1. As a user, I can open one command palette for app-wide and mode-specific commands.
2. As a user, I only see commands that make sense for the current Session, mode, focus, selection, permission profile, and project state.
3. As a developer, I can add a feature command without hardcoding everything in the app shell.

Done when:

- `openspace-core` defines command metadata contracts.
- Feature crates expose command descriptors.
- `openspace-app` merges command descriptors into a registry.
- Command palette filtering considers active mode, Session, focus, selection, project state, git state, dirty state, and permission requirements.
- A keybinding resolver exists with basic conflict detection.

Technical guardrails:

- A future modal command system should layer on top of the command registry, not replace it.
- V1 does not include a Neovim-like command language.

## 15. Terminal Mode requirements

Product requirement: Terminal Mode must be a first-class main workspace mode.

User stories:

1. As a user, I can switch a Session into Terminal Mode, so that the terminal becomes the main CenterSurface.
2. As a user, I can run shell commands inside a PTY.
3. As a user, I can create terminal tabs.
4. As a user, I can split the terminal workspace.
5. As a user, I can switch away from Terminal Mode without killing terminal processes.
6. As a user, I can restore a basic terminal layout after restart.

Done when:

- Terminal Mode renders as the CenterSurface.
- PTY input and output work on macOS.
- The terminal emulator state renders a usable grid.
- Keyboard input and focus routing work for the active terminal surface.
- At least one terminal tab can be created and closed.
- Horizontal and vertical split panes work at a basic level.
- Terminal layout descriptors can be persisted and restored.
- Terminal runtime survives mode switching.
- Raw PTY bytes do not flood AppEvent.
- Workflow-executed terminal commands are audited.

Technical guardrails:

- PTY management uses a dedicated terminal runtime actor/service.
- Terminal rendering reads snapshots or view models.
- Raw PTY data remains internal to the terminal feature.
- Terminal Mode owns terminal tabs and splits; v1 does not mix Chat or Editor as arbitrary terminal panes.
- Shell profile resolution goes through platform policy.

Later scope:

- Terminal search.
- Advanced scrollback tooling.
- Remote sessions.
- Advanced terminal theming.
- Terminal collaboration or sharing.

## 16. Chat Mode requirements

Product requirement: Chat Mode must be a workflow surface for AI-assisted work, not only a message bubble view.

User stories:

1. As a user, I can ask AI questions about the current project and Session.
2. As a user, I can stream an AI response and see progress.
3. As a user, I can inspect the ContextPack before sending.
4. As a user, I can receive action cards instead of blind execution.
5. As a user, I can approve, reject, or revise an action proposal.
6. As a user, I can see which actions were completed, failed, or cancelled.

Done when:

- Chat Mode renders as the CenterSurface.
- A chat thread can be created.
- A message can be sent to a mock provider.
- Streaming response works through the AI runtime.
- A real provider adapter can be configured after the mock flow is stable.
- Context Drawer can show included ContextPack items.
- Action cards can represent pending AI proposals.
- Approval/rejection updates action state.
- AI tasks and action decisions are audited.

Technical guardrails:

- Chat does not call provider APIs directly.
- Chat uses `openspace-ai` for AI task execution.
- Chat emits workflow state and action card state; permission routing decides execution.
- Chat history belongs to dynamic storage, not scattered local files.

Later scope:

- Autonomous task board.
- Multi-step planner/executor.
- Reusable workflows.
- Cross-session memory.

## 17. Editor Mode requirements

Product requirement: Editor Mode must provide a lightweight but real editing surface for project files.

User stories:

1. As a user, I can open a file from the project tree.
2. As a user, I can edit and save a file.
3. As a user, I can see syntax highlighting.
4. As a user, I can see basic diagnostics.
5. As a user, I can ask AI to review selected text or a file.
6. As a user, I can review and apply or reject an AI-proposed diff.

Done when:

- Editor Mode renders as the CenterSurface.
- File tabs can show opened files.
- Open/edit/save works for text files.
- Dirty state is visible.
- Rope-based buffer management is used for editor text.
- Syntax highlighting uses parser-based syntax information where available.
- Diagnostics can be displayed for the active file.
- AI review panel can create action proposals.
- Diff apply/reject flow is permission-gated and audited.

Technical guardrails:

- Editor Mode is not full IDE parity in v1.
- Buffer state belongs to the editor feature.
- File writes route through action/permission/audit paths.
- Syntax parsing internals do not become app-level events.

Later scope:

- Rich multi-cursor editing.
- Deep refactoring tools.
- Advanced editor command language.
- Full editor extension system.

## 18. LSP requirements

Product requirement: OpenSpace should provide basic language intelligence without trying to become a full IDE in v1.

User stories:

1. As a user, I can see diagnostics for the active file when a language server is available.
2. As a user, I can ask AI to explain a diagnostic.
3. As a user, I can keep language-server processes alive across mode switches.

Done when:

- LSP client runtime can start and monitor a language server process.
- Diagnostics can flow to Editor Mode and RightPanel views.
- LSP process lifecycle errors are visible.
- Basic AI diagnostic explanation can use ContextPack.
- LSP server start/fail events are audited or surfaced appropriately.

Technical guardrails:

- Raw JSON-RPC packets do not become AppEvents.
- LSP runtime owns process and protocol details.
- Editor consumes diagnostics through snapshots/view models.
- AI-assisted language behavior must still route through permission and context policies.

Later scope:

- Completion UI polish.
- Rename/refactor operations.
- Code actions.
- Workspace symbol search.
- Advanced AI-assisted language-server behavior.

## 19. AI runtime requirements

Product requirement: AI integration must be centralized through an OpenSpace AI runtime.

User stories:

1. As a user, I can configure a model/provider once and use it from Chat, Terminal, Editor, Git, and diagnostics flows.
2. As a user, I can cancel or retry AI tasks.
3. As a user, I can see AI task progress.
4. As a user, I can receive proposed actions rather than silent mutations.

Done when:

- `openspace-ai` owns provider abstraction.
- A mock provider supports deterministic development and tests.
- At least one real provider-compatible adapter can stream responses.
- AI tasks have started, streaming, completed, failed, and cancelled states.
- AI receives ContextPack input.
- AI emits stream deltas, task lifecycle events, and action proposals.
- Chat, Terminal, Editor, Git, and LSP do not call providers directly.
- Provider errors are normalized into app-visible errors.

Technical guardrails:

- HTTP transport is infrastructure detail, not the architecture.
- Provider credentials are resolved through secrets policy.
- Tool calls and action proposals are normalized before permission routing.
- No autonomous multi-agent engine in v1.

Later scope:

- Local provider adapters.
- Provider routing/fallback.
- Prompt/template registry.
- Model latency/evaluation telemetry.
- Agent task board.

## 20. ContextPack requirements

Product requirement: AI context must be hybrid: explicit anchors plus safe automatic enrichment.

User stories:

1. As a user, I can see what context will be sent to AI.
2. As a user, I can remove context items before sending.
3. As a user, I can attach explicit files, selections, diffs, diagnostics, terminal snapshots, or chat thread context.
4. As a user, I can rely on OpenSpace to add safe, relevant context without reading the whole project blindly.

Done when:

- ContextPack can include explicit anchors from Session, mode, selected text, active file, chat thread, terminal, git diff, diagnostic, or user attachment.
- Safe enrichment can include nearby lines, opened files, git status summary, active terminal metadata, relevant diagnostics, small related files, and project tree summary.
- Context Drawer shows source, scope, reason, content kind, token estimate, freshness, and permission decision.
- Ignored, binary, large, or secret-like files are not included by default.
- V1 does not require embeddings or semantic graph retrieval.

Technical guardrails:

- Context builder must respect Session permission profile.
- Context items must record why they were included.
- Full project ingestion is not the default.
- Token budget must be explicit enough to test.

Later scope:

- Semantic retrieval.
- Embeddings.
- Dependency graph.
- Context compression.
- Cross-session/project memory.

## 21. File system requirements

Product requirement: OpenSpace must understand the active project folder enough to support navigation, search, context, and file-change awareness.

User stories:

1. As a user, I can open a project folder.
2. As a user, I can browse a file tree.
3. As a user, I can search the project.
4. As a user, I can rely on OpenSpace to notice relevant file changes.

Done when:

- A project folder can be opened.
- File tree appears in the shell.
- Basic text search/indexing works for allowed files.
- File watcher updates project metadata.
- Ignored and large/binary files are handled safely.
- FS metadata can enrich ContextPack.

Technical guardrails:

- File watcher noise should be debounced and filtered.
- Raw watcher events should not flood AppEvent.
- Search/index artifacts belong in cache or storage as appropriate.
- FS feature does not execute AI actions directly.

Later scope:

- Smarter ranking.
- Dependency-aware project graph.
- Richer ignore/exclude UI.

## 22. Git requirements

Product requirement: Git integration must support enough workflow for AI-assisted review and commit creation.

User stories:

1. As a user, I can see the current branch and git status.
2. As a user, I can inspect changed files and diffs.
3. As a user, I can stage and unstage changes.
4. As a user, I can commit staged changes.
5. As a user, I can ask AI to propose a commit message from staged changes.

Done when:

- Git repo detection works for an opened project.
- Status summary appears in shell/status views.
- Changed files and diffs can be viewed.
- Stage and unstage work for basic file changes.
- Commit staged changes works through a permission-aware path.
- AI commit-message proposal uses staged diff context.
- Mutating git actions are audited.

Technical guardrails:

- Git actions are feature-owned but permission-gated.
- Git state can enrich ContextPack.
- V1 is not a full git client.

Later scope:

- Branch management.
- Merge conflict tools.
- Interactive rebase support.
- Remote operations polish.

## 23. Storage requirements

Product requirement: OpenSpace must persist dynamic state reliably without scattering storage ownership across features.

Storage model:

- SQLite for dynamic app state.
- TOML/JSON for human-editable config.
- Cache directory for derived artifacts.

User stories:

1. As a user, I can restart OpenSpace and recover my basic workspace state.
2. As a user, I can keep chat, action, audit, and Session state across app launches.
3. As a developer, I can migrate storage schema safely as the app evolves.

Done when:

- SQLite storage exists for dynamic state.
- Config files exist for human-editable non-secret settings.
- Cache directory exists for derived artifacts.
- Session descriptors can be persisted and restored.
- Chat threads/messages and action cards can be persisted.
- Audit/action records can be persisted.
- Basic migration mechanism exists.

Technical guardrails:

- Feature crates do not write storage directly.
- `openspace-app` and `openspace-storage` coordinate persistence using core contracts/events/descriptors.
- Secrets are not stored plaintext in SQLite or config.
- Full event sourcing is not v1.

Later scope:

- Export/import Session.
- Richer workflow event log.
- Embeddings/vector cache.
- Encrypted portable vault.

## 24. Secrets requirements

Product requirement: OpenSpace must protect credentials by default.

Secrets model:

- OS keychain is the default secret store.
- Environment variables are supported as a development fallback.
- Config and SQLite store secret references/metadata, not plaintext secrets.
- Temporary memory secrets are allowed for session-only flows.

User stories:

1. As a user, I can configure provider credentials without storing plaintext API keys in project files.
2. As a user, I can run terminal commands without accidentally inheriting AI provider secrets.
3. As a developer, I can test secret resolution without using the real OS keychain.

Done when:

- `openspace-secrets` can resolve secrets through keychain, environment, or memory adapter.
- Config stores provider metadata and secret references only.
- Logs and audit records redact secrets.
- Child process environment filtering prevents automatic provider-key leakage.
- Tests can use memory secret adapter.

Technical guardrails:

- Raw API keys, bearer tokens, refresh tokens, Authorization headers, and secret env values must not be written to logs, audit, SQLite, or config.
- Explicit user permission is required before injecting secrets into child processes.

Later scope:

- Encrypted portable vault.
- Import/export secret bundle.
- Per-project credential scopes.
- OAuth flows.

## 25. Audit and observability requirements

Product requirement: OpenSpace must be debuggable for developers and trustworthy for users.

Observability has two paths:

1. Developer observability through structured tracing.
2. User-visible audit/action history through storage.

User stories:

1. As a user, I can see important actions that AI or OpenSpace proposed and performed.
2. As a user, I can see permission changes and risky actions in an audit history.
3. As a developer, I can debug runtime lifecycle, provider errors, storage migrations, and feature failures.

Done when:

- Structured tracing exists for development diagnostics.
- Runtime lifecycle and feature errors are observable.
- User-visible audit records are persisted for important events.
- Audit records include timestamp, Session, source feature, action kind, risk, permission profile, approval state, summary, targets, result, and redaction policy.
- Raw sensitive content is not stored by default.

Audit-worthy events include:

- Permission profile changes.
- Full Access enable/disable.
- AI task lifecycle.
- AI action proposals.
- Approval/rejection decisions.
- Workflow-executed terminal commands.
- File create/write/delete/rename/apply diff.
- Git stage/unstage/commit.
- LSP server start/fail.
- Storage migrations.
- Provider request metadata without raw secrets.

Technical guardrails:

- Do not audit raw PTY output by default.
- Do not audit raw AI token streams by default.
- Do not audit raw secrets or raw Authorization headers.
- Full Access reduces friction but does not disable audit.

Later scope:

- Diagnostic bundle export.
- Workflow timeline view.
- Optional event replay for advanced agent workflows.

## 26. Theme and visual system requirements

Product requirement: OpenSpace should feel like a polished dark desktop tool without blocking v1 on advanced rendering effects.

User stories:

1. As a user, I can work in a comfortable dark theme.
2. As a developer, I can style components from tokens rather than hardcoded colors.
3. As a user, I can distinguish mode, permission, AI task, and audit states visually.

Done when:

- Theme tokens exist for colors, spacing, radius, typography, borders, shadows, and semantic states.
- A dark palette is applied consistently enough for v1.
- Permission, risk, action, diagnostic, and git states have semantic token mapping.
- Advanced glass/frosted effects are not required for v1.

Technical guardrails:

- Start with tokens and component styles.
- Add custom shader paths only after baseline usability and performance are stable.
- GPU effect capability should be detected through platform/theme policy.

Later scope:

- Advanced frosted/glass materials.
- More palettes.
- Visual regression testing for critical components.

## 27. Platform requirements

Product requirement: V1 is macOS-first while preserving platform adapter boundaries.

User stories:

1. As a macOS user, I can use the first official OpenSpace build.
2. As a developer, I can keep Linux and Windows behavior behind adapter boundaries without blocking v1.
3. As a developer, I can add platform behavior without scattering OS conditionals across feature code.

Done when:

- macOS is the official v1 target.
- Platform policies exist for shell profile, PTY backend, secrets backend, path behavior, shortcuts, file watcher behavior, packaging target, process environment, and GPU effect capability.
- Linux and Windows adapters may be placeholder or compile-gated if needed.
- Linux and Windows do not block v1 release.

Technical guardrails:

- `openspace-platform` owns OS capability detection and policy selection.
- Feature crates should ask platform policy instead of hardcoding OS behavior.
- macOS polish can be prioritized without making core architecture macOS-only.

Later scope:

- Linux official support.
- Windows official support.
- Platform-specific packaging/signing/updater strategy.

## 28. Testing requirements

Product requirement: V1 must have enough tests to protect architecture, permission safety, runtime behavior, and storage migrations without requiring full UI automation.

User stories:

1. As a developer, I can test core policies without launching the app.
2. As a developer, I can test feature runtime behavior with fakes and temp resources.
3. As a developer, I can verify app routing for commands, events, permissions, and audit.
4. As a user, I can rely on a manual smoke checklist for macOS private alpha quality.

Done when:

- Unit/domain tests cover permission decisions, command filtering, context budgeting, audit redaction, path policy, theme token resolution, action risk classification, Session mode/permissions, and storage value objects.
- Feature integration tests use mock PTY, mock provider/SSE stream, fake language server, temp SQLite DB, temp directories, temp git repo, memory secrets, and fake context sources.
- App routing tests cover AppCommand dispatch, AppEvent state updates, permission approval flow, audit emission, mode switching runtime survival, pending action approval/rejection, command palette filtering, and Session restore descriptors.
- Manual macOS smoke checklist exists.
- Real AI network, real OS keychain, full UI automation, and GPU snapshot tests are not required for v1 tests.

Technical guardrails:

- Tests should verify external behavior and boundary contracts, not private implementation details.
- Mock/fake adapters must be first-class enough to keep tests deterministic.

Later scope:

- Migration regression suite.
- Packaging smoke tests.
- Performance tests for terminal/editor.
- Watcher/LSP stress tests.
- Selective E2E UI automation.

## 29. Implementation milestones

### Phase 0: Workspace skeleton

Goal: create a compiling workspace boundary without building every foundation in full.

Includes:

- Cargo workspace.
- Initial crates.
- Minimal `openspace-core` contracts.
- Minimal app entry point.
- Minimal feature module skeletons.
- Basic test harness.

Done when:

- Workspace compiles.
- Crate dependency direction is valid.
- Naming conventions are visible in skeleton files.
- No feature is deeply implemented yet.

### Phase 1: Shell + Session + Mode skeleton

Goal: prove the main app model.

Includes:

- Stable shell regions.
- Active Session.
- Active mode switch.
- Minimal AppEvent/AppCommand.
- Minimal command registry.
- Theme tokens.
- Runtime manager skeleton.
- Minimal storage/audit placeholders.

Done when:

- App launches.
- Mode switch changes CenterSurface.
- Shell remains stable.
- No cross-mode mixed layout exists.

### Phase 2: Terminal risk slice

Goal: validate the most technically risky v1 surface early.

Includes:

- Terminal runtime actor.
- One PTY.
- Terminal emulator state.
- Rendered terminal grid.
- Input/focus handling.
- Terminal tab.
- Split pane.
- Layout descriptor.

Done when:

- Terminal Mode is a real main surface.
- Commands can run through PTY.
- Tabs and splits work basically.
- Runtime survives mode switch.

### Phase 3: AI + Chat workflow slice

Goal: prove the AI workflow surface and safety loop.

Includes:

- Mock provider.
- Streaming response.
- ChatWorkflowView.
- Context Drawer minimal.
- Action cards.
- Pending approvals.
- Permission routing.
- Audit for AI tasks/actions.
- First real provider-compatible streaming adapter after mock stabilizes.

Done when:

- Chat Mode can stream AI responses.
- Context is inspectable.
- AI action proposals can be approved/rejected.
- No direct mutation happens from AI output.

### Phase 4: FS + Git context slice

Goal: make OpenSpace project-aware.

Includes:

- Open project/folder.
- File tree.
- File watcher/index/search minimal.
- Git status.
- Git diff viewer.
- Stage/unstage.
- Commit staged changes.
- AI commit message proposal.
- ContextPack enrichment from FS/Git.

Done when:

- Project state appears in shell.
- Git context can feed AI safely.
- Mutating git actions are permission-gated and audited.

### Phase 5: Lightweight Editor + LSP slice

Goal: make Editor Mode useful without chasing full IDE parity.

Includes:

- Open/edit/save file.
- File tabs.
- Rope buffer.
- Parser-based syntax highlighting.
- Dirty state.
- AI review/action panel.
- Basic LSP diagnostics.
- Minimal AI-assisted LSP flow.
- Diff apply/reject.

Done when:

- Editor Mode supports real text editing.
- Diagnostics appear.
- AI review can propose diffs.
- Diff application is permission-gated and audited.

### Phase 6: Hardening v1

Goal: make macOS Private Alpha usable.

Includes:

- SQLite migrations.
- Keychain secrets.
- macOS platform polish.
- Packaging path.
- Test coverage.
- Manual smoke checklist.
- Terminal/editor performance pass.
- Audit/redaction pass.
- Permission UX pass.

Done when:

- macOS v1 build is usable for lightweight real work.
- V1 cutline is met.
- Known non-goals remain out of scope.

## 30. Future scope

Future work after M1 Private Alpha may include:

- Public beta onboarding and distribution polish.
- Official Linux support.
- Official Windows support.
- Plugin system.
- Neovim-like command mode.
- Autonomous task board.
- Multi-step planner/executor.
- Semantic retrieval and embeddings.
- Project dependency graph.
- Advanced editor features.
- Advanced git workflows.
- Advanced terminal features.
- Encrypted portable vault.
- Workflow replay/timeline.
- Production updater.
- Advanced rendering effects.

## 31. Appendix: decision summary

1. OpenSpace is a desktop AI assistant with Terminal, Chat, and Editor main modes.
2. No cross-mode mixed main layout in v1.
3. A Session owns active mode and permission profile.
4. Stable shell surrounds a mode-switching CenterSurface.
5. Use feature-first crates and feature-prefixed files.
6. Feature crates expose interfaces and hide implementation details.
7. Feature crates depend on core contracts and avoid lateral dependencies.
8. Use coarse AppEvent/AppCommand routing.
9. Keep high-frequency streams private to features.
10. Iced owns UI state; feature actors own long-lived runtime state.
11. Terminal Mode is a first-class main surface with tabs and splits in v1.
12. Chat Mode is a workflow surface with ContextPack, action cards, approvals, and audit.
13. Editor Mode is lightweight but real: open/edit/save, highlighting, diagnostics, AI review.
14. LSP is basic for v1 and does not require full IDE parity.
15. AI integration is centralized in `openspace-ai`.
16. AI receives ContextPack and emits task lifecycle events and action proposals.
17. ContextPack uses explicit anchors plus safe automatic enrichment.
18. SQLite stores dynamic state; human-editable config stores non-secret settings; cache stores derived artifacts.
19. OS keychain is the default secret store; environment variables are development fallback.
20. Child processes do not automatically inherit provider secrets.
21. Structured tracing supports developer observability.
22. User-visible audit/action history supports trust and safety.
23. Feature-registered command registry powers command palette and keybindings.
24. macOS is official v1; Linux and Windows adapter boundaries are non-blocking.
25. Testing uses unit/domain tests, feature integration tests, app routing tests, and manual macOS smoke checklist.
26. Implementation starts with a thin workspace skeleton, then risk-first vertical slices.
27. V1 means M1 Private Alpha; public beta comes later.
28. README remains short; this DESIGN.md is the primary source of truth.
