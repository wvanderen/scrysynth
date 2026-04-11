# Technology Stack

**Project:** Scrysynth
**Researched:** 2026-04-11
**Scope:** Greenfield, local-first Tauri desktop instrument with an app-owned canonical session graph, SuperCollider v1 audio adapter, and a separate visual runtime.
**Overall recommendation confidence:** MEDIUM-HIGH

## Recommended Stack

## Platform Foundation

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Desktop shell | Tauri | 2.10.x (`tauri` crate 2.10.3, `@tauri-apps/cli` 2.10.1, `@tauri-apps/api` 2.10.1) | Native desktop packaging, windowing, permissions, IPC | This is the correct shell for a local-first desktop instrument: small binaries, Rust backend, strong capability model, and first-class sidecar support for external runtimes. | HIGH |
| Backend language | Rust | Stable toolchain, target current stable; honor Tauri plugin floor `>=1.77.2` | Own canonical graph, runtime orchestration, persistence, process supervision | The canonical session graph should live in a strongly typed, non-GC core that can supervise audio/visual runtimes and survive UI churn. Rust is the right center of gravity here. | HIGH |
| Frontend | React | 19.2.5 | Desktop UI for graph, conversation, and performance views | React remains the most practical choice for a complex node editor plus inspector/control UI inside Tauri. Ecosystem support for graph tooling is strongest here. | MEDIUM-HIGH |
| App build tool | Vite | 8.0.8 | Fast frontend dev/build pipeline | Tauri officially supports Vite setups, and Vite keeps iteration fast without adding framework-specific baggage. | HIGH |
| App language | TypeScript | 6.0.2 | Typed UI and adapter contracts | Necessary for large graph/editor UI surfaces and safe Rust-TS boundary work. | HIGH |

## Canonical Session Graph And Contracts

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Domain serialization | `serde` | 1.0.228 | Serialize session graph, commands, runtime feedback | Standard Rust serialization layer; dependable and ubiquitous. | HIGH |
| Async/runtime orchestration | `tokio` | 1.51.1 | Supervise adapters, IO, process events, timers | You will need async process management, OSC/IPC bridges, and runtime supervision. Tokio is the standard Rust choice. | HIGH |
| Type export Rust -> TS | `ts-rs` | 12.0.1 | Generate TypeScript types from Rust domain types | Prefer a stable type-export crate over `specta`/`tauri-specta` RCs for the foundation phase. The Rust model should define the schema; TS should consume generated types. | MEDIUM |
| Runtime validation in UI | `zod` | 4.3.6 | Validate imported session files, agent payloads, untrusted runtime feedback | Even with generated TS types, runtime validation is still needed at file and adapter boundaries. | HIGH |
| Frontend state mirror | `zustand` + `immer` | 5.0.12 + 11.1.4 | UI projection of canonical graph and transient interaction state | Zustand is lighter than Redux for editor-style apps, and Immer makes graph mutations ergonomic. Use it as a mirror/projection layer, not the ultimate source of truth. | MEDIUM-HIGH |

## Graph UI And Interaction Layer

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Node editor | `@xyflow/react` | 12.10.2 | Visible session graph editor/viewer | This is the strongest maintained React node-editor library in 2026 and explicitly supports node-based editors, validation, undo/redo examples, layout integrations, and performance guidance. | HIGH |
| Auto-layout | `elkjs` | 0.11.1 | Structured graph layout for scene/group/subgraph views | ELK handles richer graph layout cases than `dagre`, which matters once routes, buses, macros, and grouped structures appear. | MEDIUM-HIGH |
| UI primitives | Radix UI primitives | `@radix-ui/react-dialog` 1.1.15, `@radix-ui/react-slider` 1.3.6 | Accessible primitives for inspectors, overlays, transport/control widgets | Scrysynth needs instrument-specific UI, not a canned SaaS component kit. Radix gives accessible primitives without forcing generic design language. | MEDIUM |
| Styling system | `@vanilla-extract/css` | 1.20.1 | Typed design tokens, themes, CSS extraction | Better fit than utility-first CSS for a graph/control-surface app with long-lived design tokens, mode states, and visual identity. | MEDIUM |

