---
phase: 11-release-readiness
plan: 01
subsystem: infra
tags: [tauri, sidecar, macos, codesign, supercollider, bevy, externalBin]

# Dependency graph
requires:
  - phase: 08-real-visual-runtime-path
    provides: scrysynth-visual minimal sidecar binary + JSON-lines protocol + BevySidecarAdapter raw-Command launch path
  - phase: 07-real-supercollider-execution
    provides: resolve_scsynth_executable discovery chain (SCRYSYNTH_SCSYNTH_PATH -> PATH -> macOS bundle fallback)
provides:
  - v1 Tauri bundle config substrate (1.0.0, ad-hoc macOS signing, narrowed targets, externalBin, copyright)
  - scripts/prepare-sidecar.sh build helper producing the target-triple-suffixed release sidecar
  - BevySidecarAdapter dual-path launch (packaged shell-plugin sidecar + preserved dev/test Command override)
  - Scoped shell:allow-spawn capability for binaries/scrysynth-visual (T-11-02 mitigation)
  - AppHandle threading from lib.rs setup -> SessionStore -> visual manager -> adapter
  - End-user-actionable missing-binary diagnostics for both scsynth and the bundled visual sidecar
affects: [11-release-readiness-02 (release build smoke + consolidated UAT + docs)]

# Tech tracking
tech-stack:
  added: ["tauri-plugin-shell 2.3.5 (Rust crate)"]
  patterns:
    - "Dual-path sidecar adapter: packaged build uses app.shell().sidecar() via tauri-plugin-shell; dev/tests use raw std::process::Command with SCRYSYNTH_BEVY_PATH / executable override (no Tauri runtime)"
    - "tauri-build externalBin bootstrap: empty placeholder at the suffixed path unblocks tauri-build's per-build externalBin existence check during the sidecar's own cargo build, then overwritten with the real release binary"
    - "Normalized SidecarChild enum (Std(CommandChild-clone vs std::process::Child)) so the JSON-lines protocol layer (send_and_wait/wait_for_response) is shared across spawn mechanisms"
    - "Scoped shell capability: identifier shell:allow-spawn with a single allow entry (sidecar true, args fixed [\"--minimal\"]); no shell:default/allow-execute/args:true"

key-files:
  created:
    - scripts/prepare-sidecar.sh
  modified:
    - src-tauri/tauri.conf.json
    - src-tauri/Cargo.toml
    - package.json
    - src-tauri/capabilities/default.json
    - src-tauri/src/lib.rs
    - src-tauri/src/visual/adapter.rs
    - src-tauri/src/visual/bevy_sidecar.rs
    - src-tauri/src/visual/runtime_manager.rs
    - src-tauri/src/application/session_store.rs
    - src-tauri/src/audio/supercollider.rs
    - .gitignore

key-decisions:
  - "Ad-hoc macOS signing only (signingIdentity \"-\"); full Developer ID + notarization deferred and documented (D-03)"
  - "Apple Silicon (aarch64-apple-darwin) only for v1; universal binary deferred (D-02)"
  - "Minimal GPU-free visual sidecar is the packaged default via explicit --minimal arg (D-04, Pitfall 6)"
  - "scsynth stays an external user install; discovery chain unchanged, message polished only (D-05)"
  - "Visual sidecar resolved via tauri-plugin-shell app.shell().sidecar(); BevySidecarAdapter refactored off raw Command (D-06)"
  - "prepare-sidecar.sh builds the release profile only (never the ~272 MB debug blob) and appends the host-triple suffix via rustc --print host-tuple"
  - "REL-01 is delivered as build substrate here; its actual release-build verification is owned by Plan 02 (per this plan's verification block)"

patterns-established:
  - "Pattern: same-crate Rust binary bundled as a Tauri 2 sidecar via bundle.externalBin + tauri-plugin-shell + a beforeBuildCommand helper that appends rustc --print host-tuple"
  - "Pattern: external runtime (scsynth) kept external with a polished discovery-chain message rather than bundled"
  - "Pattern: per-task cargo check gate after every config/capability/plugin change (catches externalBin + capability validation early)"

requirements-completed: []  # REL-01 substrate delivered; REL-01 build verification owned by Plan 02

# Metrics
duration: 54min
completed: 2026-06-24
status: complete
---

# Phase 11 Plan 01: Release Packaging Build & Visual Sidecar Wiring Summary

**v1 Tauri bundle config (ad-hoc signed, Apple Silicon, narrowed targets), the visual sidecar bundled via tauri-plugin-shell's externalBin mechanism, and end-user-actionable missing-runtime diagnostics — the REL-01 packaging substrate and the ROADMAP #2 message-polish prerequisite for Plan 02's build smoke + UAT.**

## Performance

- **Duration:** 54 min
- **Started:** 2026-06-24T02:33:39Z
- **Completed:** 2026-06-24T03:27:50Z
- **Tasks:** 3 (Task 1 = pre-approved package-legitimacy gate; Task 2 = bundle config + sidecar pipeline; Task 3 = adapter refactor + diagnostics)
- **Files modified:** 11 (1 created, 10 modified)

