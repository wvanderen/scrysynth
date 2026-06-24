---
phase: 11-release-readiness
plan: 01
name: Release Packaging Build & Visual Sidecar Wiring
type: hardening
status: planned
created: 2026-06-23
wave: 1
depends_on: []
td_epic: td-TBD
files_modified:
  - src-tauri/tauri.conf.json
  - src-tauri/Cargo.toml
  - src-tauri/capabilities/default.json
  - src-tauri/src/lib.rs
  - src-tauri/src/visual/bevy_sidecar.rs
  - src-tauri/src/audio/supercollider.rs
  - src-tauri/src/application/session_store.rs
  - package.json
  - scripts/prepare-sidecar.sh
  - .gitignore
autonomous: false
requirements: [REL-01]
user_setup: []

must_haves:
  truths:
    - "REL-01 / ROADMAP #1: `npm run tauri build` produces a launchable `scrysynth.app` and `.dmg` for the aarch64-apple-darwin host target."
    - "D-06: The packaged `.app` carries the visual sidecar binary next to the main executable under the target-triple-suffixed name, placed there by the `beforeBuildCommand` helper — not by hand and not via `bundle.resources`."
    - "D-06: `BevySidecarAdapter` launches the bundled sidecar through `tauri_plugin_shell::ShellExt::sidecar()` and never through a raw `std::process::Command::new` against a resolved path."
    - "D-02: The release build targets Apple Silicon only; no updater/artifact target that requires an unavailable signing key is emitted."
    - "D-03: The bundle is ad-hoc signed (`signingIdentity: \"-\"`); full Developer ID notarization is deferred and documented."
    - "ROADMAP #2: When `scsynth` is absent, the Runtime Health panel shows an actionable message naming the SuperCollider install requirement, the `SCRYSYNTH_SCSYNTH_PATH` override, and the macOS bundle fallback path."
    - "ROADMAP #2: When the bundled visual sidecar is unavailable in a packaged app, the Runtime Health panel shows a bundled-sidecar-aware message (not the dev-only `SCRYSYNTH_BEVY_PATH` instruction)."
    - "V4 Access Control: the `shell:allow-spawn` capability is scoped to exactly `binaries/scrysynth-visual` with arg `--minimal`; no `shell:default` or allow-all entry exists."
  artifacts:
    - path: "src-tauri/tauri.conf.json"
      provides: "v1 bundle config: version 1.0.0, copyright, macOS ad-hoc signing block, externalBin for visual sidecar, narrowed targets"
      contains: "externalBin"
    - path: "src-tauri/capabilities/default.json"
      provides: "Scoped shell:allow-spawn permission for the visual sidecar binary"
      contains: "shell:allow-spawn"
    - path: "scripts/prepare-sidecar.sh"
      provides: "Build helper that produces the target-triple-suffixed release sidecar binary for Tauri's externalBin step"
      contains: "rustc --print host-tuple"
    - path: "src-tauri/src/visual/bevy_sidecar.rs"
      provides: "BevySidecarAdapter refactored onto the shell-plugin sidecar API with packaged-app-aware missing-binary messaging"
      contains: "sidecar"
    - path: "src-tauri/src/audio/supercollider.rs"
      provides: "Polished missing-scsynth message suitable for a packaged app end user"
      contains: "missing_scsynth_message"
    - path: "src-tauri/src/lib.rs"
      provides: "tauri_plugin_shell registration and AppHandle threading into the session store / visual adapter"
      contains: "tauri_plugin_shell"
  key_links:
    - from: "src-tauri/tauri.conf.json (bundle.externalBin + build.beforeBuildCommand)"
      to: "scripts/prepare-sidecar.sh -> src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin"
      via: "beforeBuildCommand invokes the helper, which builds + copies the suffixed binary Tauri's bundler expects"
      pattern: "prepare-sidecar"
    - from: "src-tauri/capabilities/default.json (shell:allow-spawn allow-list)"
      to: "src-tauri/src/visual/bevy_sidecar.rs (app.shell().sidecar(\"scrysynth-visual\"))"
      via: "capability must permit the bare sidecar name and the exact --minimal arg the adapter passes"
      pattern: "binaries/scrysynth-visual"
    - from: "src-tauri/src/lib.rs (plugin registration + AppHandle)"
      to: "src-tauri/src/application/session_store.rs -> BevySidecarAdapter"
      via: "AppHandle flows from the Tauri builder into the adapter so ShellExt::sidecar() can resolve the bundled binary"
      pattern: "tauri_plugin_shell::init"
