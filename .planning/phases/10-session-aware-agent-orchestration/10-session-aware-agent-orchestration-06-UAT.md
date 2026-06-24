# Phase 10.6 UAT: Session-Aware Agent Orchestration

Task: `td-ab3c29`
Date: 2026-06-21
Provider mode: deterministic/mock planner fixtures and local parser provider, no live remote provider or credentials.
Machine-readable artifact: `uat-artifacts/td-ab3c29/evidence.json`

## Scope

This UAT verifies the session-aware planner path that exists today: bounded context packet derivation, provider-agnostic planner requests, JSON proposal normalization into typed commands, ownership/risk/approval gates, freeze/reclaim controls, and runtime diagnostics surfaced through tests and UI rendering coverage.

This pass does not verify a live LLM-backed provider, streaming provider UI, provider credentials, or GUI click-through in the packaged desktop app.

## Setup

- Branch/worktree start: `main`, clean at task start.
- Rust planner/provider path: `src-tauri/src/application/agent_planner.rs`.
- Rust command safety path: `src-tauri/src/application/agent_command.rs`.
- Primary UAT fixture tests: `src-tauri/tests/agent_commands.rs`.
- Frontend proposal/diagnostic rendering tests: `src/__tests__/ConversationView.test.ts`, `src/store/session-projections.test.ts`, and `src/lib/session-client.test.ts`.
- Provider IDs observed in tests: `mock-fixture-planner`, `json-test-provider`, `local-parser`, and unavailable `offline-provider`.

## Evidence Matrix

| Scenario | Prompt or trigger | Proposed commands | Expected behavior | Observed result |
|----------|-------------------|-------------------|-------------------|-----------------|
| Bounded context packet | `derive_session_context_packet` with node/pending/history caps | N/A | Planner context includes bounded graph, scene, ownership, pending action, runtime health, and recent history summaries with truncation counts. | Passed. Context included capped nodes, scene `scene-intro`, locked/user/agent-owned node IDs, latest pending action/history, and total/included counts. |
| Realistic multi-step planner proposal | `tighten the agent voice, recall intro, then clear the agent layer` | `SetParameterValue node-agent.param-lvl=0.42`, `RecallScene scene-intro`, `RemoveNode node-agent` | Mock planner receives context, proposal normalizes to typed commands, low/medium commands apply, high-risk command waits for review. | Passed. Provider `mock-fixture-planner` received 3-node bounded context with agent ownership and non-frozen runtime. Parameter change and scene recall applied; remove-node became pending high-risk action. |
| Parameter proposal validation | `push the agent source above the allowed ceiling` | `SetParameterValue node-agent.param-lvl=1.5` | Out-of-range parameter is rejected before mutation. | Passed. Command rejected with `must be between 0 and 1`; original value remained `0.7`. |
| High-risk rejection | Reject pending remove-node from the multi-step fixture | Stored `RemoveNode node-agent` pending action | Rejection marks pending action rejected and leaves node intact. | Passed. Pending status became `Rejected`; `node-agent` remained present. |
| High-risk approval | `clear the agent layer after review` then approve pending action | `RemoveNode node-agent` | Approval applies stored typed command through the same app command path and clears pending action. | Passed. `node-agent` was removed and no pending action remained. |
| Frozen agent | Freeze, then run `tighten the agent voice, recall intro, then clear the agent layer` | Same three proposal commands | Planner context reflects frozen state; all agent-originated commands are rejected without mutation. | Passed. Context `agent_frozen` was true; all three commands were rejected with frozen-agent reasons. |
| Reclaim ownership | `reclaim_ownership(None, None)` | N/A | Agent-owned nodes transfer to user; user/shared nodes keep their existing controllers. | Passed. `node-agent` moved to `User`; user-owned and shared nodes were unchanged. |
| Provider unavailable diagnostic | `add oscillator` with unavailable `offline-provider` | N/A | Planner returns explicit unavailable error without session mutation. | Passed. Error was `planner provider 'offline-provider' is unavailable: provider is not available`. |
| Session agent unavailable diagnostic | `add oscillator` with `agent_runtime.is_available=false` | N/A | Planner returns explicit session-runtime unavailable error. | Passed. Error was `planner provider 'local-parser' is unavailable: session agent runtime is unavailable`. |
| Invalid provider response diagnostic | `bad output` with malformed JSON | N/A | Planner returns explicit invalid-output diagnostic. | Passed. Error kind was `InvalidOutput` for `json-test-provider`. |
| UI proposal review and diagnostics | Render `PendingActionCard` and `AgentRuntimeDiagnostics` | Pending parameter/scene actions | UI shows risk, affected objects, before/after copy, provider unavailable state, blocked reason, and rejected status. | Passed. Rendered markup included `Medium risk`, `Affected`, `Before`, `Provider unavailable`, `Provider token missing`, `Rejected by performer`, and `High risk`. |

## Commands Run

```sh
td start td-ab3c29
/gsd:quick td-ab3c29 Phase 10.6 verify agent orchestration UAT and update docs
npm test -- --run src/__tests__/ConversationView.test.ts src/store/session-projections.test.ts src/lib/session-client.test.ts
cargo test --manifest-path src-tauri/Cargo.toml --test agent_commands
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Notes:

- The `/gsd:quick` command was attempted because the repo instructions require a GSD entry before edits, but it is not callable from this shell environment.
- The first full `cargo test --manifest-path src-tauri/Cargo.toml` run failed only because sandboxed tests could not bind localhost UDP sockets for OSC tests (`Operation not permitted`). The suite was rerun with elevated permission and passed.

## Verification Results

- `npm test -- --run src/__tests__/ConversationView.test.ts src/store/session-projections.test.ts src/lib/session-client.test.ts`: passed, 3 files, 36 tests.
- `cargo test --manifest-path src-tauri/Cargo.toml --test agent_commands`: passed, 36 tests.
- `npm test`: passed, 5 files, 47 tests.
- `npm run build`: passed. Vite emitted the existing large chunk warning for a 524.55 kB JS asset.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed after elevated rerun. Full Rust suite included 90 lib tests, 36 agent command tests, 20 approval flow tests, 3 audio graph tests, 22 audio runtime tests, 11 macro tests, 18 MIDI learn tests, 8 performance tests, 5 persistence tests, 21 visual runtime tests, and 1 visual sidecar UAT test.

## Conclusion

Phase 10 is verified for deterministic/mock session-aware orchestration and local parser fallback: realistic planner-shaped proposals are normalized into typed commands and constrained by validation, ownership, risk, approval, freeze, reclaim, history, and diagnostics. The remaining release gap is a live provider-backed agent path and any packaged desktop GUI UAT for that provider path.