## Accomplishments
- `npm run tauri build` now targets an ad-hoc-signed (`signingIdentity: "-"`), Apple-Silicon-only `scrysynth.app`/`.dmg` at version 1.0.0, with the visual sidecar bundled via `bundle.externalBin` and produced by `scripts/prepare-sidecar.sh` chained into `beforeBuildCommand`.
- `BevySidecarAdapter` launches the bundled sidecar through the official `app.shell().sidecar("scrysynth-visual").args(["--minimal"]).spawn()` API in the packaged path, while the dev/test override path (`SCRYSYNTH_BEVY_PATH` / `with_executable_override_and_args`, no `AppHandle`) keeps using `std::process::Command` so all Phase 8 tests pass unchanged.
- `AppHandle` flows from `lib.rs`'s `.setup` hook through the managed `Mutex<SessionStore>` into the visual manager/adapter, enabling the shell-plugin sidecar resolution.
- The `shell:allow-spawn` capability is tightly scoped to `binaries/scrysynth-visual` (`sidecar: true`, `args: ["--minimal"]`) — the V4 Access Control / T-11-02 Elevation-of-Privilege mitigation.
- Missing-binary diagnostics now speak to a packaged-app end user: scsynth message names SuperCollider install + `SCRYSYNTH_SCSYNTH_PATH` + the macOS default install path; the bundled visual sidecar failure gives reinstall guidance (not the dev-only env var).

## Task Commits

Each task was committed atomically:

1. **Task 1: Package-legitimacy gate for tauri-plugin-shell** — pre-approved by the operator before dispatch (no file mutation; approval recorded in the Task 2 commit body and here). crates.io repository = tauri-apps/plugins-workspace, max_stable_version 2.3.5, official Tauri 2 sidecar docs import `tauri_plugin_shell::ShellExt` under this exact name.
2. **Task 2: Tauri bundle config, sidecar build pipeline, plugin registration, capability scoping** — `c499dfa` (chore)
3. **Task 3: Refactor BevySidecarAdapter onto the shell-plugin sidecar API + polish missing-binary diagnostics** — `f936894` (feat)

## Files Created/Modified
- `scripts/prepare-sidecar.sh` (created) — release build helper producing `src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)` with a bootstrap placeholder to unblock tauri-build's per-build externalBin check.
- `src-tauri/tauri.conf.json` — version 1.0.0; `bundle.targets: ["app","dmg"]`; `bundle.macOS` (signingIdentity "-", minimumSystemVersion "11.0"); `bundle.copyright`; `bundle.externalBin: ["binaries/scrysynth-visual"]`; `beforeBuildCommand` chains `npm run build && ./scripts/prepare-sidecar.sh`.
- `src-tauri/Cargo.toml` — 1.0.0; `+ tauri-plugin-shell = "2"`.
- `package.json` — 1.0.0.
- `src-tauri/capabilities/default.json` — scoped `shell:allow-spawn` for `binaries/scrysynth-visual`.
- `src-tauri/src/lib.rs` — registers `tauri_plugin_shell::init()`; `.setup` hook threads `AppHandle` into the managed `SessionStore`.
- `src-tauri/src/visual/adapter.rs` — `set_app_handle` default no-op on the `VisualRuntimeAdapter` trait.
- `src-tauri/src/visual/bevy_sidecar.rs` — `SidecarChild` enum; `start_via_sidecar`/`start_via_command` dual paths; shared `complete_handshake`; branched `ensure_running`/`write_stdin`/`terminate_process`; `app_handle` field; `missing_sidecar_message_dev`/`_bundled`.
- `src-tauri/src/visual/runtime_manager.rs` — `set_app_handle` forwarding method.
- `src-tauri/src/application/session_store.rs` — `app_handle: Option<AppHandle>` field + `set_app_handle`; threaded into the visual manager in `start_visual_runtime`.
- `src-tauri/src/audio/supercollider.rs` — end-user-phrased `missing_scsynth_message` and spawn-failure message (discovery chain unchanged).
- `.gitignore` — `src-tauri/binaries/`.