## Audio Runtime Adapter

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Audio engine | SuperCollider | 3.14.1 | Real-time synthesis, routing, modulation, effects | Official docs still position `scsynth` as the real-time audio engine and the client/server model fits an app-owned canonical graph with an adapter boundary. This matches the project constraints directly. | HIGH |
| SC process strategy | Tauri sidecar support + Rust process supervision | Tauri sidecar docs current Jan 2026; `@tauri-apps/plugin-shell` 2.3.5 when needed | Launch and monitor bundled SC binaries or approved local installs | Tauri has first-class sidecar support. Use Rust-side supervision so the app can restart, health-check, and fence runtime failures without trusting the frontend. | HIGH |
| Audio control protocol | OSC via `rosc` | 0.11.4 | Send graph-derived control messages to SuperCollider | OSC is the natural control surface for SuperCollider. Keep the adapter translation in Rust and treat OSC as runtime transport, not domain truth. | MEDIUM-HIGH |

## Visual Runtime Adapter

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Visual engine | Bevy | 0.18.1 | Separate GPU-native visual runtime process | For a greenfield separate renderer, Bevy is a better 2026 bet than smaller creative-coding stacks because it is active, `wgpu`-backed, cross-platform, and workable as a dedicated runtime rather than UI garnish. | MEDIUM |
| GPU abstraction | `wgpu` | 29.0.1 | Low-level rendering foundation underneath the visual runtime | Use this through Bevy in v1, not directly in the product app. It preserves a path to lower-level optimization later. | MEDIUM |
| Visual adapter transport | Rust local IPC/WebSocket layer over `tokio` + `serde` | Use app-local typed messages, no browser-first transport | Synchronize scene/control state and receive runtime telemetry | The visual runtime needs richer typed messages and acknowledgements than pure OSC usually gives. Keep a typed adapter protocol separate from the canonical graph model. | MEDIUM |

## Persistence And Session Storage

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Local database | SQLite via `rusqlite` | 0.39.0 | Persist sessions, indexes, history, settings, crash recovery metadata | SQLite is the right local database for a single-user desktop app. Use it behind the Rust domain layer, not as a frontend-owned store. | HIGH |
| Session interchange | Versioned JSON files validated by `zod`/`serde` | N/A | Import/export portable session documents | Human-inspectable sessions matter for debugging, migration, and agent explainability. Persist live state in SQLite, export/import as explicit versioned files. | MEDIUM-HIGH |
| Telemetry/logging | `tracing` | 0.1.44 | Structured logs across UI bridge, adapter layer, and runtime supervision | You will need per-session and per-runtime diagnostics early; plain string logs are not enough once audio/visual adapters diverge. | HIGH |

## Testing And Quality Gates

| Layer | Technology | Version | Purpose | Why this choice | Confidence |
|------|------------|---------|---------|-----------------|------------|
| Frontend unit/integration | Vitest | 4.1.4 | Fast UI/domain projection tests | Natural fit with Vite and enough for editor logic, selectors, serializers, and UI state. | HIGH |
| Desktop e2e | Playwright | 1.59.1 | Smoke-test Tauri flows and session lifecycle | Tauri docs support WebDriver-based testing; Playwright is still the most practical cross-platform UI automation choice around that. | MEDIUM |
| Rust snapshot tests | `insta` | 1.47.2 | Snapshot session graphs, command outputs, migrations | Very useful for canonical graph shape, file migration outputs, and agent action plans. | MEDIUM-HIGH |
| Rust property tests | `proptest` | 1.11.0 | Invariant testing for graph operations | Graph rewrites and routing invariants are exactly where property tests pay off. | MEDIUM-HIGH |

## Prescriptive Build Order

Build the product in this order:

