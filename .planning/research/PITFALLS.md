# Pitfalls Research — v2.0 "Studio-Grade Instrument" Integration

**Domain:** Adding new capabilities (graph UX rebuild, curated node library, visuals-behind-grid, focused shell, live LLM agent, cross-platform builds, notarization + auto-updater) to an EXISTING, shipping Tauri 2 / Rust / React 19 + @xyflow/react / SuperCollider desktop audiovisual instrument
**Researched:** 2026-06-26
**Confidence:** HIGH (grounded in the actual v1 codebase: `audio/synthdefs.rs`, `audio/compiler.rs`, `audio/runtime_manager.rs`, `application/agent_planner.rs`, `application/agent_command.rs`, `visual/bevy_sidecar.rs`, `visual/adapter.rs`, `components/session/GraphViewport.tsx`, `tauri.conf.json`, `Cargo.toml`; cross-checked against current Tauri 2.10, SuperCollider 3.14, and xyflow v12 official docs)
**Scope:** Pitfalls SPECIFIC to adding the seven v2 features to this existing system. v1-domain pitfalls (canonical-state-owns-truth, ownership at the control surface, async engine ops, IPC as realtime transport, over-coupling AV, etc.) are referenced by number from the prior `PITFALLS.md` (2026-04-11) where they resurface — not repeated.

---

## Critical Pitfalls

### Pitfall V2-1: Node library expansion trips three hardcoded v1 allowlists (synthdef names, parameter names, runtime targets)

**What goes wrong:**
The v1 audio compiler (`src-tauri/src/audio/synthdefs.rs`) and topology planner carry three independent allowlists that ALL reject unknown values at compile time:

1. **`synthdef_resource(name)` ends with `unreachable!("unknown v1 synthdef name")`** — adding a new node type without extending this match produces a **panic at runtime** the first time the compiler tries to plan resources for that node, taking down the audio boot path.
2. **`normalize_parameter_name(name)` returns `None`** for any parameter outside the 9-entry v1 allowlist (`level`, `frequency`, `wave_shape`, `noise_color`, `cutoff_hz`, `resonance`, `delay_time_s`, `feedback`, `mix`), which `apply_parameters` then surfaces as `ScResourcePlanError::UnsupportedParameter`. A curated node library entry with a parameter named e.g. `attack`, `decay`, `lfo_rate`, `pulse_width`, or `drive` is rejected at topology-compile time → runtime marked `Degraded` with a confusing "unsupported v1 SuperCollider parameter" message.
3. **`validate_runtime_target`** string-matches `runtime_target` against an explicit v1 set (`"audio/source/oscillator"`, `"audio/effect/low_pass_filter"`, `"audio/mixer/stereo"`, etc.). Any new node variant fails with `UnsupportedRuntimeTarget`.

**Why it happens:**
v1 was deliberately strict: the typed-against-SC-allowlist approach was correct for shipping a verified minimal instrument. The trap is that "add a node type" looks like a UI/palette task when it is actually a **four-layer compiler change** (domain enum → runtime_target string match → parameter normalization → synthdef resource map → actual `.scsyndef` file in `resources/synthdefs/v1/`).

**How to avoid:**
- Before writing any new node UI, audit and refactor the three allowlists into a single source of truth: a `NodeKindSpec` table (per kind: `synthdef_name`, `runtime_target`, `parameter_mapping: {domain_name → scsynth_control}`, `synthdef_path`). Drive `synthdef_resource`, `normalize_parameter_name`, and `validate_runtime_target` from this table.
- Replace the `unreachable!()` in `synthdef_resource` with a real `Err(ScResourcePlanError::UnsupportedSynthdef { ... })` variant — panics on the audio boot path are unacceptable for a live instrument.
- Add a **compiler conformance test** that, for every entry in the `NodeKindSpec` table, runs `compile_session_to_topology → plan_sc_resources` against a minimal session containing that node and asserts no `Unsupported*` errors and no panics. This test alone catches the entire pitfall class.
- Treat the `.scsyndef` files as code: each new spec entry requires a committed synthdef file with a unit test that boots a real `scsynth` and confirms the def loads with the declared control names (the v1 `audio_runtime.rs` integration test is the template).

**Warning signs:**
- A new node appears in the palette but audio goes to `Degraded` with "unsupported v1 SuperCollider parameter `<name>`" the first time you boot with it.
- `cargo test` passes but the packaged app panics in `synthdef_resource` the first time a new node is added to a session.
- Synthdef files are added to `resources/synthdefs/` without corresponding entries in all three allowlists.

**Phase to address:** Phase that adds the Curated Modular Node Library — must land FIRST among v2 phases because the graph UX, agent planner prompts, and ts-rs generated types all depend on the expanded `NodeKindSpec` table being stable.

---

### Pitfall V2-2: Draggable nodes break the canonical-state-owns-truth invariant unless node positions are first-class session state

**What goes wrong:**
The current `GraphViewport.tsx` ships with `nodesDraggable={false}`. Flipping that to `true` (the v2 graph UX rebuild) without a deliberate state model creates one of two failure modes:

- **Lossy recall:** xyflow owns node `position` locally; the canonical `SessionDocument` has no position field. Reload the session and every node snaps back to a default layout — the performer's carefully-arranged patch space is gone. This is **v1 Pitfall 1 resurfacing** (runtimes/UI becoming the source of truth) but for spatial layout instead of audio state.
- **Topology thrash:** if drag events are routed through the existing `graph_edit` command layer, every drag frame emits a command, which feeds `reconcile_graph_edit` in `audio/runtime_manager.rs`, which (per its match arms) calls `reapply_live_topology` for any non-parameter graph edit — **rebuilding the entire SuperCollider node tree on every drag frame**, producing dropouts and retriggering envelopes.

**Why it happens:**
The v1 graph was deliberately non-draggable so positions never had to be modeled. The `reconcile_graph_edit` match treats `AddNode`/`RemoveNode`/`AddRoute`/`RemoveRoute`/`AssignNodeToBus` as "topology-affecting" and recompiles. There is no command type for "this node moved but nothing about the signal flow changed." Without one, position changes get bucketed with topology changes.

**How to avoid:**
- **Extend the domain model first:** add an optional `layout: { x, y }` (or a separate `NodeLayout` map keyed by node_id) to `SessionDocument`, owned by Rust, exported via ts-rs. This is the canonical position source — xyflow consumes it as a projection.
- **Introduce a new command variant** — e.g. `GraphEditCommand::SetNodeLayout { node_id, x, y }` — that updates session state WITHOUT going through `AudioRuntimeManager::reconcile_graph_edit`. The audio manager's match arm must explicitly NOT match `SetNodeLayout` (or the command must skip the manager entirely).
- **Throttle position writes:** drag emits position continuously, but the canonical command should fire on drag-end (`onNodeDragStop`) or a debounced interval — not on every `onNodeDrag` frame. Local xyflow state can update at 60fps; the session write should be transactional.
- **Verify with the cold-load test from v1 Pitfall 1:** save a session with non-trivial node positions, close, reopen, and confirm positions restore exactly. Then drag a node during playback and confirm no audio dropout / no envelope re-trigger.

