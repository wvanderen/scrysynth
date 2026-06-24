---
phase: 11-release-readiness
plan: 02
name: Release Build Smoke, Consolidated Manual UAT & Documentation
type: hardening
status: planned
created: 2026-06-23
wave: 2
depends_on: [11-release-readiness-01]
td_epic: td-TBD
files_modified:
  - README.md
  - RELEASE_NOTES.md
  - .planning/ROADMAP.md
  - .planning/STATE.md
  - .planning/REQUIREMENTS.md
  - .planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md
  - .planning/phases/11-release-readiness/11-release-readiness-03-UAT.md
autonomous: false
requirements: [REL-01, REL-02, REL-03]
user_setup:
  - service: supercollider
    why: "Audio playback UAT (REL-02 audio scenario) requires a real local scsynth; Phase 7-10 UAT all used the macOS bundle fallback install."
    env_vars:
      - name: SCRYSYNTH_SCSYNTH_PATH
        source: "Path to scsynth inside SuperCollider.app (macOS default: /Applications/SuperCollider.app/Contents/Resources/scsynth)"
    dashboard_config:
      - task: "Install SuperCollider 3.14.x if not already present"
        location: "https://supercollider.github.io/downloads"
  - service: apple-developer-account
    why: "OUT OF SCOPE for v1 — full Developer ID + notarization is deferred per D-03; ad-hoc signing only. Listed here only so the operator does not block on it."
    env_vars: []

must_haves:
  truths:
    - "REL-01 (verified): `npm run tauri build` on an aarch64-apple-darwin host produces `scrysynth.app` and `scrysynth.dmg` under `src-tauri/target/release/bundle/`, ad-hoc signed, with the visual sidecar present beside the main executable."
    - "D-03 (verified): The ad-hoc-signed `.app` launches on the build host via right-click → Open → Open Anyway, and that workflow is documented in release notes."
    - "REL-02 (verified): A single consolidated manual UAT pass against the packaged `.app` succeeds across all nine scenario areas — save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approval, visual runtime, and panic recovery — and evidence is recorded."
    - "REL-03 (verified): README and a new RELEASE_NOTES.md describe what v1 actually supports and explicitly enumerate what is deferred (richer Bevy rendering, live provider-backed agent, physical-controller GUI click-through, Windows/Linux/Intel, full notarization), without overstating scaffolded paths."
    - "REL-03 (verified): ROADMAP.md, STATE.md, and REQUIREMENTS.md mark REL-01, REL-02, REL-03 (and the runtime-ready AUD/VIS/AGNT/DEV items Phase 11 covers) consistent with the verified UAT evidence."
    - "ROADMAP #4 (verified): No planning doc claims a behavior the packaged app did not demonstrate in UAT."
  artifacts:
    - path: ".planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md"
      provides: "Evidence that the release build produced a launchable ad-hoc-signed .app/.dmg with the bundled sidecar, plus the first-run right-click→Open smoke result"
      contains: "aarch64-apple-darwin"
    - path: ".planning/phases/11-release-readiness/11-release-readiness-03-UAT.md"
      provides: "Consolidated manual UAT checklist and pass/fail evidence across all nine REL-02 scenario areas, cross-referencing Phases 7-10 UAT for already-covered scenarios"
      contains: "REL-02"
    - path: "RELEASE_NOTES.md"
      provides: "v1 release notes with Supported / Not supported matrices, install instructions, and the ad-hoc-signing first-run workflow"
      contains: "Supported in v1"
    - path: "README.md"
      provides: "Release-accurate README replacing the foundation-vs-runtime disclaimer framing with a v1 supported-behavior description"
      contains: "v1"
    - path: ".planning/REQUIREMENTS.md"
      provides: "REL-01, REL-02, REL-03 marked complete; traceability rows updated"
      contains: "REL-01"
  key_links:
    - from: ".planning/phases/11-release-readiness/11-release-readiness-03-UAT.md (UAT evidence)"
      to: ".planning/REQUIREMENTS.md (REL-02 row) + README.md (What Runs Today) + RELEASE_NOTES.md (Supported matrix)"
      via: "every supported-behavior claim in the docs cites a UAT scenario that passed"
      pattern: "REL-02"
    - from: ".planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md"
      to: ".planning/REQUIREMENTS.md (REL-01 row) + RELEASE_NOTES.md (install/signing section)"
      via: "REL-01 completion is backed by the recorded build + first-run smoke result"
      pattern: "REL-01"