---

<objective>
Make `npm run tauri build` produce a launchable, ad-hoc-signed Apple Silicon `scrysynth.app`/`.dmg` that bundles the minimal `scrysynth-visual` sidecar through Tauri's official `externalBin` + `tauri-plugin-shell` mechanism, keeps `scsynth` as an external user-installed runtime discovered by the existing path chain, and polishes missing-binary diagnostics so a packaged-app end user can recover without reading source.

Purpose: This plan delivers the REL-01 packaging substrate and the ROADMAP #2 diagnostic-polish prerequisite. Plan 02 then runs the actual release build smoke, the consolidated manual UAT, and rewrites the docs against verified behavior. Phase 11 introduces zero new app features — every primitive (Runtime Health panel, sidecar lifecycle, panic/restart, error projection) already exists and is verified from Phases 7-10; this plan is build configuration, one supervised-process refactor, and message strings.

Output: Updated `tauri.conf.json`, `Cargo.toml`, `package.json`, `capabilities/default.json`, `lib.rs`; new `scripts/prepare-sidecar.sh`; refactored `BevySidecarAdapter`; polished `missing_scsynth_message` and visual missing-sidecar message; `.gitignore` entry for the generated binaries directory.

Locked decisions encoded (do not re-litigate): D-01 version 1.0.0 + copyright; D-02 Apple Silicon only; D-03 ad-hoc signing only; D-04 minimal GPU-free visual sidecar as the packaged default; D-05 scsynth required as user install + existing discovery kept (no SC.app bundling); D-06 visual sidecar bundled via `externalBin` + `tauri-plugin-shell` with a `beforeBuildCommand` helper and `BevySidecarAdapter` refactored off raw `std::process::Command`.
</objective>

<execution_context>
@/Users/eggfam/.config/opencode/gsd-core/workflows/execute-plan.md
@/Users/eggfam/.config/opencode/gsd-core/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/11-release-readiness/11-RESEARCH.md

# Grounding source (read the cited line ranges before editing)
@src-tauri/tauri.conf.json
@src-tauri/Cargo.toml
@src-tauri/capabilities/default.json
@src-tauri/src/lib.rs
@src-tauri/src/visual/bevy_sidecar.rs
@src-tauri/src/audio/supercollider.rs
@src-tauri/src/application/session_store.rs
@package.json
</context>

<tasks>

