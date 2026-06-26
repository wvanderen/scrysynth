---
phase: 11-release-readiness
plan: 02
subsystem: infra
tags: [tauri, release, packaging, supercollider, bevy, uat, docs]

# Dependency graph
requires:
  - phase: 11-release-readiness-01
    provides: Tauri v1 bundle config (1.0.0, ad-hoc macOS signing, Apple Silicon only, app/dmg targets, externalBin visual sidecar via tauri-plugin-shell, BevySidecarAdapter app.shell().sidecar() refactor, missing-binary message polish)
provides:
  - Verified ad-hoc-signed packaged scrysynth.app + .dmg for aarch64-apple-darwin (REL-01)
  - Consolidated nine-scenario packaged-app UAT evidence (REL-02)
  - Release-accurate README.md + new RELEASE_NOTES.md with Supported/Not-supported matrices (REL-03)
  - Reconciled ROADMAP.md / STATE.md / REQUIREMENTS.md marking REL-01/02/03 complete
  - Planner wired into the production GUI (fix 415e8d8) making REL-02 scenario 7 reachable
affects: [v1-milestone-close, future-release-hardening, live-provider-agent]

# Tech tracking
tech-stack:
  added: []  # no new deps; plan was build + UAT + docs
  patterns: [ad-hoc-signed-tauri-release, quarantine-vs-gatekeeper-honest-recording]

key-files:
  created:
    - .planning/phases/11-release-readiness/11-release-readiness-03-UAT.md
    - RELEASE_NOTES.md
  modified:
    - README.md
    - .planning/REQUIREMENTS.md
    - .planning/ROADMAP.md
    - .planning/STATE.md
    - .planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md
    - src-tauri/src/lib.rs (via fix 415e8d8, pre-Plan-02-execution)
    - src-tauri/src/application/agent_command.rs (via fix 415e8d8)
    - src-tauri/tests/agent_commands.rs (via fix 415e8d8)

key-decisions:
  - "D-03 recorded honestly: no Gatekeeper 'Open Anyway' prompt on the build host because the locally-built .app carries no com.apple.quarantine (xattr shows only com.apple.provenance); the prompt is the end-user download path, documented in README/RELEASE_NOTES."
  - "REL-02 scenario 7 trigger is 'remove agent layer', matched by the additive parse_remove_command fallback in 415e8d8 (RemoveNode is RiskTier::High), making a high-risk pending action reachable from the production GUI for the first time."
  - "AUD-03R marked complete on the basis of the default source->bus->output graph topology verified in the packaged app (Phase 7 dev-mode bus/group routing re-confirmed); a dedicated multi-bus stress was not a separate release scenario — noted honestly."
  - "AGNT-01R (live provider-backed agent) left open with an honest note — only the deterministic/local-parser planner path is wired in v1."

patterns-established:
  - "Anti-overstatement gate (Pitfall 5): every supported-behavior claim in README/RELEASE_NOTES traces to a UAT scenario in 11-release-readiness-03-UAT.md; deferred items appear in both docs."
  - "Operator-gated GUI/audible UAT is recorded as terse observed results, then expanded into the plan's required (steps / observed state / Result: pass / cross-ref) format without fabricating details."

requirements-completed: [REL-01, REL-02, REL-03, DEV-01, DEV-02, AUD-01R, AUD-02R, AUD-03R, AUD-04R, VIS-01R, VIS-02R, VIS-03R]

# Metrics
duration: ~60min (across the resumed session: rebuild + Task 2 + Task 3)
completed: 2026-06-26
status: complete
---

# Phase 11 Plan 02: Release Build Smoke, Consolidated UAT & Documentation Summary

**Packaged ad-hoc-signed Apple Silicon `scrysynth.app` verified by a nine-scenario consolidated UAT, with README/RELEASE_NOTES rewritten and ROADMAP/STATE/REQUIREMENTS reconciled — REL-01/02/03 complete and v1 closed.**

## Performance

- **Duration:** ~60 min (resumed session: planner-fix rebuild + Task 2 evidence + Task 3 docs)
- **Tasks:** 3 (Task 1 steps 1–6, Task 2 nine-scenario UAT, Task 3 docs)
- **Files created:** 2 (`11-release-readiness-03-UAT.md`, `RELEASE_NOTES.md`)
- **Files modified:** 5 (`README.md`, `REQUIREMENTS.md`, `ROADMAP.md`, `STATE.md`, `11-release-readiness-02-BUILD-EVIDENCE.md`)

## Accomplishments