---

<objective>
Run the actual release build and first-run smoke against Plan 01's packaging substrate, execute one consolidated manual UAT pass across all nine REL-02 scenario areas against the packaged `.app`, then rewrite README/RELEASE_NOTES and reconcile ROADMAP/STATE/REQUIREMENTS so every supported-behavior claim is backed by recorded evidence and every deferred item is explicit.

Purpose: This plan converts Plan 01's buildable config into the three REL requirements' verified completion. The release build smoke proves REL-01. The consolidated UAT proves REL-02 (and re-confirms the AUD/VIS/AGNT runtime-ready requirements end-to-end through the packaged app rather than only through `cargo test`). The documentation rewrite proves REL-03 and closes ROADMAP success criterion #4. No new application features are added — every scenario exercised here was already verified in Phases 7-10 developer-mode UAT; Phase 11 re-runs them against the packaged artifact and records consolidated evidence.

Output: A launchable ad-hoc-signed `scrysynth.app`/`.dmg`; a build-evidence note; a consolidated UAT evidence doc; a rewritten README; a new RELEASE_NOTES.md; reconciled ROADMAP/STATE/REQUIREMENTS with REL-01/02/03 marked complete.

Locked decisions honored: D-02 (Apple Silicon only — build smoke runs on aarch64-apple-darwin and the docs name that target); D-03 (ad-hoc signing + documented right-click → Open, no notarization); D-04 (minimal GPU-free visual sidecar is the packaged default — UAT exercises it, not a visible Bevy window); D-05 (`scsynth` required as user install — UAT prereq and docs both state it).
</objective>

<execution_context>
@/Users/eggfam/.config/opencode/gsd-core/workflows/execute-plan.md
@/Users/eggfam/.config/opencode/gsd-core/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md
@.planning/phases/11-release-readiness/11-RESEARCH.md
@.planning/phases/11-release-readiness/11-release-readiness-01-PLAN.md

# Prior verified UAT evidence (cross-reference, do not re-derive)
@.planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md
@.planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md
@.planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md
@.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md
</context>

<tasks>

