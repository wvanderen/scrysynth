---
phase: 11-release-readiness
plan: 02
doc: uat
status: pass
target: aarch64-apple-darwin
app_under_test: src-tauri/target/release/bundle/macos/scrysynth.app
app_version: 1.0.0
app_signing: adhoc
app_build_source_commit: 415e8d8
executed: 2026-06-26
requirements_covered_by_this_doc: [REL-02 (verified), D-02 (verified), D-05 (verified for audio), D-06 (re-verified)]
# Consolidated packaged-app UAT. Each scenario was re-observed against the
# rebuilt packaged `.app` (commit 415e8d8, rebuilt 2026-06-26), not just dev
# mode. Cross-refs point at the Phase 7-10 dev-mode UAT that first verified
# each scenario. Result lines are the exact `Result: pass` format the plan's
# automated verify greps for.
---

# Phase 11 Plan 02: Consolidated Manual UAT (Packaged App)

**A single consolidated manual UAT pass against the packaged, ad-hoc-signed
`scrysynth.app` (rebuilt 2026-06-26 from commit `415e8d8`) passed all nine
REL-02 scenario areas — save/open, graph edit, audio playback, scene/variation
recall, macro control, hardware learn, agent approval, visual runtime, and
panic recovery. Every scenario was re-observed against the packaged `.app`,
not just the prior developer-mode UAT.**

This document closes **REL-02** for the v1 Apple-Silicon release. It re-confirms
— through the packaged artifact rather than `cargo test` or `npm run tauri dev`
— the runtime behaviors first verified in the Phase 7-10 developer-mode UAT
passes. It also exercises the two Plan 01 packaging changes end-to-end: the
bundled visual sidecar (D-06 refactor) and the planner-wiring fix (`415e8d8`)
that makes REL-02 scenario 7 reachable from the production GUI.

## App Under Test

- **Artifact:** `src-tauri/target/release/bundle/macos/scrysynth.app`
  (ad-hoc signed, `flags=0x10002(adhoc,runtime)`, `Mach-O thin (arm64)`).
- **Companion DMG:** `src-tauri/target/release/bundle/dmg/scrysynth_1.0.0_aarch64.dmg`.
- **Build source:** repo HEAD `415e8d8` (planner-wiring fix), rebuilt
  2026-06-26. See `11-release-readiness-02-BUILD-EVIDENCE.md` for the build +
  signing + secret-leak evidence and the Finder first-run smoke.

## Environment

- **Host target:** `aarch64-apple-darwin` (Apple Silicon; D-02).
- **Audio runtime:** real local SuperCollider via
  `SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth`
  (D-05).
- **Hardware-learn fixtures:** CoreMIDI virtual source + local OSC sender (the
  same approach Phase 9 used; physical-controller GUI click-through is out of
  scope per D-04/D-05 fences).
- **Agent-planner provider:** the in-app local parser provider routed through
  `plan_and_apply_agent_request` (`ParserPlannerProvider::default()`). A live
  remote LLM provider is NOT exercised here and is explicitly out of scope
  (AGNT-01R remains future hardening).

## First-run launch note (D-03 / T-11-04)

The packaged `.app` launches via Finder right-click → Open. On the build host
**no Gatekeeper "Open Anyway" prompt appeared**, because a locally-built `.app`
carries **no `com.apple.quarantine`** extended attribute (`xattr` on the bundle
shows only `com.apple.provenance`). Gatekeeper only intervenes for quarantined
apps; the quarantine attribute is applied by download agents (browsers, Mail,
AirDrop), not by `cargo build` / `tauri build` writing to disk locally.

This is expected macOS behavior, not a regression. The documented right-click →
Open → "Open Anyway" workflow (T-11-04) is the **end-user download path**: a
user who downloads the `.dmg` via a browser will hit the prompt and must use
that workflow. That workflow is documented in `README.md` and `RELEASE_NOTES.md`
(Task 3). The build host cannot reproduce the prompt without manually applying
a quarantine attribute, which would not be an honest end-user simulation.