**Warning signs:**
- Node positions reset on session reload.
- Dragging a node during audio playback causes clicks, pops, or envelope re-triggers.
- The `audio_runtime.topology_load_count` (visible in `AudioRuntimeManager` tests) climbs once per drag frame instead of once per real topology change.

**Phase to address:** Phase that does the Graph UX Rebuild (draggable nodes + edge connect/reconnect). Must come AFTER the node library domain-model work (V2-1) so the `SetNodeLayout` command is added once for all node kinds.

---

### Pitfall V2-3: Edge connect/reconnect routed around the canonical `graph_edit` command layer

**What goes wrong:**
The current `GraphViewport` has a single `onConnect: (connection) => void` callback that bubbles up to a typed command. xyflow v12 adds `onReconnect(oldEdge, newConnection)` (replacing v11 behavior), `onReconnectStart`, `onReconnectEnd`, plus an `IsValidConnection` validator and `reconnectEdge()` utility. The temptation during the rebuild is to let xyflow own the `edges` array via `useEdgesState` and write directly to local React state — bypassing `graph_edit::GraphEditCommand::AddRoute` / `RemoveRoute`.

When that happens:
- The route exists on the canvas but NOT in `SessionDocument.routes`, so `compile_session_to_topology` (`audio/compiler.rs`) never sees it — **the visual edge is a lie**; audio still routes the old way.
- Ownership checks in `agent_command.rs` (which inspect `session.routes` to enforce `OwnershipRule`) silently approve edits the agent couldn't make through the command layer.
- The v1 routing validator (`validate_routes`: bus references, port existence, disabled-node routes, cycles) is never consulted, allowing cycles that crash `topo_sort_nodes` with `TopologyCompileError::CyclicGraph` only when the session is reloaded.

**Why it happens:**
xyflow is unopinionated about state ownership — it explicitly says "edges and nodes are owned by the parent, not by React Flow." But the parent here is a Zustand mirror of a Rust canonical state, with the typed command layer as the only legitimate write path. Adding `useEdgesState` is the path of least resistance for fast drag-drop UX, and it looks correct in isolation.

**How to avoid:**
- Keep the existing architecture: `graphEdges` and `graphNodes` are PROPS derived from the Zustand mirror of the Rust `SessionDocument`. Do not introduce local edge state.
- Wire `onConnect` and `onReconnect` to the existing typed-command Tauri IPC (`graph_edit::add_route`, `remove_route`). The Rust layer applies ownership/validation, mutates canonical state, and the projection flows back to xyflow.
- Use xyflow's `onBeforeDelete` to intercept edge/node deletion and route it through `GraphEditCommand::RemoveNode` / `RemoveRoute` — do not let xyflow delete from local state.
- Use `isValidConnection` to do **fast canvas-side rejection** of obviously-invalid connections (port direction mismatch, output→output), but treat this as a UX hint ONLY — the authoritative validation stays in Rust `validate_routes`.
- Add a property test: every sequence of xyflow connect/reconnect/delete operations, applied to a session, produces a `SessionDocument` whose `routes` exactly matches the edges xyflow displays. This enforces the invariant structurally.

**Warning signs:**
- An edge appears on the canvas but the audio doesn't change.
- Cycles or invalid routes are only caught on session reload, not at edit time.
- `useEdgesState` (or `useNodesState`) appears anywhere in the graph viewport subtree.

**Phase to address:** Graph UX Rebuild phase. Pairs with V2-2 — they must be designed together because both are about which state xyflow owns vs. which the canonical layer owns.

---

### Pitfall V2-4: Live LLM provider produces `TypedCommand`s the deterministic parser never would (schema-valid, invariant-violating)

**What goes wrong:**
The provider boundary (`application/agent_planner.rs`) is well-shaped for a deterministic provider: `ParserPlannerProvider::plan` returns `PlannerProviderOutput::Typed(PlannerProposal)` with `Vec<TypedCommand>` that the parser constructed from a known-safe grammar. When a live LLM provider is wired in, `parse_planner_json` deserializes the LLM's JSON directly into the SAME `Vec<TypedCommand>` (`PlannerProposalWire`).