1. Rust domain core: canonical session graph, commands, events, serialization.
2. Tauri shell: typed command/event bridge, sidecar/process supervision, window model.
3. React UI shell: conversation, graph viewport, inspector/performance panels.
4. SuperCollider adapter: process launch, OSC bridge, health checks, stable primitive set.
5. SQLite persistence: sessions, snapshots, migration story, recovery.
6. Separate visual runtime adapter: typed scene/control sync, not engine-coupled to audio.

This ordering keeps the app-owned graph real from day one and prevents either runtime from becoming the accidental source of truth.

## What To Avoid

| Avoid | Why not | Use instead |
|------|---------|-------------|
| Electron | Bigger runtime footprint and weaker security story for a Rust-centered desktop instrument. | Tauri 2.x |
| Tone.js or Web Audio as the primary v1 audio engine | Good for browser experiments, wrong center of gravity for a desktop instrument whose audio runtime is explicitly SuperCollider. | SuperCollider adapter via OSC/process supervision |
| A graph database (Neo4j, embedded graph DB) for session truth | The product graph is a domain model, not an analytics query problem. A graph DB adds operational weight without helping live control. | In-memory typed graph + SQLite persistence |
| `@tauri-apps/plugin-sql` as the primary write path from the frontend | Letting the UI mutate canonical state in SQL bypasses domain invariants and makes migrations harder. | Rust-owned persistence layer with explicit commands |
| `Yjs`/CRDT sync in v1 | The project is explicitly single-user local-first. CRDT complexity would be pure roadmap drag right now. | Simple local history/undo and deterministic command log |
| `three` / `@react-three/fiber` in the main app webview as the canonical visual runtime | Convenient for prototypes, but it couples visuals to UI frame timing and weakens the separate-runtime boundary the project wants. | Separate Bevy sidecar/runtime |
| `nannou` as the primary visual runtime for v1 | Still interesting for sketches, but the ecosystem appears smaller and less current than Bevy for a productized greenfield runtime in 2026. | Bevy 0.18.x |
| `specta` / `tauri-specta` for the foundation layer | Current crates are still `2.0.0-rc.24`, which is fine for experiments but not my first recommendation for the canonical core contract layer. | `ts-rs` + `zod` |

## Confidence Notes By Recommendation

- **HIGH:** Tauri 2, Rust core ownership, SuperCollider 3.14.1, SQLite, `serde`, `tokio`, `@xyflow/react`, structured logging.
- **MEDIUM-HIGH:** React 19 + Vite + TypeScript, Zustand mirror store, OSC adapter via `rosc`, versioned JSON import/export.
- **MEDIUM:** Bevy as the separate visual runtime, Radix + vanilla-extract UI stack, `ts-rs` over RC type-bridge tooling.
- **LOW:** None. I avoided recommendations that I could not support with current official docs or reliable package registry data.

## Recommended Initial Package Set

```bash
# frontend
pnpm add react react-dom zustand immer zod @xyflow/react elkjs @radix-ui/react-dialog @radix-ui/react-slider @vanilla-extract/css

# frontend dev
pnpm add -D typescript vite vitest playwright

# rust core
cargo add tauri tokio serde serde_json ts-rs rosc rusqlite tracing

# visual runtime (separate crate/process)
cargo add bevy
```

## Sources

- Tauri docs: https://v2.tauri.app/start/
- Tauri sidecar docs: https://v2.tauri.app/develop/sidecar/
- Tauri shell plugin docs: https://v2.tauri.app/plugin/shell/
- Tauri SQL plugin docs: https://v2.tauri.app/plugin/sql/
- SuperCollider official site/docs: https://supercollider.github.io/ and https://docs.supercollider.online
- React Flow official docs: https://reactflow.dev/
- Bevy getting started docs: https://bevyengine.org/learn/quick-start/getting-started/
- Nannou official site: https://nannou.cc/
- Yjs official site: https://yjs.dev/
- Current package/crate versions checked via npm registry and crates.io on 2026-04-11
