---
slug: send-agent-message-planner
status: resolved
trigger: "Plan 11-02 blocker: send_agent_message bypasses the PlannerProvider machinery - wire plan_and_apply_agent_request in and ensure a pending-action-triggering input exists"
created: 2026-06-25
updated: 2026-06-25
goal: find_and_fix
source: .planning/phases/11-release-readiness/.pause-handoff.md
---

# Debug Session: send_agent_message bypasses PlannerProvider

## Symptoms

- **Expected behavior:** `send_agent_message` (the production conversation path invoked from the GUI) should route through `plan_and_apply_agent_request` so the `PlannerProvider` machinery in `agent_planner.rs` is reachable, pending actions can be created, and the approval flow (`PendingActionCard` in `ConversationView.tsx`) receives production data.
- **Actual behavior:** `send_agent_message` at `src-tauri/src/lib.rs:113-133` calls `agent_command::parse_agent_intent` directly and feeds the result to `apply_agent_command`. It never invokes `plan_and_apply_agent_request` (`src-tauri/src/application/agent_command.rs:362`). The `PlannerProvider` machinery is unreachable from the GUI; the local regex parser never emits a high-risk command, so no production GUI flow ever creates a pending action.
- **Error messages:** None — silent functional gap. No exception, just a missing code path.
- **Timeline:** Discovered during Phase 11 Plan 11-02 UAT scenario 7 (agent approval). Not a Plan 11-01 regression. Phase 10's UAT was technically honest (caveat: "does not verify... GUI click-through in the packaged desktop app") but that caveat was not propagated to REL-02 scope. Phase 11 inherited the gap.
- **Reproduction:** Type any input in the conversation UI. No pending action card ever appears because (a) the planner is bypassed and (b) the local parser never classifies anything as high-risk.

## Verification of handoff accuracy (pre-session)

Confirmed current as of session creation:
- `src-tauri/src/lib.rs:113-133` — `send_agent_message` still calls `parse_agent_intent` + `apply_agent_command` directly. No planner invocation.
- `src-tauri/src/application/agent_command.rs:362` — `plan_and_apply_agent_request` exists and is the function Phase 10 verified via Rust integration tests.
- `src-tauri/src/application/agent_planner.rs:360` — `ParserPlannerProvider` exists; `Default` impl at line 365; `PlannerProvider` impl at line 383. `ParserPlannerProvider::default()` wraps `parse_agent_intent` and is the natural behavior-preserving default provider.

## Fix scope (from handoff)

1. **Wire the planner into `send_agent_message`.** Replace the direct `parse_agent_intent` + `apply_agent_command` call with `plan_and_apply_agent_request` using a default provider. `ParserPlannerProvider::default()` is the natural default (it already wraps `parse_agent_intent`) — preserves current behavior while routing through the planner gates.
2. **Ensure at least one user-typed input produces a high-risk pending action.** The local parser today never emits a high-risk command. **Design decision: left to the debugger to recommend.** Investigate both (a) extending `parse_agent_intent` to classify certain agent-attributed commands (e.g. `remove <agent-owned node>`) as high-risk, and (b) adding a deterministic v1 provider that emits high-risk commands for a defined trigger phrase. Recommend the smaller, behavior-preserving change against the existing risk-classification code.
3. **Add a Rust integration test** that drives `send_agent_message` end-to-end and asserts a pending action appears for the chosen trigger input.
4. **Re-run Phase 10 UAT** to confirm no regression in the existing planner path.
5. **Rebuild the .app** (`npm run tauri build`) so Plan 11-02 UAT can re-target the fixed packaged artifact. (Likely out of scope for this debug session — Plan 11-02 owns the rebuild + GUI re-verification. Confirm boundary with operator if ambiguity arises.)

## Constraints / context

- This is an investigation + fix + test cycle. The new session owns the code change; Plan 11-02 then resumes from Task 1 step 6.
- A secondary issue (visual runtime not auto-starting) is noted in the handoff but is **out of scope** for this debug session unless scenario 8 regressions appear after the planner fix.
- Do NOT use `gsd:fast`/`general-purpose` fallbacks — use exact GSD subagent types.