The workspace window rendered and the bundled visual sidecar reached Ready
(confirmed transitively by scenarios 8 and 9 below running against the packaged
app).

---

## Scenario 1 — Save / open

**Steps performed:** Created a session in the packaged app, saved it to a JSON
file via the app's save flow, closed the session, reopened the file, and
confirmed the graph, macros, and scenes restored.

**Observed state:** Graph structure, macro definitions, and scene data all
restored from the saved JSON file on reopen.

**Result: pass**

**Cross-ref:** `.planning/phases/01-session-core-recall/01-UAT.md` (session
core save/open lineage) and `.planning/phases/03-performance-workspace/03-performance-workspace-01-SUMMARY.md`
(performance workspace session lineage). Requirement PERS-01.

---

## Scenario 2 — Graph edit

**Steps performed:** Added, removed, and re-routed a supported v1 graph
primitive from the graph workspace; confirmed the inspector and graph view
updated to reflect each edit.

**Observed state:** Inspector and graph view stayed in sync with each bounded
graph edit; removal and re-route both projected correctly.

**Result: pass**

**Cross-ref:** `.planning/phases/02-playable-audio-graph/02-playable-audio-graph-01-SUMMARY.md`
(graph edit command lineage). Requirements SESS-04.

---

## Scenario 3 — Audio playback (real SuperCollider)

**Prereq:** `SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth`
(D-05).

**Steps performed:** Started the audio runtime from the packaged app, confirmed
audible output from the default source-to-output graph, and adjusted a live
parameter during playback.

**Observed state:** Audio runtime reached a healthy/running state; audible
output from the default graph; the live parameter change was heard in the
output without rebuilding the session.

**Result: pass**

**Cross-ref:** `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md`
(dev-mode real-`scsynth` UAT). Requirements AUD-01R, AUD-02R.

---

## Scenario 4 — Scene / variation recall

**Steps performed:** Triggered a scene recall, saved a variation of the current
session, and restored that variation within the same session.

**Observed state:** Scene recall applied the recalled session state; the
variation saved and restored correctly within the working session.

**Result: pass**

**Cross-ref:** `.planning/phases/03-performance-workspace/03-performance-workspace-01-SUMMARY.md`
(scene/variation lineage). Requirements CTRL-02, CTRL-03.

---

## Scenario 5 — Macro control

**Steps performed:** Created/adjusted a macro mapped to both audio and visual
targets and drove it.

**Observed state:** Both the audio and visual targets moved with the macro.

**Result: pass**

**Cross-ref:** `.planning/phases/05-visual-sync-cross-modal/05-visual-sync-cross-modal-01-SUMMARY.md`
(cross-domain macro lineage). Requirement CTRL-01F (and feeds VIS-02R).

---

## Scenario 6 — Hardware learn (app-owned runtime path)

**Prereq:** CoreMIDI virtual source + local OSC sender (Phase 9 approach).
Physical-controller GUI click-through is explicitly out of scope (D-04/D-05).

**Steps performed:** Performed MIDI learn for a macro and OSC learn for scene
recall / transport, then confirmed post-learn routing drove the targets.

**Observed state:** Post-learn routing delivered control values from both the
MIDI and OSC sources to their bound targets (macro, scene, transport).

**Result: pass**

**Cross-ref:** `.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md`
(dev-mode hardware-input UAT). Requirements CTRL-04R, HW-01.

---

## Scenario 7 — Agent approval (deterministic/mock planner, now GUI-reachable)

**Steps performed:** Drove the in-app planner (local parser provider routed
through `plan_and_apply_agent_request`) to produce a high-risk pending action,
then approved one pending action and rejected one; confirmed freeze and reclaim
behave.

**Trigger input used:** `remove agent layer` — matched the additive
`parse_remove_command` fallback added by fix `415e8d8`, which targets the first
agent-owned, unlocked node. `RemoveNode` is `RiskTier::High`, so this produced a
high-risk pending action reachable from the production GUI for the first time
(this was the exact gap that blocked scenario 7 before the fix).

**Observed state:** A high-risk pending action appeared in the conversation
review UI; approve and reject both behaved correctly; freeze and reclaim
ownership behaved as verified in Phase 10.