The typed command surface was designed assuming the producer is **benign and grammar-constrained**. A live LLM is neither. It can emit commands that:
- **Reference `node_id`s that don't exist** in the session (the parser only ever produces fresh `new_id()` values; the LLM may invent or hallucinate IDs).
- **Target nodes the human has locked** or marked `user_owned` — the ownership gate in `agent_command.rs` will catch some of these via `OwnershipBlocked`, but only for commands that flow through it. If the live provider short-circuits any approval step, locked nodes get mutated.
- **Produce topological impossibilities** the parser would never generate (cycles, output→output port routes, references to ports that don't exist on the target node kind). These pass JSON schema validation and only fail in `compile_session_to_topology` — by which time the session state has already been mutated to a broken shape.
- **Self-referencing parameter commands** (LLM sets a parameter on node A whose value is sourced from a macro that targets node A — the parser's grammar doesn't admit this).
- **Stack-overflow-scale command batches** — the parser emits 1–3 commands per intent; an LLM may emit 50+ in a single proposal.

**Why it happens:**
The `PlannerProposalWire` struct is permissive: `commands: Vec<TypedCommand>` with no per-command bounds, no session-contextual validation, no referential integrity check before deserialization returns. The trait abstraction made this safe for the deterministic provider because the deterministic provider cannot produce these cases. Adding a live provider removes that guarantee without changing the type.

**How to avoid:**
- **Add a `validate_planner_proposal(&proposal, &session)` step** between `parse_planner_json` and returning to the agent command layer. It must reject proposals that:
  - reference any `node_id`, `route_id`, `port_id`, `bus_id`, `parameter_id`, or `macro_id` not present in `session`,
  - contain more than N commands (cap matching the existing `SessionContextBounds` intent — single-digit),
  - target locked/user-owned nodes for mutation commands,
  - would, if applied, produce a `CyclicGraph` or `UnknownPortReference` (run `compile_session_to_topology` on a dry-apply copy and reject on `Err`).
- **Re-derive the confidence → risk-tier → approval-threshold mapping for the live provider.** v1's parser emits confidence in `{0.7, 0.85, 0.9}`; the pending-action thresholds were tuned against those discrete values. LLMs emit continuous values that cluster differently (often >0.9 for trivial ops, <0.5 for risky ones, with a fat middle). Re-using v1 thresholds will silently auto-approve risky proposals or block trivial ones. Treat threshold recalibration as a first-class task, not a config tweak.
- **Add fuzzing:** generate random `PlannerProposalWire` JSON and feed it through `parse_planner_json → validate → dry-apply`. Any panic, hang, or session mutation on a rejected proposal is a bug.
- **Keep a hard floor on the existing safety mechanisms** — even if the live provider claims high confidence, ownership locks, frozen state (`agent_frozen`), and panic reclaim must remain absolute. The provider never gets to override these.

**Warning signs:**
- Live agent sessions produce "missing node" or "unknown port" errors from the audio compiler that the deterministic path never produces.
- A `TypedCommand` from the live provider mutates a node the human believed was locked.
- The pending-action queue is empty for risky operations (auto-approved) or full for trivial ones (over-blocked).
- Agent proposals with 20+ commands are accepted in a single turn.

**Phase to address:** Phase that lands AGNT-01R (live provider-backed agent). Must include an explicit safety-revalidation sub-task BEFORE any live provider is wired in — the deterministic path's safety properties must be re-proven against the new producer.

---

### Pitfall V2-5: Compositing a separate Bevy/GPU window behind the Tauri webview is per-platform and breaks under windowing/protocol assumptions

**What goes wrong:**
v1's `BevySidecarAdapter::start_via_sidecar` **forces `--minimal`** (a GPU-free runtime that just echoes JSON acknowledgements) on the packaged path. v2 wants the opposite: a richer Bevy render COMPOSITED BEHIND/within the webview's graph surface. This is a categorically different problem from the v1 sidecar pattern.

Three independent failure modes:

1. **Webview transparency + child-window stacking is OS-specific.** macOS: requires NSWindow `isOpaque=NO` + a child window ordered behind the WKWebView, with the webview's background painted transparent (CSS `background: transparent` is necessary but not sufficient). Windows: requires DWM composition + layered windows + WS_EX_COMPOSITED; WGC or swapchain-sharing is fragile across Windows versions. Linux: behaves differently on Wayland (no global window stacking) vs X11. There is no "just make the webview transparent" path that works on all three.

2. **The current JSON-lines protocol is synchronous request/response** (`send_and_wait` with a timeout per batch, in `bevy_sidecar.rs`). Rich Bevy rendering at 60fps with continuous parameter changes will saturate this loop: each `update_parameters` call blocks until the ACK arrives. This is **v1 Pitfall 5 resurfacing** for the visual adapter now that it actually has real work to do.

3. **`SCRYSYNTH_VISUAL_MODE` env var on the dev path** silently selects `--minimal` if inherited. The v2 packaged path must explicitly NOT inherit this env, or rich visuals silently degrade to the minimal echo runtime in production for any user who has the env set.

**Why it happens:**
"Visuals behind the grid" reads as a render-quality upgrade but is actually an architecture change: from "sidecar produces telemetry we acknowledge" to "sidecar produces a visible GPU surface we composite with." The v1 boundary was clean precisely because the visual runtime was invisible. Making it visible couples it to windowing, focus, dpi, and platform compositor behavior that v1 never had to deal with.

**How to avoid:**
- **Decide the compositing strategy explicitly and per-platform BEFORE writing renderer code.** The realistic options for a Tauri 2 app are: (a) embed the Bevy render as a streamed texture/animated image into the webview via an IPC'd frame pipe (decouples windowing; costs latency and frame fidelity), (b) run Bevy as a child window positioned behind the main window (native fidelity; per-platform windowing code; z-order/focus pain), or (c) render Bevy to a swapchain/texture and composite via a transparent webview hole (highest fidelity; most platform-specific). Do NOT pick (c) without a working spike on all three target OSes first.
- **Convert the visual adapter protocol to async/streaming** before adding rich visuals: replace `send_and_wait` per-batch blocking with a fire-and-forget parameter stream + periodic ACK/telemetry frame. The canonical state still owns truth (v1 Pitfall 1, Pitfall 7) — the protocol change is purely about transport.
- **Lock down `SCRYSYNTH_VISUAL_MODE` on the packaged path:** explicitly scrub it from the child process env in `start_via_sidecar`, do not just pass `--minimal` as an arg. Currently the env is inherited unless overridden.
- **Add a visual-runtime cold-start test on each target platform** that asserts the renderer actually initializes a GPU surface (not the minimal echo path) before the v2 release ships.

**Warning signs:**
- Visuals work on macOS but render black on Windows/Linux (or vice versa).
- Moving the app window leaves the Bevy render "stuck" in the old position (child-window z-order/focus bug).
- Drag-resizing the app window resizes the webview but not the Bevy surface.
- Frame rate of the visual layer drops as the audio graph grows (shared process/GPU pressure).
- Packaged builds silently run the minimal runtime because a developer env var leaked through.

**Phase to address:** Visuals-behind-the-grid phase. Must include a **compositing spike** as its first work item — pick the strategy with a working cross-platform prototype before committing to renderer richness. This is the highest-uncertainty v2 feature and the most likely to need plan-phase research.

---

### Pitfall V2-6: Cross-platform builds fail at runtime, not build time, because of sidecar target-triple and SuperCollider assumptions

**What goes wrong:**
Two independent landmines hide beneath "the v1 bundle works on my Mac":

1. **Sidecar target-triple mismatch.** Tauri `externalBin` requires `binaries/scrysynth-visual-<target-triple>` for EACH supported architecture. The current `prepare-sidecar.sh` script and v1 release produced only `scrysynth-visual-aarch64-apple-darwin`. Building for `x86_64-apple-darwin`, `x86_64-pc-windows-msvc.exe`, or `x86_64-unknown-linux-gnu` will succeed at `cargo tauri build` time but the packaged app will hit `BevySidecarAdapter::start_via_sidecar`'s `missing_sidecar_message_bundled` path on first launch — runtime failure with a confusing reinstall prompt.

2. **SuperCollider is not bundled.** The current `SuperColliderAdapter` (in `audio/supercollider.rs`) assumes `scsynth` is on PATH or at a known location. This was acceptable on a developer's macOS machine where SC is installed. On a fresh Windows/Linux install it is a first-launch failure: audio boot returns `Failed` with whatever scsynth-resolution error the adapter produces, and the user has no actionable guidance.

If SuperCollider IS bundled to fix (2), a third landmine appears:
3. **Bundled scsynth + signing/notarization.** Any binary inside the macOS app bundle must be signed and notarized or the whole bundle is rejected. Bundling scsynth pulls it into the signing graph (see V2-7).

**Why it happens:**
v1 was macOS-aarch64-only and ad-hoc signed; the dev machine had SC pre-installed. None of these assumptions were ever tested cross-platform, so they baked into the build config invisibly. Cross-platform "support" looks like a CI matrix change but is actually three coupled decisions about (a) which sidecar triples get built, (b) whether/how SC is bundled, and (c) how signing propagates to bundled binaries.

**How to avoid:**
- **Drive sidecar multi-triple from a build matrix, not a single script.** The `prepare-sidecar.sh` script must cross-compile `scrysynth-visual` for every target triple in the release matrix and produce all four suffixed binaries in `src-tauri/binaries/`. Verify the matrix with a CI step that runs `ls src-tauri/binaries/` and asserts the expected suffixed set before `tauri build`.
- **Decide SC bundling strategy explicitly.** Options: (a) require user-installed SC with a clear first-run diagnostic and install link (lowest bundle complexity, worst UX), (b) bundle a stripped `scsynth` per platform behind `externalBin` with target-triple suffixes (highest UX, signing/notarization complexity — see V2-7), (c) vendor SC via a per-platform installer/prereq. Do NOT silently switch between these per-platform; pick one and document it.
- **Add an end-to-end smoke test on each target OS** that boots the packaged app from a clean user account (no dev SC install, no inherited env vars) and verifies audio reaches `Ready`. This is the only reliable catch for the SC-bundling landmine; unit tests cannot reproduce it.
- **Audit `midir` and `rosc` runtime behavior on Windows/Linux early.** `midir` works cross-platform but Windows MIDI APIs (WinMM/Windows MIDI Services) and Linux ALSA/JACK have different device-enumeration and latency behavior. v1 Pitfall 11 ("cross-platform audio assumptions") resurfaces here for the hardware path too.

**Warning signs:**
- `cargo tauri build` succeeds for a target triple but the packaged app crashes or fails to boot audio/visual on first launch.
- The visual sidecar shows the "bundled visual sidecar could not be launched" message on Windows/Linux but never on macOS.
- The bundled `scrysynth-visual` binary set in `src-tauri/binaries/` is incomplete (missing one or more target-triple suffixes).
- A clean-OS smoke test fails on Windows/Linux but passes on the dev's macOS machine.

**Phase to address:** Cross-platform Tauri builds phase. Must come AFTER the visuals-behind-grid phase (V2-5) so the sidecar behavior being bundled is the v2-rich one, not the v1-minimal one. The SC-bundling decision should be made at the START of this phase, not discovered mid-build.

---

### Pitfall V2-7: Notarization + auto-updater cannot ship with ad-hoc signing, and reject any unsigned bundled binary

**What goes wrong:**
The current `tauri.conf.json` has `"signingIdentity": "-"` (ad-hoc). Three things this state cannot do:

1. **Ship an auto-updater.** Tauri's updater requires a Tauri-generated signing keypair (`TAURI_SIGNING_PRIVATE_KEY` env, `pubkey` in config — and `pubkey` MUST be inline content, not a file path). Ad-hoc-signed apps cannot produce verifiable updater artifacts. The updater is gated on switching to a real Developer ID Application certificate first.
2. **Pass notarization with bundled sidecars.** macOS notarization scans every binary in the bundle. The `scrysynth-visual-<triple>` sidecar placed via `externalBin` AND any bundled `scsynth` must each be signed with the Developer ID Application identity or the entire notarization fails with "binary not signed" errors.
3. **Survive the macOS Privacy & Security prompt.** Ad-hoc signing does NOT prevent the prompt — users still have to whitelist the app manually. This is acceptable for v1 (it shipped ad-hoc) but breaks the "studio-grade instrument" feel of v2.

A fourth, subtler landmine: **`.env` files do not work for `TAURI_SIGNING_PRIVATE_KEY`.** It must be set as a real environment variable in CI. Developers who try to use a local `.env` will see silent signing failures or unsigned artifacts.

And a fifth: **the static update manifest is validated WHOLE before version check.** If any per-platform entry in `platforms: { "darwin-x86_64": {...}, "darwin-aarch64": {...}, "windows-x86_64": {...}, "linux-x86_64": {...} }` is incomplete, Tauri rejects the manifest entirely and no platform gets an update.

**Why it happens:**
v1 explicitly deferred notarization to ship. The deferral was correct, but it means every signing/notarization/updater concern is now concentrated in v2 with no prior practice. The signing graph (app + sidecar + bundled SC + Bevy resources) is non-trivial and Tauri's docs describe each piece individually without a consolidated "sign everything in this bundle" recipe.

**How to avoid:**
- **Sequence the work in three explicit sub-tasks, not one:** (1) acquire Developer ID Application cert + App Store Connect API key, (2) sign all bundle binaries + notarize + staple, (3) add updater with generated Tauri signing keypair + published manifest. Trying to land all three at once is how teams spend two weeks on CI.
- **Sign every binary in the bundle, not just the .app.** This includes each `scrysynth-visual-<triple>` and any bundled `scsynth`. Use a pre-build or post-build script (Tauri's `beforeBuildCommand` or a custom hook) to run `codesign --force --options runtime --sign "$DEV_ID" <binary>` on each before `tauri build` packages them.
- **Treat `TAURI_SIGNING_PRIVATE_KEY` and `APPLE_*` secrets as a CI-only concern.** Document that `.env` does not work for the Tauri signing key. Store the private key in GitHub Actions secrets (or equivalent) — and **back it up**, because losing it means no v2 user can ever update again.
- **Generate the updater manifest programmatically** (e.g., via `tauri-action` or a custom script) so every platform entry is populated from a single source of truth. Never hand-edit `latest.json`.
- **Add a `--skip-stapling` first pass** to confirm notarization succeeds before adding the stapling step, per the Tauri macOS signing guide.
- **Add a Windows signing sub-task too** (`tauri.conf.json > bundle > windows > signature` or `TAURI_SIGNING_IDENTITY`); Windows users get SmartScreen warnings otherwise. Authenticode cert acquisition is its own lead-time item — start it early.

**Warning signs:**
- Notarization fails with "binary not signed" pointing at a path inside `Contents/MacOS/` or `Contents/Resources/`.
- The updater silently does nothing on first run (manifest validation failed wholesale).
- CI builds produce unsigned artifacts despite `TAURI_SIGNING_PRIVATE_KEY` being "set" — because it was in `.env`.
- SmartScreen warnings on Windows installs despite "we signed it."
- The `pubkey` field in `tauri.conf.json` is a file path string (it must be inline content).

**Phase to address:** Developer ID Notarization + Auto-updater phase. Must be the LAST v2 phase because it requires (a) cross-platform builds to exist (V2-6) and (b) every bundled binary to be at its final location/name. Doing it earlier means re-signing every time a sidecar or SC binary moves.

---

## Moderate Pitfalls

### Pitfall V2-8: xyflow node ID prefix assumptions break when the node library grows

**What goes wrong:**
`GraphViewport.tsx`'s `minimapNodeColor` switches on `node.id.startsWith("source")` / `"output"` / `"effect"` / `"mixer"`. v1 nodes happen to follow this prefix convention. The curated node library with 12–16 types (LFOs, envelopes, sequencers, additional FX) will either (a) follow a different naming scheme and silently fall through to the default amber color, producing a uniformly-colored minimap, or (b) be forced into awkward prefixes that constrain the domain model.

**How to avoid:** Drive node appearance from a typed `kind`/`category` field on the xyflow node `data` (projected from `Node.node_type` and the new `NodeKindSpec`), never from string-prefix inference on the ID. Replace `minimapNodeColor` with a lookup on `node.data.kind`.

**Warning signs:** Minimap nodes all appear the same color after adding new node types; or, node IDs are forced into prefixes that don't match the domain vocabulary.

**Phase to address:** Curated Node Library phase (drives the category taxonomy) and Graph UX Rebuild phase (consumes it).

---

### Pitfall V2-9: Pro-grade focused shell rebuild silently demotes safety-critical state

**What goes wrong:**
"Progressive disclosure" is easy to implement as "hide everything except the graph by default." The current `GlobalSafetyChrome`, `RuntimeHealthPanel`, pending-action cards, ownership indicators, and panic controls are designed to be always-visible. A focused shell that demotes any of these to a disclosure panel, hover affordance, or submenu breaks the **Control Safety** project constraint ("Human override must stay easy and reliable").

**How to avoid:** Enumerate the always-visible safety surfaces BEFORE redesigning the shell. The minimum set: panic/reclaim button, agent frozen indicator, runtime health (audio/visual), pending-action count, ownership/reclaim state on the selected node. None of these may be hidden behind a disclosure, hover, or submenu — they can be visually minimized but must remain persistently visible.

**Warning signs:** The redesigned shell requires a click to see whether the agent is frozen; panic is in a submenu; runtime health only appears on hover.

**Phase to address:** Pro-grade Focused Shell phase. Must include an explicit safety-surface inventory as a first deliverable.

---

### Pitfall V2-10: Chat sidebar becomes the only authoring surface (project constraint violation)

**What goes wrong:**
The v2 shell spec calls for a "chat sidebar." If the rebuild centers the chat surface as the primary action path while the graph becomes a read-only viewer, the product drifts toward Mindrave — exactly the framing the project was founded to avoid ("Conversation, graph, and performance control are co-equal — chat cannot become the only authoring surface").

**How to avoid:** Treat the constraint as a UAT criterion, not just a design principle. The v2 shell verification must include a scenario: "perform a complete live set using ONLY the graph and performance surfaces, without typing in chat." If that scenario fails, the shell is wrong.

**Warning signs:** Every demo of new v2 features is driven from chat; the graph surface hasn't gained a new mutation primitive in several phases; the product starts being described as "chat with a graph viewer."

**Phase to address:** Pro-grade Focused Shell phase, with UAT reinforcement in every subsequent phase.

---

### Pitfall V2-11: Visual parameter update volume overwhelms the JSON-lines sidecar protocol

**What goes wrong:**
The current `BevySidecarAdapter::update_parameters` is synchronous: it sends a batch, blocks on `wait_for_response` with a timeout, and rejects if the ACK count doesn't match (`"applied {} of {} requested patches"`). The v1 minimal runtime answers instantly because it does no real work. A richer Bevy renderer applying 60fps parameter changes from a playing session will saturate this loop: send → wait → send → wait, falling further behind realtime until the timeout fires and the visual runtime is marked `Failed`.

This is **v1 Pitfall 5 resurfacing** for the visual adapter, now that visuals have real work.

**How to avoid:** Convert the visual adapter protocol to async/streaming (fire-and-forget parameter frames + periodic ACK/telemetry, as in V2-5). The canonical state still owns truth — the change is transport-level, not semantic.

**Warning signs:** Visual `Failed` status under load; visual motion lagging audio by perceptible fractions of a second; visual parameter ACK timeouts in logs.

**Phase to address:** Visuals-behind-the-grid phase (paired with V2-5's transport refactor).

---

### Pitfall V2-12: macOS minimum version vs. Bevy/wgpu GPU feature requirements diverge

**What goes wrong:**
`tauri.conf.json` has `"minimumSystemVersion": "11.0"`. Bevy 0.18 / wgpu 29 may require Metal feature levels only available on macOS 12.0+ (or specific GPU families). Users on Big Sur 11.x will see the app install but the Bevy runtime fail to initialize a GPU surface — and depending on how the failure surfaces, the visual runtime may either crash or silently fall back to the minimal echo runtime.

**How to avoid:** Re-validate the macOS minimum against Bevy 0.18 / wgpu 29's actual Metal requirements. Test the packaged app on a Big Sur VM (or the oldest supported OS) before locking the v2 minimum. Bump `minimumSystemVersion` if needed.

**Warning signs:** Bevy runtime fails to initialize on macOS 11 but works on 13/14; user reports of "visuals never start" on older macOS.

**Phase to address:** Visuals-behind-the-grid phase (validates requirements) and Cross-platform Builds phase (locks the minimum).

---

### Pitfall V2-13: `ts-rs` generated types drift silently when the node library expands domain enums

**What goes wrong:**
Adding 12–16 node types extends `AudioSourceType` and `AudioEffectType` (and likely introduces new primitive categories). `ts-rs` regenerates `src/generated/session-types.ts`. If any frontend switch statement (`minimapNodeColor`, NodeInspector, PrimitivePalette) isn't exhaustive over the new union, new node kinds silently fall through to defaults — wrong colors, missing inspector fields, palette entries that don't map to anything.

**How to avoid:** Treat the generated `session-types.ts` as a contract. Add an exhaustive-switch TS lint (or `never`-default assertion) on every `AudioSourceType` / `AudioEffectType` / new-kind switch in the frontend. Pair with the V2-1 compiler conformance test so a new domain enum value fails BOTH a Rust test AND a TS exhaustive check.

**Warning signs:** A new node kind renders with default styling or an empty inspector; `tsc` doesn't error on incomplete switches; adding an enum variant produces no failing tests.

**Phase to address:** Curated Node Library phase (drives the enum expansion) — paired with the V2-1 conformance test.

---

## Minor Pitfalls

### Pitfall V2-14: Full audio topology reapply on every drag frame (V2-2 variant)

If `SetNodeLayout` is accidentally routed through `reconcile_graph_edit`, every drag frame recompiles and reaps the entire SC topology. The fix is structural (new command variant that bypasses `AudioRuntimeManager`) — covered in V2-2. Listed separately because the symptom is subtle: audio "feels fine" until a performer notices envelopes re-triggering on drag.

**Warning signs:** `audio_runtime.topology_load_count` increments once per drag frame; envelope tails restart during node drags.

**Phase to address:** Graph UX Rebuild phase.

---

### Pitfall V2-15: CSP is `null` — silent failure when the shell redesign tightens security

The current `tauri.conf.json` has `"csp": null` (no enforcement). v2's live LLM agent will call out to an API endpoint. If a security-hardening pass during shell redesign adds a CSP, the agent's HTTP egress will silently fail. Either keep CSP permissive and document why, or add the LLM provider origin to `connect-src` deliberately.

**Warning signs:** Agent HTTP calls fail with CSP violations in the webview console after a security pass.

**Phase to address:** Live Agent phase (validates the actual provider origin) coordinated with any security pass in the Focused Shell phase.

---

### Pitfall V2-16: `SCRYSYNTH_VISUAL_MODE=minimal` env inheritance silently degrades packaged visuals

`BevySidecarAdapter::start_via_command` (the dev path) checks `SCRYSYNTH_VISUAL_MODE` and selects `--minimal` if set. The packaged path (`start_via_sidecar`) explicitly passes `--minimal`, but if the visual-bin entrypoint (`src/bin/scrysynth-visual.rs`) re-reads the env on its side (which it does — see lines 8–9), a user with the env set will get the minimal runtime even when the parent app passes different args. Scrub the env from the child process in v2's rich-visual path.

**Warning signs:** A developer with a stale env var reports "visuals look like v1" on a v2 build.

**Phase to address:** Visuals-behind-the-grid phase.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Letting xyflow own edge state via `useEdgesState` | Faster drag-drop UX; less IPC churn | Visual edges diverge from canonical `SessionDocument.routes`; ownership/validation bypassed | Never for v2 — the canonical-state invariant is a project constraint |
| Hardcoding a new node's synthdef name in `synthdef_resource` instead of refactoring to a `NodeKindSpec` table | Faster per-node delivery | Three allowlists (`synthdef_resource`, `normalize_parameter_name`, `validate_runtime_target`) diverge; `unreachable!()` panic landmines grow | One-off prototype only — convert to spec table before the second new node ships |
| Reusing v1 confidence thresholds for the live LLM provider | No re-tuning work | Risky ops auto-approved; trivial ops over-blocked; agent trust collapses | Never — re-derive thresholds explicitly (V2-4) |
| Bundling scsynth per-platform without auditing signing propagation | "It just works" on dev machines | Notarization fails on bundle; SmartScreen warnings on Windows; release slips | Never in v2 — signing graph must be designed, not discovered |
| Treating visuals-behind-grid as a render-quality upgrade | Avoids the compositing-architecture decision | Per-platform windowing pain discovered late; v1 Pitfall 7 (AV over-coupling) resurfaces | Never — spike the compositing strategy first (V2-5) |
| Ad-hoc node ID prefix conventions for theming | Quicker minimap wiring | Brittle theming that breaks as the library grows (V2-8) | v1 only — already obsolete |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| xyflow v12 `onReconnect` | Wiring it to mutate local edge state | Route through `GraphEditCommand::RemoveRoute` + `AddRoute`; let Rust own the canonical edge list |
| xyflow `useEdgesState` / `useNodesState` | Using it in the Scrysynth graph viewport at all | Don't — graph state is a Zustand mirror of the Rust `SessionDocument`; xyflow is a controlled component |
| SuperCollider new synthdef | Adding a `.scsyndef` file without extending all three v1 allowlists | Refactor to `NodeKindSpec` table (V2-1); add compiler conformance test |
| Tauri `externalBin` (sidecar) | Bundling only the dev's host-triple binary | Build matrix produces ALL target-triple-suffixed binaries; CI asserts the expected set (V2-6) |
| Tauri updater `pubkey` | Setting it to a file path string | Must be inline key content (PEM body); never a path |
| Tauri updater `TAURI_SIGNING_PRIVATE_KEY` | Putting it in `.env` | Real env var only (CI secret); `.env` is silently ignored |
| Tauri updater static manifest | Hand-editing `latest.json` | Generate via `tauri-action` or script; Tauri validates the whole file before checking version |
| Tauri macOS signing | Ad-hoc (`-`) and expecting the updater to work | Cannot — updater requires real Developer ID Application cert + notarization |
| macOS notarization | Signing the `.app` but not the bundled sidecars/SC | Every binary in the bundle must be signed (V2-7) |
| Live LLM provider JSON | Deserializing directly into `Vec<TypedCommand>` and applying | Insert `validate_planner_proposal` step: reject unknown IDs, oversized batches, locked-target mutations, dry-apply-invalid ops (V2-4) |
| Live LLM confidence | Reusing v1 parser's discrete thresholds | Re-derive the confidence → risk-tier → approval-threshold mapping for the live provider |
| Bevy visible window behind webview | Assuming transparent webview "just works" cross-platform | Spike compositing strategy on all three OSes first (V2-5) |
| `midir` cross-platform | Assuming macOS CoreMIDI behavior on Windows/Linux | Test device enumeration + latency on each target OS; v1 Pitfall 11 |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Full SC topology reapply on every drag frame | Audio dropouts during node drags; `topology_load_count` climbs per frame | Add `SetNodeLayout` command variant that bypasses `AudioRuntimeManager` | First time a performer drags a node during live audio (V2-2, V2-14) |
| Synchronous send/wait visual protocol under 60fps load | Visual `Failed` status; render lagging audio; ACK timeouts | Convert to async streaming protocol (V2-5, V2-11) | Any session with continuous parameter motion AND rich visuals |
| Tauri IPC for dense visual parameter frames | UI jank; webview thread saturation | Aggregate/throttle; keep canonical state updates at semantic boundaries (v1 Pitfall 5) | Visual sessions with >50 param changes/sec |
| Bundling all SC resources per-platform | Bundle size; install time; CI build time | Strip unused synthdefs per-platform; consider lazy-load for non-core nodes | Cross-platform Builds phase, especially Windows installer size |
| Agent proposals with 20+ commands in one turn | UI freeze during apply; cascading ownership conflicts | Cap proposal command count in `validate_planner_proposal` | Live LLM provider producing verbose plans (V2-4) |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Live LLM provider `Vec<TypedCommand>` applied without invariant re-validation | Agent mutates locked nodes, invents node IDs, creates cycles that crash audio on reload | `validate_planner_proposal` step before any mutation; dry-apply topology check (V2-4) |
| CSP `null` tightened to restrictive without allowlisting the LLM provider origin | Agent silently fails to reach its provider; opaque error | Plan the CSP together with the live-agent integration (V2-15) |
| Tauri updater private key committed to repo or stored in `.env` | Anyone with repo access can ship malicious updates to all v2 users | CI secret only; backed up offline; never in `.env` |
| Bundled scsynth unsigned inside the macOS app | Notarization fails OR — worse — a tampered bundle passes with unsigned binaries inside | Sign every binary in the bundle (V2-7) |
| `SCRYSYNTH_BEVY_PATH` / `SCRYSYNTH_VISUAL_MODE` env honored on packaged path | User-influencable runtime that can swap the renderer or point at arbitrary binaries | Scrub env from child process on packaged path; honor override ONLY when no AppHandle (V2-16) |
| Live provider API key in frontend bundle | Key extraction; unauthorized provider usage | Key stays in Rust core (or backend proxy); frontend never sees it |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Safety surfaces (panic, frozen, ownership, health) demoted behind progressive disclosure | Performer can't tell who owns a node or whether the agent is frozen in a glance | Always-visible safety chrome; progressive disclosure only for non-safety panels (V2-9) |
| Chat becomes the only authoring surface | Drift toward Mindrave; graph and performance modes become viewers | UAT scenario: "perform a live set using only graph + performance, no chat" (V2-10) |
| Node library organized by SuperCollider synthdef category instead of musical function | Performers need SC knowledge to find an LFO | Organize by musical function (sources, modifiers, sequencers, FX, routing); hide synthdef names (v1 Pitfall 2) |
| Edge reconnect requires precise handle pinning | Reconnecting a busy patch feels fiddly and error-prone | Use xyflow's "easy connect"/proximity patterns; allow drop on the node body, not just the handle |
| Visual layer competes with graph for visual attention | Performer can't read the patch when visuals are active | Visuals are AMBIENT — behind the grid, low contrast against node chrome; the grid stays legible at all visual intensities |
| Cross-platform UI assumes macOS conventions | Windows/Linux users hit wrong-keyboard shortcuts, no right-click menus, etc. | Audit keyboard/mouse conventions per platform; respect OS conventions for menu, fullscreen, file pickers |

---

## "Looks Done But Isn't" Checklist

- [ ] **Graph drag:** Often missing — position round-trip through canonical state. Verify: save → close → reopen restores positions exactly (V2-2).
- [ ] **Edge reconnect:** Often missing — `onReconnect` wired but routed to local state instead of `GraphEditCommand`. Verify: reconnect an edge, reload, confirm audio routes through the new connection (V2-3).
- [ ] **New node type:** Often missing — synthdef committed but one of the three allowlists not updated. Verify: boot a session containing the new node against a real `scsynth`; no `Unsupported*` errors and no panics (V2-1).
- [ ] **Live agent:** Often missing — proposal validation. Verify: feed the live provider malformed/hallucinated JSON and confirm it's rejected before mutation (V2-4).
- [ ] **Visuals behind grid:** Often missing — cross-platform compositing. Verify: visuals composite correctly on macOS AND Windows AND Linux (V2-5).
- [ ] **Cross-platform build:** Often missing — sidecar target-triple for non-dev arch; SC bundling decision. Verify: clean-OS smoke test on each target boots to audio `Ready` (V2-6).
- [ ] **Notarization:** Often missing — bundled sidecars/SC signed. Verify: `spctl --assess --verbose` and `xcrun stapler validate` pass on the FINAL bundle, not just the `.app` (V2-7).
- [ ] **Auto-updater:** Often missing — manifest with ALL platform entries complete; `pubkey` is content not path; private key in CI env not `.env`. Verify: a real downgrade/upgrade test against the published manifest (V2-7).
- [ ] **Focused shell:** Often missing — always-visible safety surfaces. Verify: panic/frozen/ownership/health visible at every shell state, no disclosure required (V2-9).
- [ ] **Node library theming:** Often missing — typed kind/category field driving appearance. Verify: minimap colors are correct after adding the full library, not just v1 prefixes (V2-8).

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| V2-1 (allowlist panic on boot) | LOW | Add missing synthdef entry + parameter mapping + runtime_target match; add `.scsyndef`; rerun conformance test |
| V2-2 (positions not canonical) | MEDIUM | Add `SetNodeLayout` command + `layout` field; migrate existing sessions to populate positions from xyflow state at save time |
| V2-3 (edges diverged from session) | HIGH | Audit every edge-producing UI path; route through `graph_edit` commands; add property test; reconcile divergent sessions manually |
| V2-4 (LLM produced invariant-violating commands) | MEDIUM | Add `validate_planner_proposal`; dry-apply + ownership re-check; re-derive confidence thresholds; fuzz-test the validator |
| V2-5 (compositing strategy wrong) | HIGH | Re-spike on all three OSes; may require switching from child-window to streamed-texture approach; significant rewrite if discovered late |
| V2-6 (sidecar/SC bundling) | MEDIUM | Build the missing target-triple binaries; decide SC bundling strategy; clean-OS test on each platform |
| V2-7 (notarization fails) | MEDIUM | Backfill signing for every bundle binary; re-run notarization; verify updater manifest is complete and key is in CI env |
| V2-10 (chat became only surface) | HIGH | Product-level correction; may require re-prioritizing graph + performance features in remaining phases |

---

## Pitfall-to-Phase Mapping

Pitfalls mapped to suggested v2 phase ordering. Phases are labeled by feature; the roadmapper should assign actual phase numbers. Ordering rationale: domain model first (everything depends on it), then UX surfaces that consume it, then agent (needs stable graph + nodes), then visuals (parallelizable, high uncertainty), then cross-platform (needs stable feature set), then signing/updater (needs final binaries).

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| V2-1 (synthdef/param/runtime allowlists) | Curated Node Library (FIRST) | Compiler conformance test: every spec entry compiles + loads against real `scsynth` |
| V2-13 (ts-rs enum drift) | Curated Node Library | Exhaustive TS switch lint passes for every new enum variant |
| V2-8 (node ID prefix theming) | Curated Node Library + Graph UX Rebuild | Minimap colors correct for all node kinds via typed `category`, not ID prefix |
| V2-2 (node positions not canonical) | Graph UX Rebuild | Cold-load test restores positions; no audio dropout during drag (`topology_load_count` flat) |
| V2-3 (edges bypass canonical layer) | Graph UX Rebuild | Property test: xyflow edge set == `SessionDocument.routes` after every interaction |
| V2-14 (topology reapply on drag) | Graph UX Rebuild | Drag a node during playback; envelope tails do not re-trigger |
| V2-9 (safety surfaces demoted) | Pro-grade Focused Shell | Safety-surface inventory delivered first; UAT: all surfaces visible in every shell state |
| V2-10 (chat-only authoring drift) | Pro-grade Focused Shell (UAT in all later phases) | UAT scenario: complete live set using only graph + performance, no chat |
| V2-15 (CSP null vs. agent HTTP) | Pro-grade Focused Shell + Live Agent | Agent HTTP egress works under the chosen CSP |
| V2-4 (LLM invariant-violating commands) | Live Agent (AGNT-01R) | Fuzz test: random JSON through `validate_planner_proposal` never mutates session; thresholds re-derived |
| V2-5 (compositing strategy) | Visuals Behind the Grid | Working spike on macOS + Windows + Linux BEFORE rich renderer work |
| V2-11 (visual protocol saturation) | Visuals Behind the Grid | 60fps parameter changes for 60s without `Failed` status |
| V2-12 (macOS min vs. Bevy GPU) | Visuals Behind the Grid + Cross-platform | Packaged app boots visuals on the minimum-supported macOS |
| V2-16 (visual mode env inheritance) | Visuals Behind the Grid | Packaged app ignores `SCRYSYNTH_VISUAL_MODE` from user env |
| V2-6 (sidecar triples + SC bundling) | Cross-platform Builds | Clean-OS smoke test on each target boots audio to `Ready`; CI asserts expected sidecar triple set |
| V2-7 (signing + notarization + updater) | Notarization + Auto-updater (LAST) | `spctl` + `stapler validate` pass on final bundle; real upgrade test against published manifest |

**Research flags for phases (need deeper plan-phase research):**
- **Visuals Behind the Grid:** HIGHEST uncertainty. Compositing strategy must be spiked before the phase is planned in detail. Likely needs its own research spike on webview transparency + GPU child window per-platform.
- **Live Agent (AGNT-01R):** Confidence-threshold recalibration and `validate_planner_proposal` design need their own focused work; not a generic "wire in provider" task.
- **Cross-platform Builds + SC bundling:** The SC bundling decision (require user install vs. bundle vs. installer-prereq) is a product decision that should be made BEFORE plan-phase, not discovered mid-build.
- **Standard patterns, unlikely to need research:** Curated Node Library (compiler already understood), Graph UX Rebuild (xyflow v12 well-documented), Notarization + Auto-updater (Tauri docs are thorough; main risk is sequencing and signing-graph design, not unknown technology).

---

## Sources

- Scrysynth v1 codebase (grounded, primary source for all integration claims):
  - `src-tauri/src/audio/synthdefs.rs` — three hardcoded allowlists, `unreachable!()` landmine, `plan_sc_resources` — HIGH
  - `src-tauri/src/audio/compiler.rs` — `compile_session_to_topology`, `validate_routes`, `topo_sort_nodes` `CyclicGraph` — HIGH
  - `src-tauri/src/audio/runtime_manager.rs` — `reconcile_graph_edit` match arms, `reapply_live_topology` on non-parameter edits — HIGH
  - `src-tauri/src/application/agent_planner.rs` — `PlannerProvider` trait, `ParserPlannerProvider`, `parse_planner_json`, `PlannerProposalWire` — HIGH
  - `src-tauri/src/application/agent_command.rs` — `parse_agent_intent`, discrete confidence values `{0.7, 0.85, 0.9}`, ownership gate — HIGH
  - `src-tauri/src/visual/bevy_sidecar.rs` — `start_via_sidecar` forces `--minimal`, synchronous `send_and_wait` protocol, `SCRYSYNTH_VISUAL_MODE` env handling, bundled/dev spawn paths — HIGH
  - `src-tauri/src/visual/adapter.rs` — `VisualRuntimeAdapter` trait — HIGH
  - `src-tauri/src/bin/scrysynth-visual.rs` — env-var-driven `--minimal` selection re-read on the child side — HIGH
  - `src/components/session/GraphViewport.tsx` — `nodesDraggable={false}`, single `onConnect`, ID-prefix `minimapNodeColor` — HIGH
  - `src-tauri/tauri.conf.json` — `"signingIdentity": "-"`, `"csp": null`, `externalBin: ["binaries/scrysynth-visual"]`, `minimumSystemVersion: "11.0"`, macOS-only targets — HIGH
  - `src-tauri/Cargo.toml` — `bevy = "0.18.1"`, `midir = "0.10"`, `rosc = "0.11"` — HIGH
- Tauri 2.10 official docs (last updated Jun 15 / Nov 28 / May 17, 2026):
  - Embedding External Binaries / sidecar — host-triple-`-$TARGET_TRIPLE` requirement, `sidecar()` takes bare filename — HIGH
  - Updater plugin — `pubkey` is content not path, `.env` doesn't work, `createUpdaterArtifacts`, HTTPS enforced, static JSON validated whole, Windows auto-exit, custom target for universal macOS — HIGH
  - macOS Code Signing — Developer ID Application cert, notarization required, `APPLE_*` env vars, ad-hoc does not prevent Privacy & Security prompt — HIGH
- xyflow v12 official docs (last updated Jun 17, 2026):
  - Custom Edges guide; API reference index confirms `OnReconnect`, `reconnectEdge()`, `IsValidConnection`, `OnBeforeDelete`, `useEdgesState` — HIGH
  - Examples: Reconnect Edge, Prevent Cycles, Connection Limit, Save and Restore — confirm patterns referenced — MEDIUM-HIGH
- v1 PITFALLS.md (Scrysynth, 2026-04-11) — referenced for resurfacing pitfalls:
  - Pitfall 1 (runtimes/UI as source of truth) — resurfaces as V2-2 (positions), V2-3 (edges)
  - Pitfall 2 (graph primitives tied to engine internals) — reinforces V2-1
  - Pitfall 3 (SC execution order/bus topology) — informs V2-1 conformance testing
  - Pitfall 5 (IPC as realtime transport) — resurfaces as V2-11 (visual protocol saturation)
  - Pitfall 6 (ownership at control surface) — reinforces V2-4, V2-9
  - Pitfall 7 (over-coupling AV) — resurfaces as V2-5 (visuals behind grid)
  - Pitfall 11 (cross-platform audio) — resurfaces as V2-6 (midir/rosc per-OS)
  - Pitfall 12 (agent provenance) — reinforces V2-4
  - Pitfall 13 (coarse emergency recovery) — reinforces V2-9 (safety surfaces)

---
*Pitfalls research for: Scrysynth v2.0 "Studio-Grade Instrument" — adding features to an existing Tauri 2 / Rust / React + xyflow / SuperCollider audiovisual instrument*
*Researched: 2026-06-26*