## Current Focus

- hypothesis: Routing `send_agent_message` through `plan_and_apply_agent_request` with `ParserPlannerProvider::default()` will restore the pending-action/approval data path without changing existing behavior; a separate change is required to make at least one input actually trigger a high-risk pending action.
- test: (to be defined) Rust integration test driving `send_agent_message` end-to-end and asserting a pending action appears for the chosen trigger input.
- expecting: Pending-action card in `ConversationView.tsx` to receive production data for at least one user-typed input; existing Phase 10 integration tests still pass.
- next_action: gather initial evidence — read `plan_and_apply_agent_request` signature/body, `ParserPlannerProvider` impl, the existing risk-classification code in `parse_agent_intent`, and the Phase 10 integration tests in `src-tauri/tests/agent_commands.rs` to determine the smallest behavior-preserving fix and the smallest way to produce a high-risk pending action.

## Evidence

- 2026-06-25 initial code reading: `src-tauri/src/lib.rs:113-133` (send_agent_message), `src-tauri/src/application/agent_command.rs:301-360` (apply_agent_command), `:362-386` (plan_and_apply_agent_request + apply_planner_proposal), `:137-151` (classify_risk), `:153-240` (parse_remove_command + extract_node_reference). `src-tauri/src/application/agent_planner.rs:359-421` (ParserPlannerProvider). `src-tauri/src/application/session_store.rs:696-736` (check_ownership). `src-tauri/tests/agent_commands.rs:1-120` (test_session fixture with node-agent owned by ControllerKind::Agent).
- 2026-06-25 hypothesis refined: the pending-action gate in `apply_agent_command` is `risk == RiskTier::High && actor.actor_id != "user"` (agent_command.rs:319). The actor in send_agent_message IS `"agent"` (satisfies != "user"), and `apply_agent_command` ALREADY contains the pending-action logic — so the wiring bypass alone was NOT the sole cause. The deeper cause: the parser's only High-risk emitter is `RemoveNode`, but `extract_node_reference` only matches opaque node `id`s the user never sees in production (node ids are `new_id()` strings). So no natural user input ever reached the High-risk branch. The wiring bypass additionally meant the PlannerProvider abstraction + planner_provider_id were unreachable from the GUI.
- 2026-06-25 bug reproduced (probe test, since replaced): `agent_command::parse_agent_intent("remove the agent layer", &session)` returned an empty `parsed_commands` vec against the test_session fixture (which contains an agent-owned `node-agent`). Confirmed the parser emits zero commands for natural agent-layer removal language.
- 2026-06-25 fix applied (3 parts). See Resolution.

## Eliminated

- hypothesis: "The local regex parser never emits a high-risk command" (handoff root cause wording) — REFINED, not eliminated. The parser CAN emit RemoveNode (High) for `remove node <opaque-id>`, but that path is unreachable from natural user input because users never see node ids. The accurate statement: no NATURAL user-typed input produces a high-risk command. Confirmed by reading extract_node_reference (matches only raw node.id) and the probe test.
- hypothesis: "Wiring bypass is the sole cause" — the wiring bypass was real (PlannerProvider unreachable) but functionally near-equivalent for the default parser since both paths reach apply_agent_command (which owns the pending-action logic). The functional blocker was the parser's id-only node matching. Both fixed together.

## Resolution

status: resolved
root_cause: Two compounding gaps. (1) WIRING: `send_agent_message` called `parse_agent_intent` + `apply_agent_command` directly, never `plan_and_apply_agent_request`, so the PlannerProvider abstraction and planner_provider_id were unreachable from the GUI. (2) PENDING-ACTION TRIGGER: the parser's only High-risk emitter (RemoveNode) was unreachable from natural user input because `extract_node_reference` only matches opaque node `id`s that production users never see.

specialist_hint: rust-tauri

recommendation_chosen: "Let debugger recommend" — investigated both (a) extending parse_agent_intent and (b) adding a deterministic v1 provider. Chose (a): a small additive fallback inside `parse_remove_command` that targets the first agent-owned, unlocked node when the input references "agent". Reuses the existing High-risk classification (RemoveNode is already RiskTier::High). No new provider, no new trait impl, no dispatch broadening in parse_agent_intent.