**Result: pass**

**Cross-ref:** `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md`
(dev-mode deterministic/mock planner UAT). Requirements AGNT-02R, AGNT-03R.
(AGNT-01R — live provider — remains future hardening and was NOT exercised.)

---

## Scenario 8 — Visual runtime (bundled minimal sidecar)

**Prereq:** The visual sidecar does not auto-start — switched to the Runtime
workspace tab and clicked the Visual Runtime card's **Start** button.

**Steps performed:** Started the bundled visual sidecar, confirmed it reached
Ready, loaded a compiled scene, applied a live parameter batch, and ran the
Stop / Panic / Start cycle against the packaged sidecar.

**Observed state:** Sidecar reached Ready from the **bundled** binary (not the
dev-only missing-sidecar message — this is the key packaged-sidecar
re-confirmation of Plan 01's D-06 refactor); compiled scene loaded; parameter
batch applied; Stop/Panic/Start cycle behaved.

**Result: pass**

**Cross-ref:** `.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md`
(dev-mode visual sidecar UAT). Requirements VIS-01R, VIS-02R, VIS-03R.

---

## Scenario 9 — Panic recovery (audio + visual)

**Steps performed:** With audio active, triggered panic and confirmed sound
stopped immediately and the runtime returned to a restartable state; restarted
and confirmed recovery. Repeated the panic/restart cycle for the visual sidecar.

**Observed state:** Audio panic stopped sound immediately and left the runtime
restartable; restart recovered audio. Visual sidecar panic likewise left the
visual runtime restartable, and restart recovered the sidecar.

**Result: pass**

**Cross-ref:** `.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md`
and `.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md`
(dev-mode panic/restart lineage). Requirements AUD-04R, VIS-03R.

---

## Summary

| # | Scenario area | Result | Primary cross-ref | Requirements |
|---|---------------|--------|-------------------|--------------|
| 1 | Save / open | pass | Phase 1 UAT | PERS-01 |
| 2 | Graph edit | pass | Phase 2 SUMMARY | SESS-04 |
| 3 | Audio playback | pass | Phase 7 UAT | AUD-01R, AUD-02R |
| 4 | Scene / variation recall | pass | Phase 3 SUMMARY | CTRL-02, CTRL-03 |
| 5 | Macro control | pass | Phase 5 SUMMARY | CTRL-01F → VIS-02R |
| 6 | Hardware learn | pass | Phase 9 UAT | CTRL-04R, HW-01 |
| 7 | Agent approval | pass | Phase 10 UAT | AGNT-02R, AGNT-03R |
| 8 | Visual runtime | pass | Phase 8 UAT | VIS-01R, VIS-02R, VIS-03R |
| 9 | Panic recovery | pass | Phase 7 + 8 UAT | AUD-04R, VIS-03R |

**All nine REL-02 scenario areas passed against the packaged `scrysynth.app`.**
No failures, no packaging regressions detected.

## What is NOT claimed by this document

- **AGNT-01R (live provider-backed agent) is NOT verified.** Scenario 7 used
  the in-app local parser provider through the planner gates; a live remote LLM
  provider was not configured or exercised. AGNT-01R remains future hardening.
- **AUD-03R (explicit multi-bus / grouped routing) was not separately
  stressed.** The default source-to-output graph routes through the master bus
  (covered transitively by scenario 3), but a dedicated multi-bus/grouped-
  processing stress was not a separate packaged-app scenario. See the
  REQUIREMENTS reconciliation note for how AUD-03R is reflected.
- **Physical-controller GUI click-through is NOT verified.** Scenario 6 used a
  CoreMIDI virtual source and local OSC sender; physical hardware click-through
  remains release polish (D-04/D-05).
- **REL-01 (packaging) and the first-run smoke** are recorded in
  `11-release-readiness-02-BUILD-EVIDENCE.md`, including the Gatekeeper/quarantine
  caveat above.
- **Richer Bevy rendering / a visible render window is NOT verified.** Scenario
  8 exercised the minimal GPU-free sidecar (D-04), not a richer Bevy render.
