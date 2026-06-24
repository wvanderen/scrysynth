---
phase: 11
type: hardening
created: 2026-06-23
---

# Phase 11: Release Readiness - Research

**Researched:** 2026-06-23
**Domain:** Tauri 2 desktop packaging, sidecar bundling, macOS code signing, runtime-discovery polish, manual UAT, release documentation
**Confidence:** HIGH (grounded in current `src-tauri/tauri.conf.json`, `Cargo.toml`, `src/bin/scrysynth-visual.rs`, `visual/bevy_sidecar.rs`, `audio/supercollider.rs`, and Tauri 2 official docs fetched 2026-06-23)

## Summary

Phase 11 is a **packaging + hardening + UAT phase**, not a feature phase. There are essentially **no new runtime features** to build: Phases 7-10 have already verified real audio execution, real visual sidecar lifecycle, real hardware learn/routing, and deterministic/mock agent orchestration. The work is (1) make `npm run tauri build` produce a usable macOS `.app`/`.dmg`, (2) bundle or correctly discover the two external runtime processes in the packaged app, (3) polish missing-binary error messages so a non-developer end user can recover without reading source, (4) run one consolidated manual UAT pass against the packaged app covering every requirement area, and (5) rewrite README/ROADMAP/STATE/REQUIREMENTS so they describe actual supported behavior rather than the historical foundation-vs-runtime split.

The hardest single decision is the **sidecar bundling strategy**, because there are two sidecars with very different shapes. `scsynth` is an external 1.5 MB universal Mach-O binary that is part of a 556 MB SuperCollider.app install — bundling just the bare `scsynth` executable is tempting but **breaks** once any UGen plugin is needed beyond the built-in set, so the recommendation is to **require SuperCollider install + path discovery (already implemented) and document it as an external runtime**. `scrysynth-visual` is an in-repo same-crate Cargo binary (auto-discovered from `src-tauri/src/bin/scrysynth-visual.rs`) that currently builds to a 272 MB debug blob (~10-30 MB release-stripped with Bevy/wgpu) — this one **should be bundled** via Tauri's official `externalBin` + `tauri-plugin-shell` sidecar pattern, which requires adding `tauri-plugin-shell = "2"` and refactoring `BevySidecarAdapter` off raw `std::process::Command`.

For macOS code signing and notarization: the pragmatic v1 recommendation is **ad-hoc signed local build first (`signingIdentity: "-"`) plus a documented path to Developer ID signing as a follow-on**, because full notarization requires a paid Apple Developer account ($99/yr), an "Account Holder" role to mint a Developer ID Application certificate, App Store Connect API keys or app-specific passwords, and a CI pipeline — none of which are blocking for an evaluatable v1 build. The Tauri 2 docs explicitly support ad-hoc signing as a valid intermediate state on Apple Silicon.

**Primary recommendation:** Phase 11 scope = (a) Tauri bundling that produces a launchable macOS `.app` with the visual sidecar bundled and `scsynth` discoverable via the existing env/PATH/macOS-bundle-fallback chain; (b) runtime-discovery message polish so missing-binary errors in the packaged app name the exact expected install; (c) one consolidated manual UAT pass that **does NOT require** live LLM provider, physical MIDI hardware, or richer Bevy rendering; (d) README + release notes that honestly describe "what runs today" (the verified path) without overstating scaffolded paths. macOS-first; Windows/Linux explicitly out of scope. Live provider-backed agent orchestration, richer Bevy rendering, and physical-controller click-through are explicitly out of Phase 11 scope.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Tauri `.app`/`.dmg` bundling | Build tooling (`tauri-cli` + `cargo`) | Rust core | Tauri-cli owns bundle layout; Rust owns binary targets. No frontend change. |
| Visual sidecar binary | Rust core (`src-tauri/src/bin/scrysynth-visual.rs`) | Build tooling (`externalBin` + target-triple copy) | The sidecar is a same-crate Cargo binary; bundling is a build-step concern. |
| `scsynth` discovery | Rust core (`audio/supercollider.rs::resolve_scsynth_executable`) | OS install path | App discovers; user installs. No bundle. |
| Code signing identity | Build tooling (Tauri config + `codesign`) | Apple Developer account (out-of-band) | Identity is provided to Tauri via `APPLE_SIGNING_IDENTITY` env or `bundle.macOS.signingIdentity`; no app code change. |
| Missing-binary error UX | Rust core (error strings in `supercollider.rs`, `bevy_sidecar.rs`) | Frontend Runtime Health panel (projection of backend status) | Backend owns the actionable message; frontend projects it. Both already exist — Phase 11 is message polish, not new plumbing. |
| Manual UAT execution | Human (release operator) | — | Cannot be automated; click-through evidence artifacts only. |
| Release notes / README accuracy | Docs (`.planning/`, `README.md`) | — | Pure documentation work driven by the verified-state audit. |

## Standard Stack

### Core (already in repo — no new installs required for Phase 11's primary path)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@tauri-apps/cli` | `^2` (current resolve: 2.11.3 on npm) | Drives `npm run tauri build` bundle step | Already a devDependency in `package.json`; Tauri's official build/bundle driver. `[VERIFIED: npm registry — 2.11.3 latest as of 2026-06-23, 1.8M weekly downloads]` |
| `tauri` (Rust crate) | `2` (STACK.md target: 2.10.x) | Tauri runtime + bundler config schema | Already in `src-tauri/Cargo.toml`. |
| `tauri-plugin-opener`, `tauri-plugin-dialog` | `2` | Existing plugins registered in `lib.rs` | Already present; no change. |
| Rust toolchain | Stable, 1.96.0 verified on this machine | Compiles main binary + `scrysynth-visual` sidecar | Already installed. Tauri plugin MSRV floor is 1.77.2 per STACK.md; 1.96.0 is well above. |