### fix (3 parts)

1. **Testable core** — `src-tauri/src/application/agent_command.rs`: added `pub fn handle_agent_message(store, message) -> Result<AgentCommandResult, AgentCommandError>` that constructs the `"agent"` ActorRef, instantiates `ParserPlannerProvider::default()`, and delegates to `plan_and_apply_agent_request`. Added `ParserPlannerProvider` to the `agent_planner` use-statement. This is the end-to-end entry point the Tauri command and tests both call.
2. **Tauri wiring** — `src-tauri/src/lib.rs:113-129`: rewrote `send_agent_message` to call `agent_command::handle_agent_message(&mut store, &message)` and serialize `{ session, intent: result.intent }`. Frontend contract (session-client.ts:468 `{ session, intent }`) preserved exactly — ConversationView already reads pendingActions from the returned session. Removed the now-unused `ActorRef` from the `domain::session` use-statement (was only used inside the old send_agent_message body; verified by grep — no other lib.rs usage).
3. **Pending-action trigger** — `src-tauri/src/application/agent_command.rs` `parse_remove_command`: kept the existing `extract_node_reference` (remove-by-opaque-id) path as-is, then added an additive fallback: if the input contains "agent" and there is an agent-owned, unlocked node, emit `RemoveNode { node_id: <first agent-owned node> }`. RemoveNode is already RiskTier::High, the actor is "agent" (!= "user"), and agent-owned unlocked nodes pass check_ownership, so the existing pending-action gate fires. Smallest behavior-preserving change: only adds a fallback branch; existing remove-by-id behavior is unchanged.

files_changed:
- src-tauri/src/application/agent_command.rs (import + parse_remove_command + handle_agent_message)
- src-tauri/src/lib.rs (send_agent_message body + ActorRef import removal)
- src-tauri/tests/agent_commands.rs (new integration test `handle_agent_message_routes_through_local_parser_and_creates_pending_action`)

### verification

- `cargo test --test agent_commands`: 37 passed, 0 failed. Includes the new end-to-end test AND all Phase 10 planner tests (mock_planner_high_risk_fixture_waits_for_approve_or_reject_flow, plan_and_apply_routes_through_planner_before_validation, etc.) — no regression in the existing planner path (handoff fix-scope step 4 satisfied).
- New test asserts: `planner_provider_id == Some("local-parser")` (wiring through planner confirmed), `pending.len() == 1`, `pending[0].risk_tier == High`, `pending[0].status == Pending`, pending action present in `store.current().pending_actions`, and the agent-owned node still present (not yet approved/removed).
- `npx tsc --noEmit`: exit 0 (frontend contract intact).
- `npm test` (vitest): 47 passed, 0 failed — incl. ConversationView.test.ts (9) and session-client.test.ts (3) which exercise the sendAgentMessage contract.
- `rustfmt --check` on all three edited files: exit 0.

### out-of-scope / pre-existing issues (NOT caused by this fix, not fixed here)

