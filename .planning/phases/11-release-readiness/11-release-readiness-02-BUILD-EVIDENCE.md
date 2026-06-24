---
phase: 11-release-readiness
plan: 02
doc: build-evidence
status: steps-1-5-complete-step-6-pending-user
target: aarch64-apple-darwin
version: 1.0.0
signing: adhoc
created: 2026-06-24
requirements_covered_by_this_doc: [REL-01 (pending step 6), D-02 (verified), D-03 (pending step 6)]
# NOT a complete REL-01 record yet — Task 1 step 6 (GUI smoke) is pending user.
# Task 2 (consolidated UAT) and Task 3 (docs rewrite) remain after step 6.
---

# Phase 11 Plan 02: Release Build Evidence

**Ad-hoc-signed Apple Silicon release build of `scrysynth.app` + `scrysynth_1.0.0_aarch64.dmg` produced from Plan 01's packaging substrate; the bundled visual sidecar is signed inside the app, no `APPLE_`/`TAURA_SIGNING_PRIVATE_KEY` secrets leaked into the packaged main binary. First-run right-click→Open GUI smoke is PENDING USER.**

This document records the **shell-verifiable portion of Task 1 (steps 1-5)** of
`11-release-readiness-02-PLAN.md`. It was produced by a sequential executor that
cannot perform physical GUI interaction; **Task 1 step 6 (Finder right-click →
Open → Gatekeeper "Open Anyway" → confirm workspace renders) is explicitly
PENDING USER** and must be performed by the operator before Task 1, REL-01, or
D-03 can be marked complete.

Task 2 (consolidated nine-scenario UAT) and Task 3 (README/RELEASE_NOTES/planning
docs rewrite) are **out of scope for this evidence note** and depend on step 6
being performed first.

## Build Host & Tooling

- **Host target (D-02):** `aarch64-apple-darwin` (Apple Silicon only — v1 does
  not ship other targets).
- **App version:** `1.0.0` (per `src-tauri/tauri.conf.json` and `package.json`).
- **Signing identity:** `"-"` (ad-hoc; full Developer ID + notarization deferred
  per D-03).
- **Bundle targets:** `["app", "dmg"]` (no updater artifact).
- **Build driver:** `npm run tauri build`, whose `beforeBuildCommand` chains
  `npm run build && ./scripts/prepare-sidecar.sh` (Plan 01).

---

## Step 1 — Confirm host target is `aarch64-apple-darwin` (D-02)

**Command run:**
```
rustc --print host-tuple
```

**Observed output:**
```
aarch64-apple-darwin
```

**Result: PASS.** Host is Apple Silicon. v1's narrowed target (D-02) is
satisfied. If this had printed anything else (e.g. `x86_64-apple-darwin`), the
build would have halted — v1 does not ship other targets.

---

## Step 2 — Confirm release sidecar exists at the target-triple-suffixed path

`scripts/prepare-sidecar.sh` runs as part of `beforeBuildCommand` during
`npm run tauri build` (Step 3), but it had already produced the suffixed binary
during Plan 01. Confirmed the binary exists and is a release (not debug) Mach-O.

**Commands run:**
```
SIDECAR="src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin"
test -f "$SIDECAR" && echo FOUND || echo MISSING
ls -lh "$SIDECAR"
file "$SIDECAR"
stat -f %z "$SIDECAR"   # exact bytes
```

**Observed output:**
```
FOUND
-rw-r--r--@ 1 eggfam  staff    87M  src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin
src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin: Mach-O 64-bit executable arm64
exact bytes: 91583776
```

**Result: PASS (release binary present, arm64 Mach-O).**

### Observed deviation from research size estimate (NOT a failure)

The release sidecar measures **91,583,776 bytes (~87 MB via `ls -lh`, ~91 MB
via exact-byte / `du` rounding)**. This is consistent with the figure recorded
in `11-release-readiness-01-SUMMARY.md` ("91 MB").

The research estimate in `11-RESEARCH.md` §"Common Pitfalls" / "Domain"
projected a **~10-30 MB release-stripped** sidecar. The observed 87-91 MB is
therefore ~3x the upper end of that estimate.