### Supporting (required only if the planner picks the idiomatic sidecar-bundling option)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri-plugin-shell` | `2` | Provides `app.shell().sidecar("scrysynth-visual")` API that resolves the bundled binary path with the target-triple suffix | **Required** to use Tauri's official `bundle.externalBin` mechanism. Without it, the bundler copies the suffixed binary but no built-in API resolves its packaged location. `[VERIFIED: Tauri 2 docs — https://v2.tauri.app/develop/sidecar/]` |
| `@tauri-apps/plugin-shell` | `^2.3.5` (STACK.md pin) | JS-side `Command.sidecar()` (only needed if the frontend ever spawns the sidecar directly) | **Not required for Phase 11** — the backend owns sidecar lifecycle through `BevySidecarAdapter`. The JS package can be deferred. `[CITED: https://v2.tauri.app/develop/sidecar/]` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tauri-plugin-shell` sidecar API + `externalBin` | Manual `std::process::Command` + custom path resolution against `resource_dir()`/`current_exe()` | The current code already uses raw `Command`. Keeping it avoids adding a plugin and capability entry, but Tauri's `externalBin` does not place the binary under `resource_dir` — it goes next to the main executable inside the `.app/Contents/MacOS/` dir, and the only Tauri API that resolves that location for you is `app.shell().sidecar()`. Manual resolution means hardcoding the relative path, which is fragile across Tauri versions. **Recommend the plugin route for v1.** |
| Bundling `scsynth` as a Tauri resource | Requiring SuperCollider install + path discovery (current behavior) | Bundling bare `scsynth` (~1.5 MB) breaks the moment any non-built-in UGen is needed because SC's plugin directory lives in `SuperCollider.app/Contents/Resources/`. Bundling the full 556 MB SC app is out of scope for a music instrument installer's UX. Path discovery + clear setup messaging is the standard pattern for SC-hosting apps (e.g., Sonic Pi, sc3-plugins workflow). |
| Full Developer ID Application + notarization in Phase 11 | Ad-hoc signing (`signingIdentity: "-"`) for v1; defer full signing | Ad-hoc lets the build produce a launchable Apple Silicon `.app` immediately. Full notarization requires an Apple Developer account, Account Holder role, App Store Connect key, and a CI runner. Phase 11 deliverable is "evaluatable v1 desktop instrument" — ad-hoc + clear install instructions satisfies that. |

**Installation (only if planner selects the `tauri-plugin-shell` route):**

```bash
# In src-tauri/
cargo add tauri-plugin-shell@2
# No npm install needed for backend-owned sidecar; only if JS-side spawning is added later.
```

**Version verification (run 2026-06-23):**

```text
$ npm view @tauri-apps/cli version
2.11.3
$ rustc --print host-tuple
aarch64-apple-darwin
$ cargo --version
cargo 1.96.0
$ sw_vers -productVersion
26.5.1
```

## Package Legitimacy Audit

> Phase 11's primary path requires at most one new dependency (`tauri-plugin-shell`, Rust crate). No npm packages are required unless the planner adds JS-side sidecar spawning. The audit below covers the only packages the planner is likely to consider.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| `@tauri-apps/cli` | npm | mature (years; latest publish 2026-06-19) | ~1.83M/wk | `github.com/tauri-apps/tauri` | SUS (false positive — "too-new" heuristic fired on the latest patch publish) | Approved — already a devDependency in `package.json`; legitimacy confirmed by official Tauri ownership + multi-year history + massive download volume |
| `@tauri-apps/api` | npm | mature (latest 2026-06-17) | ~2.16M/wk | `github.com/tauri-apps/tauri` | SUS (same false-positive) | Approved — already a dependency |
| `tauri-plugin-shell` (Rust) | crates.io | mature (Tauri 2 GA) | — | `github.com/tauri-apps/plugins-workspace` | not run through seam (crates.io legitimacy not in the npm seam) `[ASSUMED]` based on official Tauri docs citation | Planner should add a `checkpoint:human-verify` before adding the dependency, per the protocol for `[ASSUMED]` packages. Verification is trivial: the Tauri docs at https://v2.tauri.app/develop/sidecar/ explicitly import `tauri_plugin_shell::ShellExt`, and the crate is published from the official `tauri-apps/plugins-workspace` monorepo. |

**Packages removed due to SLOP verdict:** none.
**Packages flagged as suspicious [SUS]:** `@tauri-apps/cli` and `@tauri-apps/api` — but the SUS verdict is a false positive from the seam's "too-new" heuristic firing on routine patch publishes. Both packages are the official Tauri toolchain and already in `package.json`; **no planner action required**.

**Packages tagged `[ASSUMED]`:** `tauri-plugin-shell` (Rust). Planner should add `checkpoint:human-verify` before `cargo add`, even though the official Tauri sidecar docs import it directly.

## Architecture Patterns

### System Architecture Diagram

The Phase 11 release build flows input (source + config) through build stages to packaged output, and the packaged app then discovers its runtime dependencies at launch:

```
[Source repo]
   │
   ├── src-tauri/Cargo.toml (binaries: scrysynth, scrysynth-visual)
   ├── src-tauri/tauri.conf.json (bundle config)
   ├── src-tauri/resources/synthdefs/v1/*.scsyndef  ────► bundle.resources
   │
   ▼
[npm run tauri build]
   │
   ├── (1) beforeBuildCommand: npm run build  ──► ../dist (frontend bundle)
   │
   ├── (2) cargo build --release  ──► target/release/scrysynth
   │                                target/release/scrysynth-visual
   │
   ├── (3) externalBin step: copy scrysynth-visual-aarch64-apple-darwin
   │                                 into .app/Contents/MacOS/
   │
   ├── (4) resources step: copy synthdefs  ──► .app/Contents/Resources/
   │
   ├── (5) bundle step: assemble .app, then .dmg
   │
   └── (6) optional: ad-hoc codesign (-) or Developer ID + notarize
       │
       ▼
[Packaged app: Scrysynth.app / Scrysynth.dmg]
   │
   ▼ (launch)
[Runtime discovery]
   │
   ├── scsynth  ◄── SCRYSYNTH_SCSYNTH_PATH  OR  PATH/scsynth  OR  /Applications/SuperCollider.app/.../scsynth
   │       │
   │       └──►  If missing → audio runtime reports Failed with setup message
   │
   ├── scrysynth-visual  ◄──  bundled (app.shell().sidecar())
   │       │                 OR SCRYSYNTH_BEVY_PATH (dev override)
   │       │                 OR PATH/scrysynth-visual (dev)
   │       └──►  If missing → visual runtime reports Failed with setup message
   │
   └── synthdefs/v1/*.scsyndef  ◄──  resource_dir()/synthdefs/v1/  (macOS .app: Contents/Resources/)
                   │
                   └──►  Read at SC topology load via resolve_resource_path()
```

A reader can trace the primary use case: source → build → packaged `.app` → on launch, discover `scsynth` externally and `scrysynth-visual` internally, then read bundled synthdefs.

### Recommended Project Structure (Phase 11 deltas only)

```
src-tauri/
├── Cargo.toml                  # + tauri-plugin-shell = "2" (if planner picks idiomatic route)
├── tauri.conf.json             # + bundle.externalBin, bundle.macOS config, version bump
├── capabilities/
│   └── default.json            # + shell:allow-spawn for scrysynth-visual sidecar
├── resources/synthdefs/v1/     # unchanged — already bundled
├── binaries/                   # NEW (build-step output, gitignored)
│   └── scrysynth-visual-aarch64-apple-darwin   # built by beforeBuildCommand
└── src/bin/scrysynth-visual.rs # unchanged source

.planning/phases/11-release-readiness/
└── 11-release-readiness-XX-UAT.md  # consolidated UAT evidence

README.md                       # rewritten for release accuracy
RELEASE_NOTES.md                # NEW — v1 supported-behavior doc
```

### Pattern 1: Same-crate Rust binary as Tauri sidecar (RECOMMENDED)

**What:** The in-repo `scrysynth-visual` binary (already auto-discovered by Cargo from `src/bin/scrysynth-visual.rs`) gets bundled via Tauri's `externalBin` mechanism and resolved at runtime via `tauri-plugin-shell`'s `app.shell().sidecar()` API.

**When to use:** Whenever a Tauri 2 app needs to spawn a Rust binary built from the same workspace as a supervised child process.

**Example config (`src-tauri/tauri.conf.json` delta):**

```jsonc
// Source: https://v2.tauri.app/develop/sidecar/  [CITED]
{
  "bundle": {
    "externalBin": ["binaries/scrysynth-visual"]
  },
  "build": {
    "beforeBuildCommand": "npm run build && ./scripts/prepare-sidecar.sh"
  }
}
```

**Build helper (`scripts/prepare-sidecar.sh`):**

```sh
#!/usr/bin/env bash
# Source: Tauri 2 sidecar docs pattern  [CITED: https://v2.tauri.app/develop/sidecar/]
set -euo pipefail
TARGET="$(rustc --print host-tuple)"   # e.g. aarch64-apple-darwin
mkdir -p src-tauri/binaries
cargo build --release --manifest-path src-tauri/Cargo.toml --bin scrysynth-visual
cp "src-tauri/target/release/scrysynth-visual" \
   "src-tauri/binaries/scrysynth-visual-${TARGET}"
```

**Runtime resolution (Rust, in `BevySidecarAdapter`):**

```rust
// Source: https://v2.tauri.app/develop/sidecar/  [CITED]
use tauri_plugin_shell::ShellExt;

// Inside a Tauri command or setup hook that has the AppHandle:
let sidecar = app
    .shell()
    .sidecar("scrysynth-visual")           // <-- bare name, no path, no target-triple suffix
    .expect("failed to create scrysynth-visual sidecar command");
let (mut rx, mut child) = sidecar
    .args(["--minimal"])                   // or no args for visible Bevy window
    .spawn()
    .expect("failed to spawn sidecar");
// Read JSON-lines from rx; write JSON-lines to child.stdin as today.
```

**Capability entry (`src-tauri/capabilities/default.json` delta):**

```jsonc
// Source: https://v2.tauri.app/develop/sidecar/  [CITED]
{
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default",
    {
      "identifier": "shell:allow-spawn",
      "allow": [
        { "name": "binaries/scrysynth-visual", "sidecar": true,
          "args": ["--minimal"] }
      ]
    }
  ]
}
```

### Pattern 2: External runtime discovery chain (CURRENT — keep, polish messages)

**What:** `scsynth` is discovered in this order: `SCRYSYNTH_SCSYNTH_PATH` env → `PATH` lookup → macOS bundle fallback `/Applications/SuperCollider.app/Contents/Resources/scsynth`. This is **already implemented** in `audio/supercollider.rs::resolve_scsynth_executable` (lines 459-477) and verified by Phase 7 UAT.

**When to use:** For external runtimes that ship as part of a larger install (SuperCollider, FFmpeg, etc.).

**Why it stays:** Bundling bare `scsynth` breaks once any UGen plugin outside the built-in set is needed; bundling the full SuperCollider.app is unreasonable. Path discovery + clear messaging is the standard pattern.

### Anti-Patterns to Avoid

- **Hardcoding `../Resources/` paths in app code for non-resource files.** `resolve_resource_path()` in `audio/supercollider.rs:436-457` does this today for synthdefs, which is fine because they are declared as `bundle.resources`. Do NOT extend the same pattern for the sidecar binary — `externalBin` does not place binaries under Resources; it places them next to the main executable. Use `app.shell().sidecar()` instead.
- **Treating `bundle.targets: "all"` as fine for v1.** `"all"` on macOS produces `.app`, `.dmg`, AND updater artifacts. If you do not configure the updater signing key (`TAURA_SIGNING_PRIVATE_KEY`), the updater bundle may emit spurious warnings. For v1, set `targets: ["app", "dmg"]` to scope the output.
- **Bundling a `debug` sidecar binary.** The current `target/debug/scrysynth-visual` is 272 MB. The `beforeBuildCommand` helper MUST call `cargo build --release --bin scrysynth-visual`, and the script MUST `strip` (or rely on `panic = "abort"` + release profile) to keep the bundle reasonable (~10-30 MB release-stripped with Bevy/wgpu default features).
- **Skipping the host-tuple suffix.** `externalBin: ["binaries/scrysynth-visual"]` requires the file `binaries/scrysynth-visual-aarch64-apple-darwin` to exist at bundle time. The build helper must append the suffix via `rustc --print host-tuple`; a bare copy will silently fail.
- **Rewriting README as marketing copy.** REL-03 is "describe supported behavior **without overstating scaffolded paths**". The current README is already honest; the Phase 11 job is to add a "Supported in v1 / Not supported in v1" matrix and remove the foundation-vs-runtime disclaimer language once UAT passes.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Resolve packaged sidecar binary path across platforms | Manual `current_exe()` + relative path heuristics | `tauri_plugin_shell::ShellExt::sidecar()` | Tauri already handles target-triple suffix resolution, macOS `.app/Contents/MacOS/` layout, Windows `.exe`, and Linux PIE expectations. |
| Generate `.app` bundle and `.dmg` from scratch | Custom `cargo-bundle` or hand-rolled plist + disk image | `@tauri-apps/cli` `tauri build` command | Tauri's bundler is the only path that gets Info.plist, entitlements, icons, and codesign hooks consistently right. |
| Code-sign the `.app` manually with raw `codesign` invocations | Shell scripts calling `codesign --force --deep` | Tauri's signing config (`bundle.macOS.signingIdentity` + `APPLE_SIGNING_IDENTITY` env) | Tauri already orchestrates signing of the main binary, sidecar, frameworks, and resources in the correct order; manual `--deep` signing is deprecated by Apple and produces broken bundles. |
| Notarize via raw `xcrun notarytool` + poll loop | Custom notarization script | Tauri's `APPLE_ID`/`APPLE_PASSWORD`/`APPLE_TEAM_ID` env (or App Store Connect key envs) | Tauri already wraps `notarytool` submit/wait/staple correctly. Only fall back to raw `notarytool` if you need `--skip-stapling` for a first pass. |
| Ship a custom "first run" diagnostic UI for missing runtimes | New frontend page | Existing Runtime Health panel (already surfaces `Failed` with the backend's message string) | The panel already exists and is verified across Phases 7-10. Phase 11 only polishes the message strings. |

**Key insight:** Phase 11 introduces **zero** new application features. Every primitive needed (Runtime Health panel, OSC adapter, sidecar lifecycle, panic/restart, error projection) already exists and is verified. The work is **build configuration, message strings, UAT, and docs**. Treat any plan task that says "implement X feature" as scope creep.

## Runtime State Inventory

> Phase 11 is primarily a packaging/docs phase, not a rename/refactor phase. The relevant runtime state to audit is **what state a packaged `.app` needs to discover or carry** that the dev-mode `cargo run` does not.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — sessions are user JSON files; the app stores no system-level state. SQLite was listed in STACK.md but is not used in the current code (`rusqlite` is not in `Cargo.toml`). | None. |
| Live service config | None — no external services are configured out-of-band. | None. |
| OS-registered state | The packaged `.app` will register its bundle identifier `com.lem.scrysynth` with LaunchServices on first launch. No prior OS-registered state exists. | None. |
| Secrets/env vars | `SCRYSYNTH_SCSYNTH_PATH`, `SCRYSYNTH_BEVY_PATH`, `SCRYSYNTH_VISUAL_MODE` — all optional dev overrides. In the packaged app, `SCRYSYNTH_BEVY_PATH` becomes unnecessary once `externalBin` is wired; `SCRYSYNTH_SCSYNTH_PATH` remains the user-facing escape hatch for non-standard SuperCollider installs. `APPLE_SIGNING_IDENTITY` / `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID` are build-time-only env vars and must NOT ship in the bundle. | Code edit: bundled sidecar resolution must gracefully ignore `SCRYSYNTH_BEVY_PATH` when the bundled binary is present (or honor it as a dev override). Build script must scrub env before signing. |
| Build artifacts | `src-tauri/target/debug/` carries 272 MB `scrysynth-visual` debug blob. `src-tauri/binaries/` will be created by the new build helper and must be gitignored. Existing `src-tauri/scrysynth-session-test.json` is a checked-in test fixture, not a release artifact. | Add `src-tauri/binaries/` to `.gitignore`. Ensure `beforeBuildCommand` runs `cargo build --release`, not debug. |

**Nothing found in category:** "Stored data" — verified by `grep`-ing `Cargo.toml` for `rusqlite` (absent) and confirming `persistence/session_file.rs` only does JSON file I/O.

## Common Pitfalls

### Pitfall 1: `externalBin` requires the target-triple-suffixed file at bundle time
**What goes wrong:** Adding `"externalBin": ["binaries/scrysynth-visual"]` and running `tauri build` fails with a "binary not found" error because Tauri looks for `binaries/scrysynth-visual-aarch64-apple-darwin`, not `binaries/scrysynth-visual`.
**Why it happens:** Tauri 2 expects the developer (or a `beforeBuildCommand` helper script) to produce the suffixed name; the bundler does not append the suffix itself.
**How to avoid:** Add a `beforeBuildCommand` that runs `cargo build --release --bin scrysynth-visual` and copies the output to `src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)`.
**Warning signs:** `tauri build` fails with a "missing external binary" error referencing an unsuffixed path.

### Pitfall 2: Cross-architecture builds break if you only build the host arch
**What goes wrong:** Building on an M-series Mac produces only `aarch64-apple-darwin`; Intel Mac users cannot run the `.app`.
**Why it happens:** `rustc --print host-tuple` returns the host, not the target. `cargo build --target x86_64-apple-darwin` would be needed for Intel.
**How to avoid:** For Phase 11 v1, explicitly scope the release to Apple Silicon (`aarch64-apple-darwin`) and document this in release notes. Universal binaries (`lipo` of `aarch64` + `x86_64`) are a follow-on; they require either two `cargo build --target` invocations or a `universal-apple-darwin` target. Do not promise Intel support in v1 release notes.
**Warning signs:** An Intel user reports the `.app` will not launch.

### Pitfall 3: Ad-hoc signed apps still trigger Gatekeeper on user machines
**What goes wrong:** Even with `signingIdentity: "-"`, macOS Gatekeeper shows "Scrysynth cannot be opened because Apple cannot check it for malicious software" when the user double-clicks the `.app` from a downloaded `.dmg`.
**Why it happens:** Ad-hoc signing makes the binary launchable on **your** machine but does not satisfy Gatekeeper on **other** machines because there is no Developer ID chain.
**How to avoid:** Document the right-click → Open → "Open Anyway" workflow in release notes, OR push full Developer ID signing + notarization + stapling before distribution. The Tauri docs are explicit about this tradeoff.
**Warning signs:** UAT passes on the dev machine but early users report Gatekeeper blocks.

### Pitfall 4: Stripped release sidecar can no longer panic with useful traces
**What goes wrong:** After `strip = true` in the release profile, panic reports from `scrysynth-visual` become useless (`panic at '...'`).
**Why it happens:** Aggressive stripping removes symbol names.
**How to avoid:** Keep debug symbols for the release build (`profile.release.debug = "line-tables-only"` or similar) OR ensure sidecar panics are surfaced as typed protocol messages through the existing JSON-lines `error_response` path before the process exits.
**Warning signs:** Sidecar crashes during UAT and the only signal is a generic non-zero exit code.

### Pitfall 5: README drift — claiming "release-ready" prematurely
**What goes wrong:** After UAT passes, the README is rewritten to drop the "foundation prototype" language, but downstream consumers interpret this as "production-grade instrument" and file bugs against deferred-v1 work (richer Bevy rendering, live LLM provider, physical-controller GUI polish).
**Why it happens:** "v1 release" and "production-complete" are different bars.
**How to avoid:** Use explicit "Supported in v1" and "Not supported in v1 (deferred)" matrices in both README and RELEASE_NOTES. The SUPPORTED list = what Phases 7-10 actually verified + what Phase 11 packaging delivers. The NOT SUPPORTED list = live provider-backed agent, richer Bevy rendering, physical-controller click-through UAT, cross-platform.
**Warning signs:** User reports an issue against a deferred-v1 feature as if it were a regression.

### Pitfall 6: Bundled sidecar inherits the wrong `SCRYSYNTH_VISUAL_MODE`
**What goes wrong:** The `BevySidecarAdapter` reads `SCRYSYNTH_VISUAL_MODE` to decide between minimal and visible Bevy runtime. In a packaged `.app`, the inherited env may differ from dev.
**Why it happens:** Env propagation through `tauri-plugin-shell`'s `spawn()` follows the parent process env.
**How to avoid:** Make the sidecar mode explicit via the capability `args` list (e.g., always pass `--minimal` for v1, since the visible Bevy window is the documented default and the minimal mode is the test/dev path). Alternatively, set `SCRYSYNTH_VISUAL_MODE` explicitly in the `Command::env()` call inside `BevySidecarAdapter`.

### Pitfall 7: `npm run tauri build` re-runs `write_generated_typescript_contract()` on a clean checkout and fails
**What goes wrong:** `lib.rs::run()` calls `write_generated_typescript_contract()` and exits non-zero on failure. On a fresh release machine without the right permissions, this can block the build.
**Why it happens:** The contract-writer writes into `src/generated/` which may be read-only in CI.
**How to avoid:** Verify the release build path includes a writable `src/generated/` directory; the existing dev workflow already does this, so it should "just work", but flag it in the UAT plan as a launch-time smoke check.

## Code Examples

### Current scsynth discovery (verified, keep, polish message)

```rust
// Source: src-tauri/src/audio/supercollider.rs:459-477  [VERIFIED: codebase]
fn resolve_scsynth_executable() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os(SCSYNTH_OVERRIDE_ENV) {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    env::var_os("PATH")
        .and_then(|path_var| {
            env::split_paths(&path_var)
                .map(|entry| entry.join(SCSYNTH_BIN))
                .find(|candidate| is_executable(candidate))
        })
        .or_else(|| {
            let app_bundle_path = PathBuf::from(MACOS_APP_BUNDLE_SCSYNTH);
            is_executable(&app_bundle_path).then_some(app_bundle_path)
        })
}
```

### Current sidecar discovery (verified, REPLACE for bundled path)

```rust
// Source: src-tauri/src/visual/bevy_sidecar.rs:253-266  [VERIFIED: codebase]
fn resolve_bevy_executable() -> Option<PathBuf> {
    if let Some(override_path) = env::var_os(BEVY_OVERRIDE_ENV) {
        let path = PathBuf::from(override_path);
        if path.is_file() {
            return Some(path);
        }
    }

    env::var_os("PATH").and_then(|path_var| {
        env::split_paths(&path_var)
            .map(|entry| entry.join(BEVY_BIN))
            .find(|candidate| is_executable(candidate))
    })
}
```

**Note:** Unlike `resolve_scsynth_executable`, the sidecar resolver has **no macOS bundle fallback**. Phase 11 must add either (a) a bundled-binary path via `tauri-plugin-shell`, or (b) a manual fallback that probes `current_exe().parent()/scrysynth-visual` and `current_exe().parent()/../Resources/scrysynth-visual`. Option (a) is recommended.

### Current synthdef resource resolution (verified, keep)

```rust
// Source: src-tauri/src/audio/supercollider.rs:436-457  [VERIFIED: codebase]
fn resolve_resource_path(relative_path: &str) -> PathBuf {
    let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    if dev_path.exists() {
        return dev_path;
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let local_path = exe_dir.join(relative_path);
            if local_path.exists() {
                return local_path;
            }

            let macos_resource_path = exe_dir.join("../Resources").join(relative_path);
            if macos_resource_path.exists() {
                return macos_resource_path;
            }
        }
    }

    dev_path
}
```

This already handles dev (`CARGO_MANIFEST_DIR`), local-relative (`current_exe`), and macOS-bundle (`../Resources/`) layouts. **No Phase 11 change needed.**

### Current tauri.conf.json (verbatim)

```jsonc
// Source: src-tauri/tauri.conf.json  [VERIFIED: codebase]
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "scrysynth",
  "version": "0.1.0",
  "identifier": "com.lem.scrysynth",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://127.0.0.1:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [ { "title": "scrysynth", "width": 800, "height": 600 } ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "resources": [ "resources/synthdefs" ],
    "icon": [ "icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png",
              "icons/icon.icns", "icons/icon.ico" ]
  }
}
```

**Gaps for v1:**
- No `bundle.macOS` block → no `minimumSystemVersion`, no `signingIdentity`.
- No `bundle.externalBin` → sidecar cannot be bundled through the official mechanism.
- `bundle.targets: "all"` → produces updater bundles that need a signing key we do not have.
- `version: "0.1.0"` → should bump to `1.0.0` for the v1 release.
- No `bundle.copyright`, no `bundle.longDescription`, no `bundle.shortDescription`.
- No `app.windows[0].fileDropEnabled`, transparency, etc. — but these are not blocking for v1.

### Current capabilities/default.json (verbatim)

```jsonc
// Source: src-tauri/capabilities/default.json  [VERIFIED: codebase]
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [ "core:default", "opener:default", "dialog:default" ]
}
```

**Gap:** No `shell:allow-spawn` for the sidecar. Required if `tauri-plugin-shell` is adopted.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri 1.x `sidecar` array on root config | Tauri 2.x `bundle.externalBin` + `tauri-plugin-shell` + capability `shell:allow-spawn/execute` | Tauri 2.0 GA (2024) | Phase 11 must use the Tauri 2 pattern; old Tauri 1 examples online will mislead. |
| `--print host-tuple` unavailable | `rustc --print host-tuple` (Rust 1.84+) | Rust 1.84 (Dec 2024) | Host-tuple probe is a one-liner; no need to parse `rustc -Vv`. Current toolchain (1.96.0) supports it. |
| `notarytool` with Apple ID password | `notarytool` with App Store Connect API key OR Apple ID + app-specific password + team ID | Apple deprecated the old altool path in 2023 | Tauri 2 wraps `notarytool` correctly; just provide the right env vars. |
| Ad-hoc signing unsupported for shipping | Ad-hoc signing (`-`) explicitly supported by Tauri 2 docs as an intermediate state for Apple Silicon | Tauri 2 docs updated May 2026 | Valid Phase 11 v1 strategy; users right-click → Open. |

**Deprecated/outdated:**
- `codesign --force --deep`: deprecated by Apple; Tauri does its own layer-by-layer signing. Never invoke raw.
- `productbuild` for the `.app`: Tauri 2's bundler handles this.
- Tauri 1 examples showing `"tauri": { "sidecars": [...] }` — wrong for Tauri 2.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `tauri-plugin-shell` Rust crate is published and current on crates.io under that exact name | Standard Stack | Planner cannot `cargo add tauri-plugin-shell@2`; mitigated by official Tauri sidecar docs importing it directly. |
| A2 | Release-stripped `scrysynth-visual` binary is in the 10-30 MB range | Don't Hand-Roll (Bundling a debug sidecar) | Bundle size budget may need to expand; mitigated by measuring during UAT. |
| A3 | Bare `scsynth` binary lifted out of SuperCollider.app will not produce sound for non-built-in UGens | Standard Stack (Alternatives) | If wrong, scsynth could be bundled as a resource; current recommendation to require SC install remains conservative and safe. |
| A4 | Tauri 2's `externalBin` does not place binaries under `resource_dir()` | Anti-Patterns | If wrong, manual resolution would work and the plugin refactor could be skipped; this is well-established Tauri behavior and very unlikely to be wrong. |
| A5 | The Phase 10 deterministic/mock planner path satisfies REL-02's "agent approval" scenario without a live LLM provider | UAT Strategy | If wrong, Phase 11 would block on live provider integration, which is documented as out of scope; the safer reading is that "agent approval" maps to the verified approval/rejection flow, not to provider intelligence. |
| A6 | `productName: "scrysynth"` (lowercase) is acceptable for v1 | Code Examples | Cosmetic; macOS will display "scrysynth" in the menu bar. May want to capitalize to "Scrysynth" for polish, but not blocking. |

**Planner action:** Each `[ASSUMED]` row should be confirmed in `discuss-phase` or resolved by a `checkpoint:human-verify` task before execution.

## Open Questions

1. **Should Phase 11 bump the version to `1.0.0`?**
   - What we know: `tauri.conf.json` and `Cargo.toml` are both at `0.1.0`. The milestone is named "v1 runtime hardening".
   - What's unclear: Whether the user wants the first release to be `1.0.0`, `0.1.0` (semver-stable foundation), or `1.0.0-rc1`.
   - Recommendation: Default to `1.0.0` for the release build; surface as a `discuss-phase` question.

2. **Apple Silicon only, or universal binary for v1?**
   - What we know: Dev machine is `aarch64-apple-darwin`. Universal-binary builds add CI complexity.
   - What's unclear: Whether Intel Mac users are in scope for the first release.
   - Recommendation: Apple Silicon only for Phase 11; document Intel support as deferred. Surface as `discuss-phase` question.

3. **Should the visible Bevy window or the minimal sidecar be the v1 default in the packaged app?**
   - What we know: The current default (when `SCRYSYNTH_VISUAL_MODE` is unset) opens a visible Bevy window via `run_visible_runtime()`. Phase 8 UAT verified both paths.
   - What's unclear: Whether the visible-window path is appropriate for a release whose visuals are intentionally minimal.
   - Recommendation: Keep the visible window (it is verified and matches "audiovisual instrument" product framing) but document the minimal mode as a runtime setting. No code change beyond explicit env/arg passing.

4. **Does the user already have an Apple Developer Program membership?**
   - What we know: Tauri config has no signing identity configured. No `APPLE_*` env vars visible in repo.
   - What's unclear: Whether full notarization is achievable within Phase 11 or must be a documented follow-on.
   - Recommendation: Default to ad-hoc signing for Phase 11 (`signingIdentity: "-"`) and document the upgrade path. If the user confirms a Developer ID Application cert is available, escalate signing to a Phase 11 stretch goal.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Building `scrysynth` + `scrysynth-visual` | ✓ | rustc 1.96.0 | — |
| `cargo` | Same | ✓ | cargo 1.96.0 | — |
| Node.js | Frontend build (`npm run build`) | ✓ | 22.22.3 | — |
| `npm` | Frontend deps install | ✓ | 10.9.8 | — |
| `@tauri-apps/cli` (npm) | `tauri build` | ✓ | `^2` declared; 2.11.3 latest | — |
| Xcode Command Line Tools / `xcrun` | macOS bundling, `notarytool`, `codesign` | ✓ | xcrun 72 | — |
| macOS SDK | `cargo build` for `aarch64-apple-darwin` | ✓ | macOS 26.5.1 | — |
| SuperCollider (with `scsynth`) | Audio runtime at UAT time | ✓ | 3.14.x at `/Applications/SuperCollider.app/...` | Document install path; UAT cannot verify audio without it |
| `tauri-plugin-shell` Rust crate | Idiomatic sidecar bundling | NOT INSTALLED yet | — | Manual `current_exe()`-based sidecar resolution (less robust) |
| Apple Developer ID Application cert + notarization credentials | Full signing + notarized distribution | ✗ (none configured) | — | Ad-hoc signing (`signingIdentity: "-"`) + documented right-click-Open workflow |
| Physical MIDI controller | REL-02 "hardware learn" click-through UAT | NOT VERIFIED in this environment | — | CoreMIDI virtual source (already verified Phase 9); physical-controller click-through is explicitly out of Phase 11 scope |
| Live LLM provider credentials | REL-02 "agent approval" with live intelligence | ✗ | — | Deterministic/mock planner (already verified Phase 10); live provider explicitly out of Phase 11 scope |

**Missing dependencies with no fallback:**
- None for the primary Phase 11 deliverable (ad-hoc signed `.app`/`.dmg`).

**Missing dependencies with fallback:**
- Full notarization → ad-hoc signing + right-click-Open documentation.
- Physical MIDI hardware → CoreMIDI virtual source (Phase 9 path).
- Live LLM provider → deterministic/mock planner (Phase 10 path).

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local single-user app; no auth surface. |
| V3 Session Management | no | Local single-user app; no sessions. |
| V4 Access Control | yes (capability scope) | Tauri capability system: `default.json` must scope `shell:allow-spawn` to exactly the sidecar binary name and args; do NOT use `shell:default` or allow-all. |
| V5 Input Validation | yes (existing) | `zod` validates session JSON import; `serde` validates Rust command payloads. No Phase 11 change. |
| V6 Cryptography | yes (signing) | macOS code signing uses Apple's codesign + notarytool; never hand-roll signing. Ad-hoc identity (`-`) is acceptable for Phase 11; full signing is a follow-on. |
| V8 Data Protection | yes (at rest) | Sessions are user-owned JSON files; no app-managed secrets at rest. `APPLE_*` and `TAURA_SIGNING_PRIVATE_KEY` env vars must NOT ship in the bundle. |
| V14 Configuration / Deployment | yes | Hardened release config: explicit `bundle.targets` (not `"all"`), explicit `bundle.macOS.minimumSystemVersion`, scrubbed env at build time, no debug assertions in release profile. |

### Known Threat Patterns for Tauri 2 desktop release

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Sidecar binary tampering post-install | Tampering | Code-sign the sidecar alongside the main binary (Tauri does this when `signingIdentity` is set); bundle-side integrity falls out of `.app` bundle signature. Ad-hoc signing provides weaker but non-zero integrity. |
| Sidecar privilege escalation via loose capability | Elevation of Privilege | Scope `shell:allow-spawn` capability to the exact sidecar name + exact args (`"args": ["--minimal"]`); never use `shell:allow-execute` with `"args": true`. |
| Bundled secrets leaked via `strings` | Information Disclosure | Build script must scrub env (`APPLE_*`, `TAURA_SIGNING_PRIVATE_KEY`) before invoking `tauri build`; verify with `strings Scrysynth.app/Contents/MacOS/scrysynth | grep APPLE`. |
| DYLD injection on launch (macOS) | Tampering | `bundle.macOS.signingIdentity` (even ad-hoc) + `hardenedRuntime` entitlement if escalating to full signing. |
| User opens untrusted session JSON | Tampering | Already mitigated by `zod` runtime validation on the import path (existing). |
| Unsigned `.dmg` prompts user to disable Gatekeeper | Spoofing (social) | Mitigated by either (a) full notarization for v1 or (b) explicit release-note instructions + right-click-Open documentation for ad-hoc v1. |

## Sources

### Primary (HIGH confidence)
- **Tauri 2 official docs — "Embedding External Binaries"** at https://v2.tauri.app/develop/sidecar/ (fetched 2026-06-23; "Last updated: Jun 15, 2026"). Confirms `externalBin` + target-triple suffix + `tauri_plugin_shell::ShellExt::sidecar()` + capability `shell:allow-spawn/execute` + capability `args` validator pattern.
- **Tauri 2 official docs — "macOS Code Signing"** at https://v2.tauri.app/distribute/sign/macos/ (fetched 2026-06-23; "Last updated: May 17, 2026"). Confirms `bundle.macOS.signingIdentity` config, `APPLE_SIGNING_IDENTITY` env, `APPLE_CERTIFICATE` + `APPLE_CERTIFICATE_PASSWORD` for CI, notarization via `APPLE_ID`+`APPLE_PASSWORD`+`APPLE_TEAM_ID` or App Store Connect API (`APPLE_API_ISSUER`+`APPLE_API_KEY`+`APPLE_API_KEY_PATH`), `--skip-stapling` flag, ad-hoc signing (`"signingIdentity": "-"`).
- **Tauri 2 official docs — "Embedding Additional Files" (resources)** at https://v2.tauri.app/develop/resources/ (fetched 2026-06-23; "Last updated: Feb 18, 2026"). Confirms `bundle.resources` semantics, `$RESOURCE` resolution, `PathResolver::resolve(..., BaseDirectory::Resource)`.
- **Codebase** — `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`, `src-tauri/src/lib.rs`, `src-tauri/src/audio/supercollider.rs`, `src-tauri/src/visual/bevy_sidecar.rs`, `src-tauri/src/bin/scrysynth-visual.rs`. All read directly during research.

### Secondary (MEDIUM confidence)
- **STACK.md** (project) — locks Tauri 2.10.x, SuperCollider 3.14.1, Bevy 0.18.x, and explicitly states "Use Tauri sidecar support + Rust process supervision" for SC and "Tauri has first-class sidecar support" for the visual sidecar. This is the project-level decision basis.
- **Phase 7-10 UAT evidence files** — `.planning/phases/07-…-07-UAT.md`, `08-…-05-UAT.md`, `09-…-06-UAT.md`, `10-…-06-UAT.md`. These ground every "verified" claim about what runs today.

### Tertiary (LOW confidence)
- Live `notarytool` API behavior in macOS 26.x — based on Tauri docs and Apple's documented `notarytool` interface; not independently verified in this session. Tauri wraps this correctly per official docs.
- Release-stripped `scrysynth-visual` binary size estimate (10-30 MB) — based on Bevy/wgpu release-build norms; not measured in this session.

## Metadata

**Confidence breakdown:**
- Tauri 2 packaging/sidecar/signing claims: **HIGH** — grounded in official Tauri 2 docs fetched 2026-06-23 plus direct codebase inspection.
- In-repo current state (config gaps, capability gaps, plugin gaps): **HIGH** — verified by reading `tauri.conf.json`, `Cargo.toml`, `capabilities/default.json`, `lib.rs`, and the sidecar/scsynth resolver source.
- UAT coverage map (what is already verified vs. what Phase 11 must re-verify): **HIGH** — grounded in the four Phase 7-10 UAT evidence files plus the existing test files inventory.
- Scope-fence recommendations (macOS-first, deferred items): **MEDIUM** — derived from project constraints + STATE.md + ROADMAP.md; the user may disagree on individual scope boundaries.
- Release-strip binary size, notarization cost/complexity: **MEDIUM** — based on Tauri docs and standard Apple/Bevy release-build norms; not independently measured.

**Research date:** 2026-06-23
**Valid until:** 2026-07-23 (30 days; stable Tauri 2 docs and slow-moving Apple tooling. Re-verify if Tauri 2.11+ ships breaking config-schema changes or if Apple changes notarization requirements.)

## RESEARCH COMPLETE