- `cargo clippy --all-targets -- -D warnings` fails with ~29-30 errors on HEAD (confirmed via `git stash` that they pre-date this change). Locations: src/audio/runtime_manager.rs, src/visual/runtime_manager.rs, src/visual/bevy_runtime.rs, src/application/session_store.rs, src/domain/session.rs, and pre-existing lints in agent_command.rs at lines 223/282/342/444/462/504/719 (none at this fix's edit sites). Repo simply does not pass clippy-on-HEAD. Recommend a separate cleanup task.
- `rustfmt --check` repo-wide reports diffs in src/visual/runtime_manager.rs (pre-existing) — not touched.
- rustfmt could not fully resolve the module tree due to a pre-existing parse error in src/visual/bevy_sidecar.rs:364 (`async` block needs edition 2024) — unrelated to this fix.
- Lib unit test `osc_listener_shutdown_releases_udp_port` (session_store.rs:1532) is environmentally flaky: fails with "Address already in use" under parallel test load, passes in isolation. UDP port contention, unrelated to agent/planner code.

### scope boundary (handoff fix-scope steps 4-5)

- Step 4 (re-run Phase 10 UAT planner path): satisfied at the integration-test level (all Phase 10 agent_commands tests still pass). Full Phase 10 UAT re-run is Plan 11-02's responsibility.
- Step 5 (rebuild .app via `npm run tauri build`): OUT OF SCOPE for this debug session — Plan 11-02 owns the rebuild and the GUI UAT re-verification (scenario 7 is now reachable). The fix changes Rust code, so the prior packaged .app is stale; Plan 11-02 must rebuild before re-running Task 1 step 6 and Task 2 scenarios.
- Trigger phrase for Plan 11-02 scenario 7: type "remove the agent layer" (or any "remove/delete ... agent ..." input) in the conversation UI against a session that has at least one agent-owned, unlocked node. The PendingActionCard should now render with production data.

### known limitation (v1)

The parser fallback resolves "remove ... agent ..." to the FIRST agent-owned, unlocked node. If multiple agent-owned nodes exist, only one is targeted per message (producing one pending action). This is sufficient to make the approval flow reachable (the v1 goal) and matches the handoff's "remove <agent-owned node>" framing. Richer multi-node / name-based resolution is a future enhancement, not a v1 blocker.

## Resolution (filled above)

## Follow-up fix (same session): approve/reject IPC arg-name mismatch

After the planner wiring fix landed, the operator reported that clicking Accept/Reject on the PendingActionCard showed "Unable to reject action." / "Unable to approve action." in the rebuilt app.

root_cause (follow-up): Tauri v2 command-argument binding mismatch. Frontend sent `{ actionId }` (session-client.ts:485,489); Rust params were `pending_action_id: String` (lib.rs:149,159). Tauri binds JS camelCase keys to Rust snake_case params, so Rust `pending_action_id` expected JS `pendingActionId`, not `actionId`. Binding failed → invoke rejected with a plain string (not an Error) → sessionStore.ts:636,648 catch block fell through to the literal "Unable to ... action." fallback. Symptom string matched exactly.

why latent until now: (a) browser-preview path (browser-preview-session.ts:93-97,625,639) reads `actionId` on BOTH sides of its own internal dispatch, so vitest passed; (b) Phase 10 Rust integration tests call `agent_command::approve_pending_action`/`reject_pending_action` domain functions directly, bypassing the Tauri binding; (c) no pending action was ever created in production before the planner-wiring fix, so the real Tauri binding for approve/reject had never been exercised. The planner-wiring fix made scenario 7 reachable, which made this binding bug reachable.

fix (follow-up): renamed the Rust command params `pending_action_id` → `action_id` in both `approve_pending_action` and `reject_pending_action` (src-tauri/src/lib.rs). Aligns the binding with the frontend's existing `actionId` convention and the browser-preview's `actionId` key. Internal domain functions (`agent_command::approve_pending_action`/`reject_pending_action`) unchanged — only the Tauri-command parameter name and its local reference changed.

audit (follow-up): cross-checked EVERY multi-word Tauri command param in lib.rs against its frontend invoke key. All others already match: `node_ids`↔`nodeIds`, `target_controller`↔`targetController`, `binding_id`↔`bindingId`, `max_events`↔`maxEvents`, single-word params (`path`, `message`, `command`, `target`, `settings`). The approve/reject pair was the only mismatch.

files_changed (follow-up): src-tauri/src/lib.rs (approve_pending_action + reject_pending_action param rename).

verification (follow-up): `cargo check --all-targets` clean; `cargo test --test agent_commands` 37/37. Final GUI verification (click Approve/Reject against a planner-created pending action in the rebuilt .app) belongs to Plan 11-02 scenario 7.

note: no regression test added for the binding itself — Tauri command-arg binding cannot be exercised without a full Tauri runtime, and frontend tests mock the IPC layer. The safeguard is the name-alignment audit above. If a future refactor wants a structural guard, the path forward is generating the IPC arg names from the Rust command signatures (ts-rs over the command params, or tauri-specta) rather than hand-writing session-client.ts.