**This is an observed deviation from a research estimate, not a build failure
and not a REL-01 blocker.** The Pitfall 4 concern was specifically about
avoiding the **~272 MB DEBUG blob** (`target/debug/scrysynth-visual`); the
release profile binary at ~87-91 MB is well below that and is the intended
`--release` artifact. `src-tauri/Cargo.toml` has no `[profile.release] strip`
config, so the binary retains debug symbols for panic traceability (Pitfall 4
cautions against over-stripping for exactly this reason).

**Carried-forward note for Task 3 (docs) and any future bundle-size budgeting:**
if a smaller `.dmg` is desired, a follow-on could add `[profile.release] strip =
"debuginfo"` (or `panic = "abort"` + partial strip) and re-measure. That is out
of scope for Phase 11 v1; it does not block REL-01.

---

## Step 3 — Run `npm run tauri build`; confirm `.app` and `.dmg` produced

**Command run:**
```
npm run tauri build
```

(Build was given a generous timeout. `beforeBuildCommand` ran
`npm run build && ./scripts/prepare-sidecar.sh`, then `cargo build --release`
compiled the main binary and the Tauri bundler assembled + ad-hoc-signed the
artifacts.)

**Observed output (tail):**
```
    Finished `release` profile [optimized] target(s) in 3m 05s
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
       Built application at: /Users/eggfam/dev/scrysynth/src-tauri/target/release/scrysynth
    Bundling scrysynth.app (/Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/macos/scrysynth.app)
     Signing with identity "-"
Signing with identity "-"
Signing /Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth-visual
...scrysynth-visual: replacing existing signature
Signing with identity "-"
Signing /Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth
...scrysynth: replacing existing signature
Signing with identity "-"
Signing /Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/macos/scrysynth.app
...scrysynth.app: replacing existing signature
        Warn skipping app notarization, no APPLE_ID & APPLE_PASSWORD & APPLE_TEAM_ID or APPLE_API_KEY & APPLE_API_ISSUER & APPLE_API_KEY_PATH environment variables found
    Bundling scrysynth_1.0.0_aarch64.dmg (/Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/dmg/scrysynth_1.0.0_aarch64.dmg)
     Running bundle_dmg.sh
    Finished 2 bundles at:
        /Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/macos/scrysynth.app
        /Users/eggfam/dev/scrysynth/src-tauri/target/release/bundle/dmg/scrysynth_1.0.0_aarch64.dmg
```

**Result: PASS.** Both bundle targets were produced:
- `src-tauri/target/release/bundle/macos/scrysynth.app`
- `src-tauri/target/release/bundle/dmg/scrysynth_1.0.0_aarch64.dmg`

### Warnings observed (neither is a failure)

1. **`Warn skipping app notarization, no APPLE_ID ...`** — **expected and
   correct per D-03.** v1 ships ad-hoc-signed only; full Developer ID +
   notarization is explicitly deferred. This is not a Plan 01 regression.
2. **`warning: ... block v0.1.6 ... rejected by a future version of Rust`** — a
   transitive-dependency future-incompatibility lint (from `block v0.1.6`,
   pulled in via the macOS objc stack). It is a pre-existing upstream warning,
   not introduced by Plan 01/02 and not caused by this task's changes. Out of
   scope to fix (scope boundary). The build succeeded regardless.

### No Plan 01 regressions detected

- **No updater artifact** was emitted (bundle config has no updater; correct).
- **No missing-signing-key failure** — ad-hoc identity `"-"` succeeded.
- **No `externalBin` existence failure** — `prepare-sidecar.sh`'s bootstrap
  placeholder (Plan 01 Rule-3 deviation fix) unblocked `tauri-build` correctly.

### Bundle artifact sizes

| Artifact | Path | Size |
|----------|------|------|
| `.app` bundle (total) | `src-tauri/target/release/bundle/macos/scrysynth.app` | **139 MB** (`du -sh`) |
| `.app` main binary | `.../scrysynth.app/Contents/MacOS/scrysynth` | 54,136,240 B (52 MB) |
| bundled sidecar | `.../scrysynth.app/Contents/MacOS/scrysynth-visual` | 91,073,616 B (87 MB) |
| `.dmg` | `src-tauri/target/release/bundle/dmg/scrysynth_1.0.0_aarch64.dmg` | 44,292,755 B (42 MB) |

(The `.dmg` is smaller than the `.app` because DMG compression deflates the
binaries; both contain the same ad-hoc-signed payload.)

---

## Step 4 — Confirm ad-hoc signing of `.app` and bundled sidecar (D-03, T-11-01)