- **REL-01 verified:** `npm run tauri build` produced ad-hoc-signed `scrysynth.app` (139 MB) + `scrysynth_1.0.0_aarch64.dmg` (42 MB) for `aarch64-apple-darwin`; bundled `scrysynth-visual` signed inside the app; zero `APPLE_`/`TAURA_SIGNING_PRIVATE_KEY` leakage; first-run right-click → Open launch verified.
- **REL-02 verified:** all nine scenario areas passed against the rebuilt packaged app — save/open, graph edit, audio playback (real `scsynth`), scene/variation recall, macro control, hardware learn (CoreMIDI + local OSC), agent approval (scenario 7 now reachable), visual runtime (bundled minimal sidecar), panic recovery (audio + visual).
- **REL-03 verified:** `README.md` rewritten with Supported/Not-supported matrices and packaged-app install path; `RELEASE_NOTES.md` added; every supported-behavior claim traces to a UAT scenario.
- **Planner wired into the production GUI** (`415e8d8`, landed before Plan 02 resumed): `send_agent_message` → `handle_agent_message` → `plan_and_apply_agent_request`; approve/reject IPC param rename fixed; additive `remove … agent …` high-risk fallback. This closed the structural gap that had paused Plan 02.

## Task Commits

Each task was committed atomically:

1. **Task 1 (release build + ad-hoc sign + first-run smoke)** — `3541c7d` (chore: rebuild after 415e8d8) + `1258820` (docs: step 6 + UAT) — the rebuild refreshed stale artifacts; step 6 recorded from operator observation.
2. **Task 2 (consolidated nine-scenario UAT)** — `1258820` (docs: record packaged-app UAT, all nine passes).
3. **Task 3 (README/RELEASE_NOTES/planning-doc rewrite)** — this commit (docs: REL-03 reconciliation).

## Files Created/Modified

- `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md` — consolidated packaged-app UAT (nine scenario areas, all pass, Phase 7–10 cross-refs).
- `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md` — build evidence + 2026-06-26 rebuild notice + operator step-6 smoke (Gatekeeper/quarantine caveat).
- `RELEASE_NOTES.md` — new v1.0.0 release notes (Supported matrix, system requirements, deferred matrix, install, verification).
- `README.md` — rewritten from foundation-vs-runtime framing to v1 Supported/Not-supported matrices + packaged-app/developer install + build-from-source.
- `.planning/REQUIREMENTS.md` — REL-01/02/03 + AUD-01R..04R + VIS-01R..03R + DEV-01..02 marked `[x]`; AGNT-01R left open; traceability table updated.
- `.planning/ROADMAP.md` — Phase 11 marked Complete (2/2 plans); top "Current Picture" reframed to v1 packaged-and-evaluated.
- `.planning/STATE.md` — status `v1-released`; position, latest verification, decisions, and pending todos reconciled.

## Decisions Made

- **D-03 honest recording.** The operator observed no Gatekeeper "Open Anyway" prompt on right-click → Open. Root cause: the locally-built `.app` carries no `com.apple.quarantine` attribute (`xattr` shows only `com.apple.provenance`); Gatekeeper only intervenes for quarantined (downloaded) apps. The prompt is the end-user download path and is documented in README/RELEASE_NOTES. Recorded honestly rather than claiming a prompt that did not appear.
- **Scenario 7 trigger.** The high-risk pending action was produced by the input `remove agent layer`, matched by the additive `parse_remove_command` fallback from `415e8d8` (targets the first agent-owned, unlocked node; `RemoveNode` is `RiskTier::High`).
- **Inline execution over subagent dispatch.** Plan 02's remaining work was anti-overstatement-sensitive doc writing where the orchestrator held the exact operator-reported observations; GSD's sanctioned "close out manually" path for resumed/paused plans was used rather than spawning an executor that would have to re-derive those facts.

## Deviations from Plan

None — plan executed as written. The only structural event (planner-GUI wiring gap) was resolved by fix `415e8d8` before Plan 02 resumed; the pause-handoff's recommended fix scope (wire planner, add high-risk trigger, add integration test, rebuild) was fully delivered in that commit.

## Issues Encountered

- **Stale artifacts.** The 2026-06-24 packaged build predated `415e8d8`; a clean `npm run tauri build` was re-run on 2026-06-26 to refresh the `.app`/`.dmg` before UAT. Recorded in the BUILD-EVIDENCE rebuild notice.
- **Plan verify-command quirks.** Two of the plan's automated `verify` shell snippets produce false negatives: `rg -c '...' README.md | rg -q '^0$'` prints nothing on 0 matches (so WARN fires spuriously), and `test -f .../scrysynth.app` is false because an `.app` bundle is a directory. Both requirements are genuinely satisfied (verified directly): zero "foundation prototype" matches, and the `.app` directory exists and is adhoc-signed. No README/evidence changes were needed to satisfy the intent.

## User Setup Required

None for the plan itself. End-user setup (SuperCollider install, `SCRYSYNTH_SCSYNTH_PATH`, ad-hoc-signing first-run workflow) is documented in `README.md` and `RELEASE_NOTES.md`.

## Next Phase Readiness

- **v1 milestone (v1.0) is closeable.** Phases 7–11 are complete; REL-01/02/03 verified. Ready for `/gsd:complete-milestone` (or equivalent audit/archive).
- **Deferred beyond v1** (tracked, not blockers): live provider-backed agent (AGNT-01R), richer Bevy visuals / visible render window, physical-controller GUI click-through UAT, Windows/Linux/Intel/universal builds, full Developer ID notarization + auto-update, multiplayer.

---
*Phase: 11-release-readiness*
*Completed: 2026-06-26*