<task type="checkpoint:human-verify" gate="blocking-human">
  <name>Task 1: Release build, ad-hoc sign, and first-run right-click→Open smoke (REL-01, D-02, D-03)</name>
  <files>.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md</files>
  <read_first>
    - .planning/phases/11-release-readiness/11-release-readiness-01-PLAN.md (Plan 01 outputs: tauri.conf.json deltas, prepare-sidecar.sh, capability, BevySidecarAdapter refactor)
    - .planning/phases/11-release-readiness/11-RESEARCH.md §"System Architecture Diagram" (build flow), §"Common Pitfalls" (Pitfalls 2, 3, 4, 7), §"Environment Availability"
    - src-tauri/tauri.conf.json (post-Plan-01 state — confirm version 1.0.0, targets, externalBin, macOS signingIdentity)
    - scripts/prepare-sidecar.sh (confirm it exists and is executable)
  </read_first>
  <action>
    Run the release build on the aarch64-apple-darwin host and perform the first-launch smoke. This is a human-gated task because `npm run tauri build` drives `cargo build --release` + bundling + ad-hoc `codesign` and cannot be honestly verified by a unit test (REL-01 is a release artifact, per planning_context). Record the outcome in `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md`.

    Steps the operator/agent performs:
    1. Confirm the host target is Apple Silicon: `rustc --print host-tuple` must print `aarch64-apple-darwin` (D-02). If it does not, halt — v1 does not ship other targets.
    2. Confirm `scripts/prepare-sidecar.sh` ran as part of `beforeBuildCommand` and that `src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin` exists and is a release (not debug) binary — check its size is in the ~10-30 MB range, not ~272 MB (Pitfall 4).
    3. Run `npm run tauri build`. Capture the tail of the output. Confirm it produces both `scrysynth.app` and `scrysynth.dmg` under `src-tauri/target/release/bundle/` (D-02 narrowed targets). If the bundler emits an updater artifact or fails on a missing signing key, that is a config regression — return to Plan 01 Task 2.
    4. Confirm ad-hoc signing: `codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app` should show `Signature=adhoc` (D-03). Confirm the sidecar is signed as part of the bundle: `codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth-visual`.
    5. T-11-03 leakage check: run `strings src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth | rg 'APPLE_|TAURA_SIGNING_PRIVATE_KEY'` and confirm zero matches (no build-time secrets in the bundle).
    6. First-run right-click → Open smoke (D-03, T-11-04 documented workflow): in Finder, right-click `scrysynth.app` → Open → confirm the Gatekeeper "Open Anyway" prompt appears → open the app → confirm the workspace window renders. If a live `scsynth` is NOT configured, confirm the Runtime Health panel shows the polished missing-scsynth message (this validates Plan 01 Task 3's message polish in a real packaged app). Confirm the visual sidecar launches from the bundle (Runtime Health shows visual Ready, not the dev-only missing-sidecar message).
    7. Record all of the above (commands run, outputs, the Gatekeeper prompt behavior, the Runtime Health readings, file sizes) into `11-release-readiness-02-BUILD-EVIDENCE.md`.

    Do NOT attempt full Developer ID notarization or `xcrun notarytool` — D-03 explicitly defers that. Do NOT build for x86_64 or universal — D-02 scopes v1 to Apple Silicon only.
  </action>
  <what-built>
    Plan 01's bundle config is exercised end-to-end: `beforeBuildCommand` builds the release sidecar, `cargo build --release` builds the main binary, the Tauri bundler assembles the `.app`/`.dmg`, ad-hoc signing is applied, and the packaged app is launched once via the documented Gatekeeper workaround. Evidence is captured to `11-release-readiness-02-BUILD-EVIDENCE.md`.
  </what-built>
  <how-to-verify>
    1. `rustc --print host-tuple` prints `aarch64-apple-darwin`.
    2. `npm run tauri build` exits 0 and `src-tauri/target/release/bundle/macos/scrysynth.app` and `.../scrysynth.dmg` both exist.
    3. `codesign -dv .../scrysynth.app` reports `Signature=adhoc`; the bundled `scrysynth-visual` is signed within the bundle.
    4. `strings .../scrysynth | rg 'APPLE_|TAURA_SIGNING_PRIVATE_KEY'` returns nothing.
    5. Right-click → Open launches the workspace window; with no `scsynth` configured the Runtime Health panel shows the end-user missing-scsynth message; the visual sidecar reaches Ready from the bundle.
    6. `11-release-readiness-02-BUILD-EVIDENCE.md` exists and records all six items above with observed values.
  </how-to-verify>
  <resume-signal>Type "approved" if all six checks pass, or list the failing check(s) and the observed output so Plan 01 can be revised.</resume-signal>
  <verify>
    <human-check>Operator confirms the build produced an ad-hoc-signed `.app`+`.dmg` for aarch64-apple-darwin, the bundled sidecar launches, no `APPLE_`/`TAURA_` secrets leaked, and the first-run right-click→Open workflow succeeds.</human-check>
    <automated>test -f src-tauri/target/release/bundle/macos/scrysynth.app && test -f src-tauri/target/release/bundle/dmg/scrysynth*.dmg && codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app 2>&1 | rg -q 'adhoc' && test -f .planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md</automated>
  </verify>
  <acceptance_criteria>
    - Release `.app` and `.dmg` exist for `aarch64-apple-darwin` (REL-01, D-02).
    - Bundle is ad-hoc signed and the sidecar is signed within it (D-03, T-11-01).
    - No `APPLE_`/`TAURA_SIGNING_PRIVATE_KEY` strings in the packaged main binary (T-11-03 mitigated).
    - Right-click → Open launches the workspace (D-03 / T-11-04 documented workflow holds on the build host).
    - With no `scsynth` configured, the Runtime Health panel shows the polished end-user missing-scsynth message (validates Plan 01 Task 3 in a real packaged app).
    - The visual sidecar reaches Ready from the bundled binary (validates Plan 01's D-06 refactor end-to-end).
    - `11-release-readiness-02-BUILD-EVIDENCE.md` records all of the above.
  </acceptance_criteria>
  <done>Build-evidence note records a successful ad-hoc-signed Apple Silicon build with a launching workspace, a bundled-and-running visual sidecar, no secret leakage, and the polished missing-scsynth message visible when scsynth is absent.</done>
</task>

<task type="checkpoint:human-verify" gate="blocking-human">
  <name>Task 2: Consolidated manual UAT pass across all nine REL-02 scenario areas (REL-02)</name>
  <files>.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md</files>
  <read_first>
    - .planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md (the packaged app under test)
    - .planning/phases/07-real-supercollider-execution/07-real-supercollider-execution-07-UAT.md (audio playback + panic recovery scenario coverage already verified in dev mode)
    - .planning/phases/08-real-visual-runtime-path/08-real-visual-runtime-path-05-UAT.md (visual runtime scenario coverage)
    - .planning/phases/09-hardware-input-runtime-wiring/09-hardware-input-runtime-wiring-06-UAT.md (hardware learn scenario coverage via virtual MIDI + local OSC)
    - .planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-06-UAT.md (agent approval scenario coverage via deterministic/mock planner)
    - .planning/REQUIREMENTS.md (REL-02 scenario list: save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approvals, visual runtime, panic recovery)
  </read_first>
  <action>
    Execute one consolidated manual UAT pass against the packaged `scrysynth.app` (from Task 1) covering all nine REL-02 scenario areas, and record structured pass/fail evidence in `11-release-readiness-03-UAT.md`. This is human-gated because REL-02 is a manual UAT requirement (per planning_context, it cannot be proven by unit tests). The Phase 7-10 UAT evidence files above already cover these scenarios in developer mode; Phase 11's job is to re-confirm them against the packaged artifact and consolidate the evidence into a single doc, not to invent new scenarios.

    UAT prereq (user_setup): a real local `scsynth` is required for the audio playback and panic-recovery scenarios — set `SCRYSYNTH_SCSYNTH_PATH` to the scsynth inside SuperCollider.app (macOS default `/Applications/SuperCollider.app/Contents/Resources/scsynth`) before launching the packaged app. A CoreMIDI virtual source and a local OSC sender are required for the hardware-learn scenario (the same approach Phase 9 used; physical-controller click-through is explicitly out of scope per D-04/D-05 fences). The deterministic/mock planner (Phase 10) satisfies the agent-approval scenario — a live LLM provider is NOT required and is explicitly out of scope.

    For each of the nine scenario areas below, record: the exact steps performed, the observed Runtime Health / workspace state, a result on its own line as exactly `Result: pass` or `Result: fail`, and a cross-reference to the Phase 7-10 UAT doc that first verified this scenario in dev mode. Build the checklist directly into `11-release-readiness-03-UAT.md` (the `Result:` line format is mandatory — the automated verify greps for it):

    1. Save/open: create a session, save to a JSON file via the app, close, reopen the file, confirm graph/macros/scenes restore. (Cross-ref Phase 1/3 UAT lineage.)
    2. Graph edit: add/remove/re-route a supported v1 primitive; confirm the inspector and graph view update. (Cross-ref Phase 2.)
    3. Audio playback: with `SCRYSYNTH_SCSYNTH_PATH` set, start the audio runtime, confirm audible output from the default source-to-output graph, adjust a live parameter, hear the change. (Cross-ref Phase 7 UAT.)
    4. Scene/variation recall: trigger a scene recall; save a variation; restore it within the same session. (Cross-ref Phase 3.)
    5. Macro control: create/adjust a macro mapped to audio + visual targets; confirm both targets move. (Cross-ref Phase 5.)
    6. Hardware learn: with a CoreMIDI virtual source and local OSC sender, perform MIDI learn for a macro and OSC learn for scene recall/transport; confirm post-learn routing drives the targets. (Cross-ref Phase 9 UAT. Physical-controller click-through is out of scope.)
    7. Agent approval: drive the deterministic/mock planner to produce a high-risk pending action; approve one and reject one; confirm freeze and reclaim behave. (Cross-ref Phase 10 UAT. Live provider is out of scope.)
    8. Visual runtime: confirm the bundled minimal sidecar reaches Ready, loads a compiled scene, applies a live parameter batch, and that Stop/Panic/Start cycle works against the packaged sidecar. (Cross-ref Phase 8 UAT — this is the key packaged-sidecar re-confirmation of Plan 01's D-06 refactor.)
    9. Panic recovery: with audio active, trigger panic; confirm sound stops immediately and the runtime returns to a restartable state; restart and confirm it recovers. Repeat for the visual sidecar. (Cross-ref Phases 7 + 8 panic/restart.)

    If any scenario fails against the packaged app but passed in dev-mode UAT, that is a packaging regression (most likely the sidecar bundling in Plan 01) — halt and surface it for Plan 01 revision rather than marking REL-02 complete. Do NOT mark a scenario passed based on the prior dev-mode UAT alone; each scenario must be re-observed against the packaged `.app`.
  </action>
  <what-built>
    A single consolidated UAT evidence document proving the packaged v1 app performs across save/open, graph edit, audio playback, scene/variation recall, macro control, hardware learn, agent approval, visual runtime, and panic recovery. No code is changed in this task.
  </what-built>
  <how-to-verify>
    Open `11-release-readiness-03-UAT.md` and confirm it contains a section per scenario area (1-9) with: steps performed, observed state, pass/fail, and a cross-reference to the Phase 7-10 UAT doc. All nine scenarios must be marked pass. Any fail must be flagged with a root-cause note pointing back to Plan 01.
  </how-to-verify>
  <resume-signal>Type "approved" if all nine scenarios pass, or list the failing scenario(s) with observed behavior so Plan 01 can be revised before docs are written.</resume-signal>
  <verify>
    <human-check>Operator confirms all nine REL-02 scenario areas were re-verified against the packaged `.app` (not just dev mode) and recorded with pass status and cross-references.</human-check>
    <automated>F=.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md; test -f "$F" && rg -c 'Cross-ref' "$F" | rg -q '^([8-9]|[1-9][0-9]+)$' && ! rg -q -i 'Result:[[:space:]]*fail' "$F" && [ "$(rg -c -i 'Result:[[:space:]]*pass' "$F")" -ge 9 ]</automated>
  </verify>
  <acceptance_criteria>
    - All nine REL-02 scenario areas have a recorded pass against the packaged `scrysynth.app` (not dev mode).
    - Each scenario entry cross-references the Phase 7-10 UAT doc that first verified it.
    - Audio playback and panic recovery ran with a real local `scsynth`.
    - The visual runtime scenario confirms the bundled minimal sidecar (Plan 01 D-06 refactor) cycles Ready → scene load → parameter batch → Stop/Panic/Start.
    - No scenario is marked passed on dev-mode evidence alone.
    - `11-release-readiness-03-UAT.md` records the consolidated evidence.
  </acceptance_criteria>
  <done>Consolidated UAT doc shows nine passes against the packaged app with cross-references to Phases 7-10 evidence and no failures.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 3: Rewrite README, create RELEASE_NOTES.md, reconcile ROADMAP/STATE/REQUIREMENTS (REL-03, ROADMAP #4)</name>
  <files>
    README.md,
    RELEASE_NOTES.md,
    .planning/ROADMAP.md,
    .planning/STATE.md,
    .planning/REQUIREMENTS.md
  </files>
  <read_first>
    - README.md (current — still framed around foundation-vs-runtime; lines 1-146)
    - .planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md (what the release build actually delivered)
    - .planning/phases/11-release-readiness/11-release-readiness-03-UAT.md (what the packaged app actually does — source of truth for every supported-behavior claim)
    - .planning/REQUIREMENTS.md (REL-01/02/03 rows + AUD/VIS/AGNT/DEV rows that Phase 11 closes)
    - .planning/ROADMAP.md (Phase 11 section lines 137-146 + the deferred section 148-156)
    - .planning/STATE.md (current focus + decisions)
    - .planning/phases/11-release-readiness/11-RESEARCH.md §"Common Pitfalls" Pitfall 5 (README drift — claiming release-ready prematurely)
  </read_first>
  <action>
    Rewrite the user-facing and planning docs so every supported-behavior claim is backed by the Task 2 UAT evidence and every deferred item is explicit. Implement REL-03 and ROADMAP #4. No code changes.

    README.md: replace the foundation-vs-runtime disclaimer framing with a release-accurate description. Structure it as: (a) a one-paragraph "What Scrysynth is" kept from the current opener; (b) a "Supported in v1" matrix listing exactly what the Task 2 UAT verified (save/open, graph edit, audio playback via SuperCollider, scene/variation recall, macro control, MIDI/OSC learn via the app runtime, deterministic/mock agent approval, minimal visual sidecar, panic recovery) — each row cited to the consolidated UAT doc; (c) a "Not supported in v1 (deferred)" matrix listing: richer Bevy rendering / visible render window, live provider-backed agent orchestration, physical-controller GUI click-through UAT, Windows/Linux/Intel/universal builds, full Developer ID notarization + auto-update, multiplayer; (d) "Install" section with two paths — packaged-app install (download the `.dmg`, drag to Applications, right-click → Open → Open Anyway the first time per D-03, install SuperCollider separately per D-05) and developer install (the existing npm/cargo/SCRYSYNTH_SCSYNTH_PATH flow); (e) "Build from source" pointing at `npm run tauri build` and `scripts/prepare-sidecar.sh`. Remove the stale pre-release staging disclaimer language (the framing that positions the project as an unfinished prototype still undergoing runtime hardening) now that UAT passes — but do NOT swap it for marketing copy (Pitfall 5): the deferred matrix must be prominent. Keep the link to `.planning/ROADMAP.md`.

    RELEASE_NOTES.md (new): v1.0.0 release notes. Sections: "What's in this release" (the Supported matrix, version 1.0.0, Apple Silicon macOS target per D-02); "System requirements" (macOS on Apple Silicon, SuperCollider 3.14.x install required per D-05, ad-hoc signed so Gatekeeper requires right-click → Open the first time per D-03); "Known limitations & deferred work" (the Not-supported matrix, verbatim intent with the README); "How to install" (the packaged-app path with the right-click → Open workflow called out explicitly); "Verification" (link to the consolidated UAT doc and the build-evidence note). Do not claim notarization, Intel support, or richer visuals.

    .planning/REQUIREMENTS.md: mark REL-01, REL-02, REL-03 as complete (`[x]`) in the "v1 Release Requirements Still Needed" section, with a one-line note pointing at the build-evidence and UAT docs. Update the traceability table rows for REL-01..03 from "Pending" to "Complete (Phase 11)". Also reconcile the runtime-ready rows Phase 11 re-confirmed end-to-end through the packaged app (AUD-01R..04R, VIS-01R..03R, AGNT-02R/03R, CTRL-04R, HW-01, DEV-01/02) — mark those `[x]` complete where the packaged-app UAT now covers them, and leave AGNT-01R (live provider) and any item the UAT did not exercise unchanged with an honest note. Do not mark anything the UAT did not actually demonstrate.

    .planning/ROADMAP.md: update the Phase 11 section to record completion — add a "Status: Complete" line under Phase 11 and a short note pointing at the build-evidence and consolidated UAT docs. Update the top "Current Picture" paragraphs so they describe v1 as packaged-and-evaluated on Apple Silicon rather than "in v1 runtime hardening", while keeping the deferred items (live provider, richer Bevy, cross-platform, full notarization) visible. Do not touch the Phase 1-10 history sections except to reflect their final verified status consistently with REQUIREMENTS.md.

    .planning/STATE.md: update the YAML frontmatter `status` from `hardening` to `v1-released` (or `released` if that enum is preferred — keep it consistent with how prior milestones were labeled), bump `hardening_phases_completed` to 6 (Phases 7-11), update `current_stage` to reflect that v1 is packaged and UAT-verified on Apple Silicon with documented deferred work, and rewrite the "Current Position" and "Latest Verification" entries to cite the Phase 11 build-evidence and consolidated UAT docs. Keep the "Pending Todos" list honest: remove items Phase 11 closed and retain the genuinely deferred ones (live provider-backed agent, physical-controller click-through, richer Bevy, cross-platform, full notarization).

    Throughout: every supported-behavior claim must trace to a UAT scenario in `11-release-readiness-03-UAT.md`; every deferred item must appear in both README and RELEASE_NOTES. This is the REL-03 anti-overstatement gate (Pitfall 5).
  </action>
  <verify>
    <automated>test -f RELEASE_NOTES.md && rg -q 'Supported in v1' README.md && rg -q 'Not supported in v1' README.md && rg -q 'right-click' RELEASE_NOTES.md</automated>
    <automated>rg -c 'foundation prototype' README.md | rg -q '^0$' && echo "NO_STALE_STAGING_LANGUAGE" || echo "WARN: stale staging language remains"</automated>
    <automated>rg -q '\[x\] \*\*REL-01' .planning/REQUIREMENTS.md && rg -q '\[x\] \*\*REL-02' .planning/REQUIREMENTS.md && rg -q '\[x\] \*\*REL-03' .planning/REQUIREMENTS.md</automated>
    <automated>rg -q '11-release-readiness-03-UAT' README.md RELEASE_NOTES.md</automated>
    <automated>rg -q 'Complete' .planning/ROADMAP.md && rg -q '11-release-readiness-02-BUILD-EVIDENCE' .planning/ROADMAP.md</automated>
    <human-check>Operator reviews README + RELEASE_NOTES supported/deferred matrices against the UAT doc and confirms no claim overstates what UAT verified.</human-check>
  </verify>
  <acceptance_criteria>
    - README.md has explicit "Supported in v1" and "Not supported in v1" matrices, packaged-app install instructions including the right-click → Open workflow, and no residual pre-release staging disclaimer language (REL-03, Pitfall 5).
    - RELEASE_NOTES.md exists with version 1.0.0, Apple Silicon scope (D-02), SuperCollider install requirement (D-05), ad-hoc-signing first-run workflow (D-03), the deferred matrix, and links to the UAT and build-evidence docs.
    - REQUIREMENTS.md marks REL-01/02/03 complete with traceability rows updated; runtime-ready rows reconciled with what the packaged-app UAT actually demonstrated.
    - ROADMAP.md Phase 11 section records completion and the top-of-file picture reflects v1 packaged-and-evaluated status without hiding deferred work.
    - STATE.md frontmatter and prose reflect v1 release on Apple Silicon with honest pending todos.
    - Every supported-behavior claim in README/RELEASE_NOTES traces to the consolidated UAT doc; no claim overstates UAT evidence.
  </acceptance_criteria>
  <done>README and RELEASE_NOTES describe verified v1 behavior with explicit deferred matrices, ROADMAP/STATE/REQUIREMENTS mark REL-01/02/03 complete consistent with the recorded UAT evidence, and no doc claims a behavior the packaged app did not demonstrate.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Packaged `.app` → end-user macOS host | Gatekeeper mediates the first launch of the ad-hoc-signed app; this boundary is only mitigated by documentation in v1 (D-03). |
| End-user macOS host → external `scsynth` | UAT and end users both rely on a user-installed SuperCollider; the user vouches for that install (D-05). |
| Release artifact publication surface | When the `.dmg` is eventually distributed, the ad-hoc signature offers weak integrity; full signing is deferred. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11-03 | Information Disclosure | Build-time env leaked into bundled binary | mitigate | Task 1 step 5 runs a `strings ... \| rg 'APPLE_\|TAURA_SIGNING_PRIVATE_KEY'` check over the packaged main binary and records zero matches in the build-evidence doc. |
| T-11-04 | Spoofing (social) | Unsigned `.dmg` Gatekeeper prompt | accept | v1 ships ad-hoc signed only (D-03). Task 3 documents the right-click → Open → "Open Anyway" workflow in README and RELEASE_NOTES. Full notarization is an explicit follow-on. |
| T-11-06 | Information Disclosure | UAT/build-evidence docs may contain local file paths or config | accept | Evidence docs live under `.planning/` (developer-facing only), contain no credentials, and are not distributed with the `.app`. Low-value local artifacts; accepted. |
| T-11-07 | Repudiation | No release artifact provenance chain (no notarization ticket) | accept | Ad-hoc signing provides no notarization ticket; accepted for v1 per D-03. The build-evidence doc records the build host, target, and signature type as the v1 provenance record. |
</threat_model>

<verification>
Phase-level checks (Plan 02 + Plan 01 together close Phase 11):
- The release build produced an ad-hoc-signed `scrysynth.app`/`.dmg` for `aarch64-apple-darwin` (REL-01) — recorded in build-evidence doc.
- The consolidated UAT doc records nine passes across all REL-02 scenario areas against the packaged app, cross-referencing Phases 7-10 evidence.
- README and RELEASE_NOTES carry Supported/Not-supported matrices with no overstated claims, and every supported row traces to the UAT doc (REL-03, ROADMAP #4).
- ROADMAP/STATE/REQUIREMENTS mark REL-01/02/03 complete and reconcile the runtime-ready rows consistently with the UAT evidence.
- `npm test` and `npm run build` still pass (no code regression introduced by Plan 02's doc edits).
</verification>

<success_criteria>
- REL-01 verified by a recorded release build + first-run smoke.
- REL-02 verified by a consolidated nine-scenario UAT pass against the packaged app.
- REL-03 verified by README/RELEASE_NOTES/ROADMAP/STATE/REQUIREMENTS that match the recorded evidence and explicitly enumerate deferred work.
- Phase 11 is the final phase of the v1 runtime hardening milestone; its completion means the milestone can be audited/archived.
</success_criteria>

<output>
Create `.planning/phases/11-release-readiness/11-release-readiness-02-SUMMARY.md` when done, recording: the build-evidence and UAT doc paths, the README/RELEASE_NOTES deltas, the ROADMAP/STATE/REQUIREMENTS reconciliation, the final REL-01/02/03 status, and any deferred items carried forward. This SUMMARY plus Plan 01's SUMMARY together constitute the Phase 11 record.
</output>

## Artifacts this phase (plan) produces or modifies

**Created:**
- `RELEASE_NOTES.md` — v1.0.0 release notes with Supported/Not-supported matrices, install instructions (incl. right-click → Open for ad-hoc signing), SuperCollider install requirement, and links to evidence.
- `.planning/phases/11-release-readiness/11-release-readiness-02-BUILD-EVIDENCE.md` — recorded release build + ad-hoc sign + first-run smoke + secret-leak check results.
- `.planning/phases/11-release-readiness/11-release-readiness-03-UAT.md` — consolidated nine-scenario UAT checklist + pass evidence with Phase 7-10 cross-references.

**Modified:**
- `README.md` — rewritten from foundation-vs-runtime framing to v1 supported-behavior description with Supported/Not-supported matrices and packaged-app install path.
- `.planning/REQUIREMENTS.md` — REL-01/02/03 marked `[x]` complete; traceability rows updated; runtime-ready rows reconciled with packaged-app UAT.
- `.planning/ROADMAP.md` — Phase 11 marked complete; top-of-file picture updated to v1 packaged-and-evaluated status.
- `.planning/STATE.md` — status bumped to v1-released; position, latest verification, and pending todos reconciled with Phase 11 evidence.

**Generated (not committed):**
- `src-tauri/target/release/bundle/macos/scrysynth.app` and `src-tauri/target/release/bundle/dmg/scrysynth*.dmg` — the ad-hoc-signed release artifacts (build-output, gitignored under `target/`).