**Command run:**
```
codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app
codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth-visual
codesign -dv src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth
```

**Observed output — `.app` bundle:**
```
Executable=.../scrysynth.app/Contents/MacOS/scrysynth
Identifier=com.lem.scrysynth
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20500 size=105714 flags=0x10002(adhoc,runtime) hashes=3297+3 location=embedded
Signature=adhoc
Info.plist entries=15
TeamIdentifier=not set
Runtime Version=26.5.0
Sealed Resources version=2 rules=13 files=10
Internal requirements count=0 size=12
```

**Observed output — bundled sidecar `scrysynth-visual`:**
```
Executable=.../scrysynth.app/Contents/MacOS/scrysynth-visual
Identifier=scrysynth-visual-555549447e76eedf079a347f9897446eca26f4aa
Format=Mach-O thin (arm64)
CodeDirectory v=20500 size=177722 flags=0x10002(adhoc,runtime) hashes=5547+2 location=embedded
Signature=adhoc
Info.plist=not bound
TeamIdentifier=not set
Runtime Version=26.5.0
Sealed Resources=none
Internal requirements count=0 size=12
```

**Result: PASS.**
- `.app` reports **`Signature=adhoc`** with `flags=0x10002(adhoc,runtime)`
  (D-03 satisfied; hardened runtime enabled).
- Bundled sidecar `scrysynth-visual` reports **`Signature=adhoc`** with
  `flags=0x10002(adhoc,runtime)` — **the sidecar is signed inside the bundle**
  (T-11-01 / Plan 01 D-06 refactor end-to-end).
- Both are `Mach-O thin (arm64)` (D-02 Apple-Silicon-only).
- `TeamIdentifier=not set` — expected for ad-hoc signing (D-03 defers Developer ID).

---

## Step 5 — T-11-03 secret-leakage check on the packaged main binary

**Command run:**
```
strings src-tauri/target/release/bundle/macos/scrysynth.app/Contents/MacOS/scrysynth \
  | rg 'APPLE_|TAURA_SIGNING_PRIVATE_KEY'
```

**Observed output:** *(none)*

```
RESULT: ZERO MATCHES (PASS — no build-time secrets leaked)
```

For thoroughness, the same check was run against the bundled sidecar:
```
strings .../scrysynth.app/Contents/MacOS/scrysynth-visual | rg 'APPLE_|TAURA_SIGNING_PRIVATE_KEY'
SIDECAR RESULT: ZERO MATCHES (PASS)
```

**Result: PASS.** No `APPLE_*` (Apple notarization credentials) or
`TAURA_SIGNING_PRIVATE_KEY` strings are baked into either packaged binary
(T-11-03 mitigated). This is consistent with Plan 01's finding that no
`APPLE_*`/`TAURA_*` env vars were written into `tauri.conf.json`.

---

## Step 6 — PENDING USER: First-run right-click → Open GUI smoke (D-03, T-11-04)

> **STATUS: NOT YET PERFORMED.** This step requires physical interaction with the
> macOS GUI (Finder + Gatekeeper prompt + observing the rendered workspace and
> Runtime Health panel). A sequential executor cannot perform it. The operator
> must perform the checklist below and then report the observed results so this
> section can be completed and Task 1 / REL-01 / D-03 closed.

### What the operator needs to do

Perform each of the following against the freshly built
`src-tauri/target/release/bundle/macos/scrysynth.app`, and record the observed
result for each line:

1. **Right-click → Open the packaged app (D-03, T-11-04 documented workflow).**
   - In Finder, navigate to
     `src-tauri/target/release/bundle/macos/scrysynth.app`.
   - **Right-click** (or Control-click) the `.app` → choose **Open**.
   - Observe: a Gatekeeper dialog should appear. Because the app is ad-hoc signed
     (not Developer ID / notarized), the dialog will warn that the app is from an
     unidentified developer.
   - Click **Open Anyway** (or the dialog's "Open" button, depending on macOS
     version).
   - **Expected:** the app launches (does not get silently blocked).
   - *Record:* did the Gatekeeper "Open Anyway" prompt appear? Did the app launch?

2. **Confirm the workspace window renders.**
   - After launch, a `scrysynth` window (title "scrysynth", 800×600 default per
     `tauri.conf.json`) should appear and render the workspace UI.
   - *Record:* did the workspace window render without an immediate crash?

