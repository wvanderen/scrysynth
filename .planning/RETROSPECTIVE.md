# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — v1 Runtime Hardening

**Shipped:** 2026-06-26
**Timeline:** 2026-04-10 → 2026-06-26 (77 days)
**Phases:** 11 (1–5 foundation + 6–11 hardening) | **Plans:** 21 (foundation) + 6 (hardening) | **Sessions:** multiple `td` sessions across phases

### What Was Built

- **Canonical core + workspace:** Rust-owned `SessionDocument`, generated TS contracts, JSON persistence, React Flow graph workspace, conversation/graph/performance views, scene recall, variations, cross-domain macros.
- **Real audio execution:** SuperCollider topology compiler + OSC adapter verified for audible playback, live parameter updates, stop, panic, and restart-after-panic.
- **Minimal visual sidecar:** `scrysynth-visual` process with JSON-lines protocol, scene load, parameter batches, full stop/panic/restart lifecycle; bundled via Tauri externalBin.
- **Hardware runtime:** CoreMIDI + OSC learn and post-learn routing for macros, scenes, transport, and panic.
- **Session-aware agent:** bounded context packets, provider-agnostic planner, typed-command normalization, ownership/risk/approval gates, freeze/reclaim, diagnostics — wired into the production GUI.
- **Release packaging:** ad-hoc-signed `scrysynth.app` + `.dmg` for `aarch64-apple-darwin`, README + RELEASE_NOTES with Supported/Not-supported matrices.

### What Worked

- **Consolidated release UAT (Phase 11):** re-verifying every runtime path (audio, visual, hardware, agent, panic) against the *packaged* app in one nine-scenario pass caught a real planner-wiring defect (`415e8d8`) that earlier per-phase UATs had missed. This was the single highest-value verification step of the milestone.
- **Typed-command safety boundary:** keeping every agent mutation behind typed commands, ownership gates, risk tiers, approvals, freeze/reclaim, and action history meant the Phase 10 planner could be added *above* the boundary without re-litigating safety. The boundary held.
- **App-owned canonical truth:** deferring runtime engines to adapter roles (SuperCollider for execution, sidecar for visuals) kept the Rust core stable through UI churn and made the UAT scenarios composable.
- **Honest stub/scaffold labeling:** marking Phases 1–5 "foundation complete, not release complete" up front prevented overclaiming and made the hardening scope unambiguous.

### What Was Inefficient

- **Per-phase plan/summary naming drift:** Phases 7–9 used ad-hoc plan names (`SC-RESOURCE-PLAN.md`, `VISUAL-SIDECAR-PROTOCOL.md`, consolidated single plans) instead of the standard `NN-PLAN.md`/`NN-SUMMARY.md` pairs. This made `roadmap.analyze` undercount completion (50% vs 100%) at milestone close and required `--force` plus manual reconciliation. Adopting the standard naming earlier would have avoided this.
- **Stale UAT metadata:** several phase UAT files carried `[unknown]`/`[diagnosed]` status labels after they were superseded by the Phase 11 consolidated UAT, which surfaced as false-positive "open items" in the pre-close audit.
- **Phase 6 absorbed silently:** "Local Developer Readiness" had no dedicated phase directory because its work was folded into Phase 11 — but ROADMAP.md still listed it as a standalone phase, blocking the milestone-close CLI until `--force` was used.
- **Environment-dependent verification:** Rust tests needing OSC UDP bind permissions and the GUI click-through UAT both required escapes out of the sandbox/CI environment. Local-only verification paths leave a gap for future cross-platform/cross-machine work.

### Patterns Established

- **Safety boundary above the planner:** agent intelligence lives above a typed-command/ownership/approval layer, never beside it. Carry this into the live-provider milestone.
- **Packaged-app UAT as the release gate:** a consolidated UAT against the built `.app` (not `tauri dev`) is the moment of truth for release readiness. Repeat for every future release.
- **ExternalBin sidecar bundling:** `tauri-plugin-shell` externalBin is the working pattern for shipping a second Rust binary inside the Tauri app.
- **Honest Supported/Not-supported matrices in release notes:** ship a two-column matrix in README + RELEASE_NOTES so claims are auditable against UAT scenarios.

### Key Lessons

1. **Use standard GSD plan/summary naming from day one of each phase** — ad-hoc names break `roadmap.analyze`, milestone-close checks, and accomplishment extraction. The retrofitted structure cost real reconciliation time at the worst moment (milestone close).
2. **When phase scope is absorbed by another phase, update the roadmap immediately** — leaving Phase 6 as a phantom standalone phase blocked the close CLI.
3. **Re-verify every runtime path against the packaged app before claiming release-ready** — `tauri dev` and packaged builds diverge (planner-wiring bug `415e8d8` was only reachable in the production GUI path).
4. **Keep UAT status metadata in sync when superseding a UAT** — either mark the old file superseded or delete it; stale `[unknown]` labels create audit noise.
5. **Defer provider/live-LLM work behind a verified provider-agnostic boundary** — this protected the v1 schedule and left a clean extension point.

### Cost Observations

- Model mix: not instrumented per-model this milestone (carry-over action: enable per-model token tracking before the next milestone).
- Sessions: multiple `td` work sessions per phase (e.g., Phase 9 `td-dcaf9a` epic with 6 child tasks; Phase 10 `td-cebec0` epic with 6 child tasks; Phase 11 split across two plans).
- Notable: the highest-leverage session was the Phase 11 Plan 02 consolidated UAT + planner-wiring fix — a single session closed the release gate and unblocked REL-02.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | many (per-phase `td` epics) | 11 | Adopted `td` for task tracking mid-milestone; retrofitted GSD phase structure onto Phases 1–5 |

### Cumulative Quality

| Milestone | Tests | Coverage | Zero-Dep Additions |
|-----------|-------|----------|-------------------|
| v1.0 | `npm test` (frontend) + `cargo test` (Rust) + packaged-app UAT (9 scenarios) | not measured | n/a (greenfield stack) |

### Top Lessons (Verified Across Milestones)

1. *(Single milestone so far — to be cross-validated after v1.x.)* Standard plan/summary naming and packaged-app UAT are the two process choices that most reduced close-time friction.