<task type="checkpoint:human-verify" gate="blocking-human">
  <name>Task 1: Package-legitimacy gate for tauri-plugin-shell (T-11-SC, [ASSUMED] crate)</name>
  <files>src-tauri/Cargo.toml</files>
  <read_first>
    - .planning/phases/11-release-readiness/11-RESEARCH.md §"Package Legitimacy Audit" (rows for tauri-plugin-shell marked [ASSUMED]) and §"Standard Stack" (Pattern 1 citations to https://v2.tauri.app/develop/sidecar/)
  </read_first>
  <action>
    Hold as a blocking gate before Task 2 runs `cargo add tauri-plugin-shell@2`. Do NOT mutate any file in this task. The research audit tagged the Rust crate `tauri-plugin-shell` as [ASSUMED] (crates.io legitimacy was not run through the npm seam); per the package-legitimacy protocol an [ASSUMED] package requires a blocking human verification before it is added, and legitimacy checkpoints are never auto-approvable regardless of `workflow.auto_advance`. Operator confirms the crate against crates.io and the official Tauri 2 sidecar docs in the how-to-verify steps below; only an explicit approval unblocks Task 2.
  </action>
  <what-built>
    Nothing yet. This is a pre-install legitimacy gate required before adding the only new Phase 11 dependency. The research audit tagged the Rust crate `tauri-plugin-shell` as [ASSUMED] because crates.io legitimacy was not run through the npm seam; per the package-legitimacy protocol, an [ASSUMED] package requires a blocking human verification before `cargo add`. Legitimacy checkpoints are never auto-approvable.
  </what-built>
  <how-to-verify>
    Before Task 2 runs `cargo add tauri-plugin-shell@2`, confirm the crate is the official Tauri one:
    1. Open https://crates.io/crates/tauri-plugin-shell and confirm the publisher links to the `tauri-apps/plugins-workspace` repository and the latest publish is a Tauri 2 GA release (version `2.x`).
    2. Open https://v2.tauri.app/develop/sidecar/ and confirm the official Tauri 2 sidecar docs import `tauri_plugin_shell::ShellExt` and call `app.shell().sidecar(...)`.
    3. Cross-check that the crate name spelled exactly `tauri-plugin-shell` (hyphenated) resolves on crates.io and is not a typosquat.
    If all three hold, approve. If anything is off, halt and surface the concern before any code change.
  </how-to-verify>
  <resume-signal>Type "approved" to proceed to Task 2, or describe the legitimacy concern.</resume-signal>
  <verify>
    <human-check>Operator confirms crates.io publisher = tauri-apps/plugins-workspace, version line = 2.x, and official sidecar docs import the crate under this exact name.</human-check>
  </verify>
  <acceptance_criteria>
    - `tauri-plugin-shell` confirmed as the official Tauri 2 crate on crates.io before it is added to Cargo.toml.
    - No [SLOP] or unresolved [SUS] package reaches `Cargo.toml`.
  </acceptance_criteria>
  <done>Gate recorded as approved (or a concern surfaced and resolved) before Task 2 mutates Cargo.toml.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: Tauri bundle config, visual sidecar build pipeline, plugin registration, and capability scoping</name>
  <files>
    src-tauri/tauri.conf.json,
    src-tauri/Cargo.toml,
    package.json,
    src-tauri/capabilities/default.json,
    scripts/prepare-sidecar.sh,
    .gitignore,
    src-tauri/src/lib.rs
  </files>
  <read_first>
    - src-tauri/tauri.conf.json (current full file — version 0.1.0, no bundle.macOS, no externalBin, targets "all")
    - src-tauri/capabilities/default.json (current permissions list — no shell entry)
    - src-tauri/Cargo.toml (current deps — no tauri-plugin-shell; package version 0.1.0)
    - package.json (version 0.1.0)
    - src-tauri/src/lib.rs lines 306-352 (the `run()` builder — plugins registered at 316-317)
    - .planning/phases/11-release-readiness/11-RESEARCH.md §"Code Examples > Current tauri.conf.json", §"Code Examples > Current capabilities/default.json", §"Pattern 1: Same-crate Rust binary as Tauri sidecar", §"Common Pitfalls" (Pitfalls 1, 2, 4, 6), §"Anti-Patterns to Avoid"
    - .gitignore (current — node_modules, dist, .DS_Store already ignored)
  </read_first>
  <action>
    Apply the v1 release bundle configuration and the visual sidecar build pipeline. Implement D-01, D-02, D-03, D-04, and the config/plugin half of D-06.

    On `src-tauri/tauri.conf.json`:
    - Bump top-level `version` from `0.1.0` to `1.0.0` (per D-01).
    - Add `bundle.copyright` with a v1 copyright string (e.g. "© 2026 Scrysynth contributors").
    - Change `bundle.targets` from the string `"all"` to the array `["app", "dmg"]` so the bundler does not emit updater artifacts that require a signing key we do not have (per D-02 and Pitfall: "treating bundle.targets: all as fine for v1").
    - Add a `bundle.macOS` object with `minimumSystemVersion` set to `"11.0"` and `signingIdentity` set to the ad-hoc identity string `"-"` (per D-03). Do NOT set `APPLE_*` env-derived fields.
    - Add `bundle.externalBin` as an array containing the bare logical path `"binaries/scrysynth-visual"` (per D-06 and Pattern 1). Tauri will append the host target triple at bundle time; do NOT pre-suffix it here.
    - Update `build.beforeBuildCommand` from `npm run build` to `npm run build && ./scripts/prepare-sidecar.sh` so the suffixed release sidecar binary exists before the bundler runs (per D-06).
    - Leave `bundle.resources` (synthdefs), `bundle.icon`, `identifier`, `productName`, `app.windows`, and `app.security.csp` unchanged.

    On `package.json`: bump `version` from `0.1.0` to `1.0.0` to match the Tauri config (per D-01). Do not change scripts or dependencies here.

    On `src-tauri/Cargo.toml`: bump `[package]` `version` from `0.1.0` to `1.0.0` (per D-01). Add `tauri-plugin-shell = "2"` to `[dependencies]` (per D-06; legitimacy confirmed in Task 1). Leave `bevy`, `tauri`, and all other deps unchanged. Do not change `default-run` or the `[lib]` block.

    On `src-tauri/capabilities/default.json`: add a scoped shell permission object to the `permissions` array. The object's `identifier` is `shell:allow-spawn`, and its `allow` array contains exactly one entry with `name` set to `"binaries/scrysynth-visual"`, `sidecar` set to `true`, and `args` set to the array `["--minimal"]`. Do NOT use `shell:default`, do NOT use `shell:allow-execute`, and do NOT set `args: true` anywhere — this is the V4 Access Control mitigation for T-11-02 (Elevation of Privilege). Keep the existing `core:default`, `opener:default`, `dialog:default` entries.

    Create `scripts/prepare-sidecar.sh` as an executable bash script (chmod +x) following Pattern 1 from the research: `set -euo pipefail`; read the host target via `TARGET="$(rustc --print host-tuple)"`; ensure `src-tauri/binaries` exists; run `cargo build --release --manifest-path src-tauri/Cargo.toml --bin scrysynth-visual`; copy the resulting `src-tauri/target/release/scrysynth-visual` to `src-tauri/binaries/scrysynth-visual-${TARGET}`. The suffixed name is mandatory — a bare copy will silently fail the bundler (Pitfall 1). This script must build the release profile, never debug (the debug blob is ~272 MB; Pitfall 4 / "Bundling a debug sidecar").

    On `.gitignore`: add `src-tauri/binaries/` so the generated suffixed sidecar binaries are not committed. Place it near the existing build-artifact ignores.

    On `src-tauri/src/lib.rs` `run()` (around lines 314-317): register the new plugin by adding `.plugin(tauri_plugin_shell::init())` alongside the existing `tauri_plugin_opener::init()` and `tauri_plugin_dialog::init()` calls. Do NOT change the invoke_handler list or the generated-typescript-contract guard at the top of `run()`. Thread the `AppHandle` into `SessionStore` is done in Task 3 (this task only registers the plugin so the build compiles and the capability is satisfiable); if the compiler requires the `AppHandle` plumbing to land in the same commit to type-check, perform the minimal `tauri_plugin_shell::init()` registration only and leave the adapter refactor to Task 3.
  </action>
  <verify>
    <automated>cd src-tauri && cargo check --manifest-path Cargo.toml 2>&1 | tail -20</automated>
    <automated>./scripts/prepare-sidecar.sh && ls -la src-tauri/binaries/ && test -f "src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)"</automated>
    <automated>node -e "const c=require('./src-tauri/tauri.conf.json'); const ok = c.version==='1.0.0' && Array.isArray(c.bundle.targets) && c.bundle.targets.includes('app') && c.bundle.targets.includes('dmg') && c.bundle.macOS && c.bundle.macOS.signingIdentity==='-' && Array.isArray(c.bundle.externalBin) && c.bundle.externalBin.includes('binaries/scrysynth-visual') && /\(c\)|©|contributors/.test(c.bundle.copyright); console.log(ok?'CONF_OK':'CONF_FAIL'); process.exit(ok?0:1)"</automated>
    <automated>node -e "const c=require('./src-tauri/capabilities/default.json'); const perm=c.permissions.find(p=>typeof p==='object'&&p.identifier==='shell:allow-spawn'); const ok = perm && perm.allow.length===1 && perm.allow[0].name==='binaries/scrysynth-visual' && perm.allow[0].sidecar===true && JSON.stringify(perm.allow[0].args)===JSON.stringify(['--minimal']); console.log(ok?'CAP_OK':'CAP_FAIL'); process.exit(ok?0:1)"</automated>
    <automated>grep -q 'tauri-plugin-shell = "2"' src-tauri/Cargo.toml && grep -q '^version = "1.0.0"' src-tauri/Cargo.toml && grep -q '"version": "1.0.0"' package.json</automated>
    <automated>grep -q 'src-tauri/binaries/' .gitignore</automated>
  </verify>
  <acceptance_criteria>
    - `tauri.conf.json` version is 1.0.0, `bundle.targets` is the array `["app","dmg"]`, `bundle.macOS.signingIdentity` is `"-"`, `bundle.externalBin` lists `binaries/scrysynth-visual`, and `bundle.copyright` is set (D-01, D-02, D-03, D-06).
    - `beforeBuildCommand` chains `npm run build` and `./scripts/prepare-sidecar.sh`.
    - `Cargo.toml` and `package.json` versions are 1.0.0; `tauri-plugin-shell = "2"` is a dependency.
    - `capabilities/default.json` contains exactly one `shell:allow-spawn` entry scoped to `binaries/scrysynth-visual` with `sidecar: true` and `args: ["--minimal"]`, and no `shell:default` or allow-all entry.
    - `scripts/prepare-sidecar.sh` is executable and produces `src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)` from a release build.
    - `src-tauri/binaries/` is gitignored.
    - `lib.rs` registers `tauri_plugin_shell::init()` and `cargo check` passes.
  </acceptance_criteria>
  <done>All config and capability checks above return OK, the prepare-sidecar helper produces the correctly suffixed release binary, `cargo check` passes, and no `shell:default`/allow-all capability exists.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 3: Refactor BevySidecarAdapter onto the shell-plugin sidecar API and polish missing-binary diagnostics</name>
  <files>
    src-tauri/src/visual/bevy_sidecar.rs,
    src-tauri/src/audio/supercollider.rs,
    src-tauri/src/application/session_store.rs,
    src-tauri/src/lib.rs
  </files>
  <read_first>
    - src-tauri/src/visual/bevy_sidecar.rs lines 1-120 (struct, Default, with_executable_override*, start() that calls Command::new at line 92 and resolve_bevy_executable at line 82) and lines 253-282 (resolve_bevy_executable + missing_sidecar_message)
    - src-tauri/src/audio/supercollider.rs lines 14-16 (SCSYNTH_OVERRIDE_ENV, SCSYNTH_BIN, MACOS_APP_BUNDLE_SCSYNTH constants), lines 40-70 (start() failure paths that call missing_scsynth_message and the spawn-failure message), lines 419-481 (missing_scsynth_message, resolve_scsynth_executable, resolve_resource_path)
    - src-tauri/src/application/session_store.rs (how start_visual_runtime constructs BevySidecarAdapter today — Default::default vs with_executable_override; this is where AppHandle must be threaded in)
    - src-tauri/src/lib.rs lines 306-352 (run() builder; the managed Mutex<SessionStore> state and where AppHandle is available)
    - .planning/phases/11-release-readiness/11-RESEARCH.md §"Pattern 1" runtime-resolution Rust snippet, §"Code Examples > Current sidecar discovery", §"Common Pitfalls" (Pitfall 6: bundled sidecar inherits wrong SCRYSYNTH_VISUAL_MODE), §"Don't Hand-Roll" (resolve packaged sidecar path row)
  </read_first>
  <action>
    Refactor the supervised visual sidecar launch onto Tauri's official sidecar API and make both missing-binary messages actionable for a packaged-app end user. Implement D-05 (scsynth stays external, message polished) and the refactor half of D-06.

    AppHandle threading (lib.rs + session_store.rs): the `tauri_plugin_shell::ShellExt::sidecar()` method requires an `AppHandle` (or `App`). Today `BevySidecarAdapter` is constructed inside `SessionStore` without one. Thread the `AppHandle` from `lib.rs::run()` — where `tauri::Builder` provides it via the `.setup` hook or via `tauri::AppHandle` captured after `.build()` — into the managed `Mutex<SessionStore>` (or store it alongside the store) so `SessionStore::start_visual_runtime` can hand it to `BevySidecarAdapter`. Prefer the `.setup(|app| { ... })` hook: obtain `app.handle()`, clone it, fetch the managed `Mutex<SessionStore>`, and store the handle on the store (e.g. a new `Option<tauri::AppHandle>` field set via a `set_app_handle` method, or construct the store inside setup with the handle). Keep the `Mutex<SessionStore>` as the managed state; do not change the command signatures in the invoke_handler.

    BevySidecarAdapter refactor (bevy_sidecar.rs):
    - Add an `Option<tauri::AppHandle>` field to `BevySidecarAdapter` (and a setter or constructor parameter), defaulting to `None`. Keep `executable_override`/`executable_args` for the dev and test paths (`with_executable_override_and_args` is used by Phase 8 tests).
    - In `start()`, when an `AppHandle` is present AND no `executable_override` is set, launch via the sidecar API: build the command through `app_handle.shell().sidecar("scrysynth-visual")` (bare name, no path, no target-triple suffix — the plugin resolves the suffix), pass the arg `--minimal` explicitly so the packaged default is the GPU-free minimal runtime regardless of inherited env (per D-04 and Pitfall 6), then `.spawn()`. Wire the resulting child's stdin/stdout into the existing JSON-lines reader/writer (`spawn_response_reader`, `send_and_wait`) unchanged — only the spawn mechanism changes, not the protocol.
    - When an `AppHandle` is absent (unit tests, dev runs without the plugin) OR `executable_override` is set, keep the existing `Command::new` + `resolve_bevy_executable` path so Phase 8's `visual_sidecar_uat` integration test and `with_executable_override` callers continue to work without a real Tauri runtime. This dual path is intentional: production packaged app uses the sidecar API; tests use the override path.
    - Rewrite `missing_sidecar_message` so that when the bundled sidecar path was attempted (i.e. the sidecar API path), the message states that the bundled visual sidecar could not be launched and points the user to reinstalling Scrysynth (not to the dev-only `SCRYSYNTH_BEVY_PATH` env). When the dev/test override path was attempted, keep the existing `SCRYSYNTH_BEVY_PATH` guidance. Branch on which resolution path produced the failure.
    - Keep `SCRYSYNTH_BEVY_PATH` honored as a dev override (Pitfall 6): if set, prefer the override path even when an AppHandle is present, so developers can point at a debug build. Document this precedence in a code comment.

    scsynth message polish (supercollider.rs):
    - Polish `missing_scsynth_message()` (line 419) so a packaged-app end user with no Rust/cargo context can act on it. Keep the three resolution channels (install SuperCollider, `SCRYSYNTH_SCSYNTH_PATH` override, macOS bundle fallback) but phrase for an end user: name "SuperCollider" and the install step first, then the `SCRYSYNTH_SCSYNTH_PATH` environment variable as the override, then the macOS default install location. Do not change `resolve_scsynth_executable` logic (D-05: discovery chain stays as-is and is verified by Phase 7 UAT).
    - Likewise polish the spawn-failure message around line 65 so it reads as a setup instruction rather than an internal error, while keeping the `SCRYSYNTH_SCSYNTH_PATH` and macOS bundle fallback references.
    - Do NOT bundle `scsynth` or add any new resolution channel (per D-05 and research §"Alternatives" — bundling bare scsynth breaks non-built-in UGens).

    Keep the existing Phase 8 Rust tests (`adapter_reports_missing_configured_sidecar_with_setup_guidance` and the `visual_sidecar_uat` integration test) passing. If the dual-path refactor changes the missing-configured-sidecar test's expectations, update the test to assert the new bundled-sidecar-aware message wording while preserving the test's intent (a missing sidecar produces a setup-guidance message).
  </action>
  <verify>
    <automated>cargo test --manifest-path src-tauri/Cargo.toml --lib visual::bevy_sidecar 2>&1 | tail -30</automated>
    <automated>cargo test --manifest-path src-tauri/Cargo.toml --test visual_sidecar_uat 2>&1 | tail -30</automated>
    <automated>cargo test --manifest-path src-tauri/Cargo.toml --lib audio::supercollider 2>&1 | tail -20</automated>
    <automated>cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10</automated>
    <automated>rg -n 'std::process::Command' src-tauri/src/visual/bevy_sidecar.rs | rg -v 'test|override|dev' | wc -l | rg -q '^0$' && echo "NO_RAW_COMMAND_IN_PROD_PATH" || echo "WARN: raw Command remains — confirm it is only the dev/test path"</automated>
    <automated>rg -n 'sidecar\(' src-tauri/src/visual/bevy_sidecar.rs | rg -q 'shell' && echo "SIDECAR_API_WIRED" || echo "FAIL: shell sidecar API not used"</automated>
  </verify>
  <acceptance_criteria>
    - `BevySidecarAdapter::start()` uses `app_handle.shell().sidecar("scrysynth-visual")` with arg `--minimal` when an `AppHandle` is present and no override is set (D-06).
    - The dev/test override path (`with_executable_override_and_args`, no `AppHandle`) still uses `Command::new` so Phase 8 tests pass unchanged in intent.
    - `SCRYSYNTH_BEVY_PATH` remains honored as a dev override that takes precedence over the bundled sidecar.
    - `missing_sidecar_message` distinguishes the bundled-sidecar failure (end-user reinstall guidance) from the dev override failure (`SCRYSYNTH_BEVY_PATH` guidance).
    - `missing_scsynth_message` and the spawn-failure message read as end-user setup instructions naming SuperCollider install, `SCRYSYNTH_SCSYNTH_PATH`, and the macOS bundle fallback (D-05, ROADMAP #2).
    - `resolve_scsynth_executable` logic is unchanged.
    - All existing visual and audio Rust tests pass; the full `cargo test --manifest-path src-tauri/Cargo.toml` suite passes.
    - `cargo build` succeeds (the `AppHandle` threading type-checks end to end).
  </acceptance_criteria>
  <done>Sidecar launches through the shell-plugin API in the packaged path, the dev/test override path is preserved and green, both missing-binary messages are end-user-actionable, scsynth discovery is unchanged, and the full Rust test suite plus `cargo build` pass.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Tauri capability layer | The new `shell:allow-spawn` permission is the only thing allowing the app to spawn a child process; its scope is the trust boundary between the React/Rust app and the supervised sidecar binary. |
| Build host → bundled `.app` | The `beforeBuildCommand` helper and `cargo build --release` run on the build host and produce the binary that ships inside the `.app`; build-time env vars must not leak into the bundle. |
| Packaged `.app` → external `scsynth` | The app discovers and spawns a user-installed external binary; this is an inherent trust boundary (the user vouches for their own SuperCollider install). |
| End-user macOS host → ad-hoc-signed `.app` | Gatekeeper sits between a downloaded ad-hoc-signed `.app` and the user's first launch; this boundary is not fully mitigated in v1 (accept). |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-11-SC | Tampering | `cargo add tauri-plugin-shell` ([ASSUMED] crate) | mitigate | Blocking human checkpoint (Task 1) verifies the crate publisher is `tauri-apps/plugins-workspace` on crates.io and that official Tauri 2 sidecar docs import it under this exact name, before it reaches Cargo.toml. Legitimacy checkpoints are never auto-approvable. |
| T-11-01 | Tampering | Sidecar binary post-install | mitigate | Bundle is ad-hoc signed (`signingIdentity: "-"`) so the main `.app` and bundled sidecar carry a (weak) integrity signature; Tauri signs the sidecar alongside the main binary. Full Developer ID + notarization is the documented v1 follow-on (D-03). |
| T-11-02 | Elevation of Privilege | `shell:allow-spawn` capability | mitigate | Capability scoped to exactly `binaries/scrysynth-visual` with `sidecar: true` and `args: ["--minimal"]`. No `shell:default`, no `shell:allow-execute`, no `args: true`. Verified by an automated capability-shape check in Task 2. |
| T-11-03 | Information Disclosure | Build-time env (`APPLE_*`, `TAURA_SIGNING_PRIVATE_KEY`) baked into bundle | mitigate | Do not set `APPLE_*` or signing-key env in `tauri.conf.json`; ad-hoc identity needs none. Plan 02 Task 1 (build smoke) runs a `strings` grep over the packaged main binary to confirm no `APPLE_`/`TAURA_` secrets leaked. |
| T-11-04 | Spoofing (social) | Unsigned `.dmg` Gatekeeper prompt on end-user machines | accept | v1 ships ad-hoc signed only (D-03). Mitigation is documentation: RELEASE_NOTES.md (written in Plan 02) explains the right-click → Open → "Open Anyway" first-run workflow. Full notarization is an explicit v1 follow-on, not a Phase 11 deliverable. |
| T-11-05 | Tampering | DYLD injection / runtime tampering on launch | accept | Ad-hoc signing provides weaker integrity than hardened-runtime + Developer ID; accepted for v1 per D-03. Escalate to `hardenedRuntime` entitlement when full signing lands. |
</threat_model>

<verification>
Plan-level checks (before Plan 02 can build the release artifact):
- `cargo check --manifest-path src-tauri/Cargo.toml` passes with the new plugin + AppHandle plumbing.
- `cargo test --manifest-path src-tauri/Cargo.toml` (full suite) passes — visual sidecar tests, audio tests, and the `visual_sidecar_uat` integration test all green.
- `./scripts/prepare-sidecar.sh` produces `src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)` and it is a release-stripped binary (not the 272 MB debug blob).
- `npm run build` (frontend) still passes.
- Capability-shape and config-shape automated assertions (Task 2 verify block) all return OK.
- The actual `npm run tauri build` + ad-hoc sign + first-run right-click → Open smoke test is performed in Plan 02 Task 1 (it requires Plan 01's code to be complete first).
</verification>

<success_criteria>
- `tauri.conf.json`, `Cargo.toml`, and `package.json` are at version 1.0.0 with the macOS ad-hoc signing block, externalBin entry, narrowed targets, and copyright set (D-01, D-02, D-03, D-04, D-06).
- The `shell:allow-spawn` capability is tightly scoped to the visual sidecar (V4 / T-11-02 mitigated).
- `scripts/prepare-sidecar.sh` builds and places the correctly-suffixed release sidecar.
- `BevySidecarAdapter` launches the bundled sidecar via the shell-plugin API in the packaged path while preserving the dev/test override path (D-06).
- `scsynth` remains external with an end-user-actionable missing-binary message (D-05, ROADMAP #2).
- The full Rust test suite and frontend build pass.
- Plan 02 is unblocked to run the release build smoke and consolidated UAT.
</success_criteria>

<output>
Create `.planning/phases/11-release-readiness/11-release-readiness-01-SUMMARY.md` when done, recording: the final `tauri.conf.json` delta, the capability entry, the prepare-sidecar helper, the BevySidecarAdapter dual-path refactor, the polished messages, the test results, and any deviations from the locked decisions (there should be none). Plan 02 will reference this SUMMARY for the build smoke and UAT.
</output>

## Artifacts this phase (plan) produces or modifies

**Modified:**
- `src-tauri/tauri.conf.json` — version 1.0.0, `bundle.copyright`, `bundle.macOS` block (ad-hoc `signingIdentity`, `minimumSystemVersion`), `bundle.externalBin: ["binaries/scrysynth-visual"]`, `bundle.targets: ["app","dmg"]`, `build.beforeBuildCommand` chaining `prepare-sidecar.sh`.
- `src-tauri/Cargo.toml` — version 1.0.0, `+ tauri-plugin-shell = "2"`.
- `package.json` — version 1.0.0.
- `src-tauri/capabilities/default.json` — scoped `shell:allow-spawn` entry for `binaries/scrysynth-visual` (`sidecar: true`, `args: ["--minimal"]`).
- `src-tauri/src/lib.rs` — `tauri_plugin_shell::init()` registration; `AppHandle` threaded into `Mutex<SessionStore>` via the `.setup` hook.
- `src-tauri/src/visual/bevy_sidecar.rs` — `BevySidecarAdapter` gains an `Option<AppHandle>` field; `start()` uses `app_handle.shell().sidecar("scrysynth-visual").args(["--minimal"]).spawn()` in the packaged path; `missing_sidecar_message` branches on bundled vs dev-override failure; `SCRYSYNTH_BEVY_PATH` remains a dev override.
- `src-tauri/src/audio/supercollider.rs` — `missing_scsynth_message()` and the spawn-failure message rephrased for end-user recovery; `resolve_scsynth_executable` unchanged.
- `src-tauri/src/application/session_store.rs` — accepts/stores the `AppHandle` and passes it into `BevySidecarAdapter` construction.
- `.gitignore` — `src-tauri/binaries/` added.

**Created:**
- `scripts/prepare-sidecar.sh` — executable build helper producing `src-tauri/binaries/scrysynth-visual-$(rustc --print host-tuple)` from a release build.

**Generated (gitignored, not committed):**
- `src-tauri/binaries/scrysynth-visual-aarch64-apple-darwin` — the release sidecar binary consumed by Tauri's `externalBin` step.