3. **Observe the Runtime Health panel — missing-scsynth message (validates Plan 01 Task 3).**
   - **If a live `scsynth` is NOT configured** (no `SCRYSYNTH_SCSYNTH_PATH` env,
     no `scsynth` on `PATH`, no macOS bundle fallback): the Runtime Health panel
     for the audio runtime should show the **polished end-user missing-scsynth
     message** — naming SuperCollider install, `SCRYSYNTH_SCSYNTH_PATH`, and the
     macOS default install path
     (`/Applications/SuperCollider.app/Contents/Resources/scsynth`).
   - **If `scsynth` IS configured** (e.g. SuperCollider is installed at the macOS
     default path): the audio runtime may show a different state (Idle/Unknown or
     similar). That is fine; the missing-scsynth message specifically validates
     when scsynth is absent.
   - *Record:* what did the Runtime Health panel show for the audio runtime? If
     scsynth was absent, did the polished missing-scsynth message appear?

4. **Confirm the visual sidecar reaches Ready from the bundle (validates Plan 01 D-06).**
   - The Runtime Health panel for the **visual runtime** should transition from
     booting → **Ready** (NOT the dev-only "missing configured sidecar" message,
     which would indicate the bundle did not wire the sidecar correctly).
   - This is the key packaged-sidecar re-confirmation of Plan 01's D-06 refactor
     (the sidecar launches via `app.shell().sidecar()` from the bundled binary).
   - *Record:* did the visual runtime reach Ready? Or did it show a missing-sidecar
     / error state?

5. **Quit the app cleanly** (⌘Q or window close) to end the smoke.

### Resume signal for the next dispatch

Once step 6 is performed, the operator should report the observed results for
items 1-4 above. A subsequent dispatch will:
- fill in this Step 6 section with the observed values,
- execute **Task 2** (consolidated nine-scenario UAT against the packaged `.app`,
  with `SCRYSYNTH_SCSYNTH_PATH` set for the audio/panic scenarios), recording
  evidence in `11-release-readiness-03-UAT.md`, and
- execute **Task 3** (README + RELEASE_NOTES rewrite + ROADMAP/STATE/REQUIREMENTS
  reconciliation marking REL-01/02/03 complete).

If step 6 reveals a regression (e.g. the workspace does not render, the bundled
sidecar does NOT reach Ready, or the missing-scsynth message is the old dev-only
one), **halt and surface it for Plan 01 revision** rather than marking REL-01
complete.

---

## What is NOT claimed by this document

- **REL-01 is NOT marked complete.** Step 6 (first-run GUI smoke) is pending
  user; REL-01 requires the recorded build smoke AND the first-run launch
  evidence.
- **REL-02 is NOT marked complete.** Task 2 (consolidated nine-scenario UAT) has
  not been started; it depends on step 6 being performed first.
- **REL-03 is NOT marked complete.** Task 3 (README/RELEASE_NOTES/planning docs)
  has not been started; it depends on Tasks 1 + 2 being fully evidenced.
- **D-03 (right-click → Open documented workflow) is verified only
  configurally** (ad-hoc signing confirmed via `codesign`); the actual
  Gatekeeper prompt behavior is pending step 6.
- **No updates to `STATE.md`, `ROADMAP.md`, or `REQUIREMENTS.md`** were made by
  this dispatch. Those reconciliations belong to the Task 3 dispatch once all
  evidence is in.

## Threat model status (from Plan 02 `<threat_model>`)

| Threat | Status after steps 1-5 |
|--------|------------------------|
| T-11-03 (build-time env leaked into bundle) | **Mitigated & verified** — zero `APPLE_`/`TAURA_` matches (Step 5). |
| T-11-04 (unsigned `.dmg` Gatekeeper prompt) | **Accepted for v1** (D-03); workflow documentation is Task 3; actual Gatekeeper behavior is step 6. |
| T-11-06 (evidence docs contain local paths) | **Accepted** — this doc lives under `.planning/`, contains no credentials, is not distributed. |
| T-11-07 (no notarization provenance) | **Accepted for v1** (D-03); this doc records build host (`aarch64-apple-darwin`), target, and signature type (`adhoc`) as the v1 provenance record. |

---

*Built: 2026-06-24 on `aarch64-apple-darwin`, ad-hoc signed, `scrysynth` v1.0.0.*
*Steps 1-5 verified by sequential executor; step 6 pending operator GUI smoke.*
