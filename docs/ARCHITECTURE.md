# OpenSpace Architecture

## Overview

OpenSpace is structured around a Rust core engine that orchestrates desktop integration, AI model communication, and task automation. The architecture prioritizes performance, safety, and modularity.

## Layers

### 1. Desktop Connector

Responsible for integrating with the host operating system:
- Window and process monitoring
- File system event watching
- Clipboard and input context
- Screen content analysis (where permitted)

### 2. AI Bridge

Handles communication with AI models:
- Unified protocol for local and remote models
- Context window management
- Streaming response handling
- Token usage tracking

### 3. Task Engine

Executes user-requested operations:
- Workflow parsing and validation
- Step-by-step execution with rollback capability
- Integration with desktop APIs for automation
- Result capture and formatting

### 4. Subagent Orchestrator

Manages specialized subagents for complex tasks:
- Spawns task-specific subagents
- Aggregates results from multiple agents
- Handles subagent failures and retries
- Maintains overall task context

### 5. Memory Store

Persistent storage layer:
- Conversation history
- User preferences and learned patterns
- Workflow definitions
- Secure credential storage

## Data Flow

```
User Request
    |
    v
Desktop Connector --(context)--> AI Bridge
    |                                |
    v                                v
Task Engine <--(plan)--> Subagent Orchestrator
    |                                |
    v                                v
Desktop APIs <--(execute)--> AI Models
    |                                |
    +----------(results)-------------+
                   |
                   v
              Memory Store
```

## Module Structure

```
src/
├── main.rs              # Application entry point
├── desktop/             # Desktop integration layer
│   ├── mod.rs
│   ├── monitor.rs
│   └── automation.rs
├── ai/                  # AI model bridge
│   ├── mod.rs
│   ├── client.rs
│   └── context.rs
├── engine/              # Core task engine
│   ├── mod.rs
│   ├── executor.rs
│   └── workflow.rs
├── subagent/            # Subagent orchestration
│   ├── mod.rs
│   ├── orchestrator.rs
│   └── agent.rs
├── memory/              # Storage layer
│   ├── mod.rs
│   ├── store.rs
│   └── models.rs
└── config/              # Configuration management
    ├── mod.rs
    └── settings.rs
```

## Key Design Decisions

- **Rust for Core**: Memory safety and zero-cost abstractions make Rust ideal for a long-running desktop daemon
- **Async Runtime**: Tokio-based async for handling concurrent AI streams and desktop events
- **Plugin Model**: Future support for WASM-based plugins to extend capabilities without core changes
- **Event Sourcing**: Internal state changes are logged as events for debugging and replay