## Decisions Made
None beyond the locked decisions D-01..D-06 encoded in the plan (1.0.0 version, Apple Silicon only, ad-hoc signing, minimal packaged default, scsynth external, tauri-plugin-shell sidecar). The two engineering decisions made during execution (both Rule 3 auto-fixes) are documented under Deviations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] prepare-sidecar.sh bootstrap chicken-and-egg**
- **Found during:** Task 2 (running the script to verify acceptance criteria)
- **Issue:** The plan's literal `prepare-sidecar.sh` runs `cargo build --release --bin scrysynth-visual`, but `tauri-build` (`copy_binaries`, tauri-build lib.rs:56-85) validates that every `bundle.externalBin` entry exists on **every** cargo build — including the sidecar binary's own build, which compiles before the suffixed binary can exist. So the script as written could never bootstrap: `cargo build --bin scrysynth-visual` fails with `resource path 'binaries/scrysynth-visual-aarch64-apple-darwin' doesn't exist`.
- **Fix:** The script now creates an empty placeholder at the suffixed path before invoking `cargo build` (so tauri-build's existence check passes), then overwrites the placeholder with the real release binary via `cp`. Standard Tauri sidecar bootstrap pattern.
- **Files modified:** scripts/prepare-sidecar.sh
- **Verification:** `./scripts/prepare-sidecar.sh` produces a 91 MB Mach-O arm64 release binary at `src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin`; subsequent `cargo check` passes.
- **Committed in:** c499dfa (Task 2 commit)

**2. [Rule 3 - Blocking] accidental drop of `MidiInputPort` import in lib.rs**
- **Found during:** Task 3 (cargo build after adding `use tauri::Manager;`)
- **Issue:** The Edit that added `use tauri::Manager;` matched against an import block that omitted the `MidiInputPort,` line (Read output had not surfaced it), so the edit silently dropped `MidiInputPort` from the `domain::session` import, producing `E0425: cannot find type MidiInputPort`.
- **Fix:** Restored `MidiInputPort` to the `domain::session` import list.
- **Files modified:** src-tauri/src/lib.rs
- **Verification:** `cargo build` passes; `list_midi_input_ports` command compiles.
- **Committed in:** f936894 (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary for the build to bootstrap and for the refactor to compile. No scope creep; both stay within the plan's locked decisions.

## Known Stubs
None. No hardcoded empty values, placeholder text, or unwired data sources were introduced. The bundled-sidecar launch is fully wired (the packaged path resolves the real binary via `app.shell().sidecar()`); only the dev/test path retains the override knob intentionally.

## Threat Flags
None. No security-relevant surface beyond the plan's `<threat_model>` was introduced. The shell capability (T-11-02) is mitigated and verified by the Task 2 capability-shape assertion. No `APPLE_*`/`TAURA_*` env vars were baked into `tauri.conf.json` (T-11-03); Plan 02's build smoke runs the `strings` grep over the packaged binary.

## Issues Encountered
- Measured release-stripped sidecar binary is 91 MB (research estimated 10–30 MB). The Cargo.toml has no `[profile.release] strip` config; the plan's Task 2 did not require adding one, and research §"Common Pitfalls" #4 actually cautions against over-stripping for panic traceability. Left unstripped; flagged here so Plan 02's build smoke can decide whether to add `strip = "debuginfo"` for bundle-size budgeting. Not a blocker for REL-01.

## User Setup Required
None for Plan 01. The runtime dependency a packaged end user still needs (SuperCollider install, or `SCRYSYNTH_SCSYNTH_PATH`) is documented by the polished scsynth message surfaced in the Runtime Health panel. Plan 02 owns the README/RELEASE_NOTES install + right-click→Open first-run instructions for the ad-hoc-signed `.dmg`.

## Next Phase Readiness
- Plan 01 delivers the complete build substrate for REL-01; it does **not** run `npm run tauri build` itself (that is Plan 02 Task 1, per this plan's `<verification>` block — REL-01 is a release artifact that cannot be honestly verified by a unit test).
- REL-01 is intentionally **not** marked complete in REQUIREMENTS.md here; Plan 02 owns REL-01/02/03 completion after the recorded build smoke + consolidated UAT.
- `cargo check`, the full `cargo test` suite (~235 tests, 0 failed), and `npm run build` all pass.
- Plan 02 is unblocked to: run the ad-hoc-signed release build + first-run smoke, run the consolidated nine-scenario UAT against the packaged app, and rewrite README/RELEASE_NOTES/planning docs against verified behavior.

## Verification Results (automated)
- Task 2 config-shape assertion: CONF_OK
- Task 2 capability-shape assertion: CAP_OK (scoped; no shell:default/allow-execute/args:true)
- Task 2 version + plugin-dep greps: VERSION_GREPS_OK
- Task 2 gitignore + plugin registration + beforeBuildCommand: OK
- Task 2 `./scripts/prepare-sidecar.sh`: produces Mach-O arm64 release binary at the suffixed path
- Task 2 `cargo check` + `npm run build`: pass
- Task 3 `cargo build`: pass
- Task 3 visual::bevy_sidecar lib tests: 7 passed (incl. `adapter_reports_missing_configured_sidecar_with_setup_guidance` unchanged)
- Task 3 `visual_sidecar_uat` integration test: 1 passed
- Task 3 audio::supercollider lib tests: 10 passed
- Task 3 sidecar API wired (`app.shell().sidecar`): SIDECAR_API_WIRED; `--minimal` arg passed; no raw Command in the prod path (dev/test comments only)
- Full `cargo test --manifest-path src-tauri/Cargo.toml`: all binaries green (~235 tests, 0 failed)

## Self-Check: PASSED
All 11 created/modified files exist on disk; both task commits (`c499dfa`, `f936894`) present in git log; `src-tauri/binaries/` is gitignored (0 entries in `git status`).

---
*Phase: 11-release-readiness*
*Completed: 2026-06-24*
