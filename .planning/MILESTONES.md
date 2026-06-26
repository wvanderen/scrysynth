# Milestones

## v1.0 — v1 Runtime Hardening (Shipped: 2026-06-26)

**Status:** ✅ SHIPPED
**Timeline:** 2026-04-10 → 2026-06-26 (77 days)
**Phases:** 11 (1–5 foundation + 6–11 hardening; Phase 6 dev-readiness folded into Phase 11)
**Git range:** `b503760` (first commit) → `3d9834c` (REL-03 doc reconciliation)
**Scope:** 180 files changed, +53,718 LOC, 0 deletions of substance

### What Shipped

Scrysynth v1 (1.0.0) is a packaged, ad-hoc-signed macOS Apple Silicon desktop instrument: `scrysynth.app` + `scrysynth_1.0.0_aarch64.dmg`. The canonical app-owned session model, workspace UI, command layer, persistence, runtime adapters, and safety scaffolding are implemented and were re-verified end-to-end against the packaged app in the consolidated Phase 11 UAT (9/9 scenario areas passed).

### Key Accomplishments

- **Canonical session core & workspace (Phases 1–3, 5):** Rust-owned `SessionDocument` with generated TypeScript contracts, versioned JSON persistence, React Flow graph workspace with inspector/palette/route gestures, conversation/graph/performance view switching, scene recall, variation save/restore, and cross-domain macros targeting audio + visual parameters.
- **Agent safety scaffold (Phase 4):** Deterministic intent parser, node ownership metadata + badges, high-risk pending actions with approve/reject, freeze/reclaim controls, and structured action history/diffs — all through typed commands.
- **Real SuperCollider execution (Phase 7):** Topology compiles to SC synthdefs/groups/buses/node order; OSC bundles drive boot, resource creation, routing, live parameter updates, teardown; audible playback + panic + restart-after-panic verified against local `scsynth`. A Stop-button projection defect found in UAT was fixed and retested.
- **Minimal visual sidecar path (Phase 8):** In-repo `scrysynth-visual` launches as a separate process, JSON-lines handshake, loads compiled scene snapshots, applies live parameter batches, and cycles stop/panic/restart. Runtime Health panel surfaces missing-sidecar through restartable states.
- **Hardware input runtime wiring (Phase 9):** CoreMIDI virtual source + local OSC sender verified live MIDI/OSC learn and post-learn routing for macros, scene recall, transport play/stop, and panic through the app-owned runtime path.
- **Session-aware agent orchestration (Phase 10):** Bounded context packets, provider-agnostic planner interface, deterministic/mock proposal fixtures, typed-command normalization, ownership/risk/approval gates, freeze/reclaim, and provider-unavailable/invalid-response diagnostics.
- **Release readiness (Phase 11):** Tauri bundle config (ad-hoc, Apple Silicon, narrowed targets), visual sidecar bundled via `tauri-plugin-shell` externalBin, end-user-actionable missing-runtime diagnostics, planner wired into the production GUI (`415e8d8`), nine-scenario packaged-app UAT passed, README + RELEASE_NOTES rewritten with Supported/Not-supported matrices.

### Requirements Outcome

36/37 v1 requirements verified complete. `AGNT-01R` (live provider-backed planner) is the sole unchecked requirement — deterministic/mock planner orchestration is verified and wired into the production GUI; a live LLM provider remains future hardening.

### Known Deferred Items at Close

- 6 historical UAT files with non-`passed` metadata statuses (all 0 pending scenarios; superseded by the Phase 11 consolidated UAT) — see STATE.md `## Deferred Items`.
- Beyond-v1 scope: richer Bevy-rendered visuals + visible render window, live provider-backed agent (AGNT-01R), physical-controller GUI click-through UAT, Windows/Linux/Intel/universal builds, full Developer ID notarization + auto-update, multiplayer.

### Archives

- Roadmap: `.planning/milestones/v1.0-ROADMAP.md`
- Requirements: `.planning/milestones/v1.0-REQUIREMENTS.md`

---
