# OpenSpace

[![Rust](https://img.shields.io/badge/core-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Desktop AI Assistance powered by Rust.** OpenSpace bridges your desktop environment with AI models to help you accomplish tasks faster, automate workflows, and stay productive.

---

## Overview

OpenSpace is a desktop AI assistant built with **Rust at its core**. It connects your daily desktop activities with powerful AI models to understand context, automate repetitive tasks, and provide intelligent assistance right where you work.

Whether you need help writing code, managing files, summarizing documents, or orchestrating complex workflows, OpenSpace brings AI capabilities directly to your desktop environment with performance and safety guaranteed by Rust.

---

## Features

- **Rust Core Engine** — High-performance, memory-safe core built entirely in Rust
- **Desktop Integration** — Deep integration with your operating system for contextual assistance
- **AI Model Agnostic** — Works with local and remote AI models (BYOK and proxy support)
- **Workflow Automation** — Automate repetitive desktop tasks with AI-driven playbooks
- **Privacy-First** — Keep sensitive data on-device with local model support
- **Cross-Platform** — Built for macOS, Windows, and Linux desktops
- **Subagent Orchestration** — Delegate complex tasks to specialized AI subagents
- **Native Performance** — Rust-powered backend ensures minimal resource footprint

---

## Architecture

```
┌─────────────────────────────────────────┐
│           Desktop Client UI              │
├─────────────────────────────────────────┤
│         Rust Core Engine                 │
│  ┌──────────┐ ┌──────────┐ ┌────────┐  │
│  │ Desktop  │ │  AI      │ │ Task   │  │
│  │ Connector│ │  Bridge  │ │ Engine │  │
│  └──────────┘ └──────────┘ └────────┘  │
│  ┌──────────┐ ┌──────────┐ ┌────────┐  │
│  │ Subagent │ │  Memory  │ │ Config │  │
│  │ Orchestr.│ │  Store   │ │ Manager│  │
│  └──────────┘ └──────────┘ └────────┘  │
├─────────────────────────────────────────┤
│     Local AI Models │ Remote APIs        │
└─────────────────────────────────────────┘
```

The Rust core engine handles:
- Desktop event monitoring and interaction
- AI model communication and context management
- Task execution and workflow automation
- Subagent spawning and result aggregation
- Secure local data storage

---

## Installation

### Prerequisites

- Rust toolchain (latest stable)
- Operating System: macOS 12+, Windows 10+, or Linux (Ubuntu 20.04+)

### From Source

```bash
git clone https://github.com/bengidev/openspace_in_rust.git
cd openspace_in_rust
cargo build --release
```

### Prebuilt Binaries

Download the latest release for your platform from the [Releases](https://github.com/bengidev/openspace_in_rust/releases) page.

---

## Usage

```bash
# Start OpenSpace
openspace

# Start with a specific AI model endpoint
openspace --model-endpoint http://localhost:11434

# Start in background daemon mode
openspace --daemon
```

---

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint
cargo clippy
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design and module breakdown
- [Contributing](CONTRIBUTING.md) — Setup, coding standards, and PR process
- [Security](SECURITY.md) — Reporting vulnerabilities and security practices
- [Code of Conduct](CODE_OF_CONDUCT.md) — Community guidelines

---

## Roadmap

- [ ] Core desktop integration layer
- [ ] Multi-AI-model connector framework
- [ ] Workflow automation engine
- [ ] Subagent orchestration system
- [ ] Cross-platform UI shell
- [ ] Plugin/extension system

---

## License

OpenSpace is licensed under the [MIT License](LICENSE).

---

*Built with Rust for performance, safety, and reliability.*
