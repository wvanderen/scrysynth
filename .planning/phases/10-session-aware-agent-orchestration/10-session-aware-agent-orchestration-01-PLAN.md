---
phase: 10-session-aware-agent-orchestration
plan: 01
type: hardening
status: planned
created: 2026-06-21
depends_on:
  - 09-hardware-input-runtime-wiring
td_epic: td-cebec0
requirements:
  - AGNT-01R
  - AGNT-02R
  - AGNT-03R
  - UI-03F
---

# Phase 10: Session-Aware Agent Orchestration

## Goal

Agent collaboration becomes meaningfully intelligent while preserving human override.

This phase should replace the current keyword parser as the agent's main planning path with a session-aware planner that can reason over the current graph, scenes, macros, ownership, pending approvals, runtime health, and recent action history. The result must still be a bounded Scrysynth instrument workflow: the agent proposes typed app commands, the app validates and applies them, and the human can freeze, reject, or reclaim control at any time.

## Product Boundary

Canonical session state remains app-owned. Agent providers, model clients, prompts, network handles, credentials, streaming state, tool-call transcripts, retry queues, and provider-specific IDs must stay outside `SessionDocument`.

Agent proposals are not canonical truth. Only validated `TypedCommand` values that pass ownership, risk, approval, and command-specific validation can mutate the session.

## Current Baseline

- `AgentIntent`, `TypedCommand`, `PendingAction`, `RiskTier`, `ActionHistoryEntry`, and `AgentRuntimeState` are part of the Rust session contract and generated TypeScript types.
- `send_agent_message` currently calls `parse_agent_intent`, a deterministic keyword parser in `agent_command.rs`.
- Parsed commands are routed through `apply_agent_command`, ownership checks, risk classification, pending high-risk approvals, typed graph/performance command handlers, and action history logging.
- The conversation view shows messages, command previews, pending high-risk actions, freeze/unfreeze, and reclaim controls.
- Runtime health projections expose agent availability, pending count, and frozen state.
- There is no real session-aware planner, no provider abstraction, no bounded context packet, no provider diagnostics, and no UAT evidence for realistic agent proposals.

## Release Gap

The current app proves the safety scaffold but not meaningful agent collaboration. Phase 10 closes that gap by adding a provider-agnostic orchestration layer above the existing typed command boundary. The agent should be able to propose multi-step, context-aware graph and performance changes without bypassing validation or making chat the only authoring surface.

## Success Criteria

1. Agent requests are planned from a bounded session context packet that includes graph topology, nodes, routes, scenes, variations, macros, ownership, pending approvals, runtime health, and recent action summaries.
2. The planner returns structured proposals containing typed command candidates, rationale, affected object IDs, confidence, risk hints, and user-review copy.
3. Every proposal is normalized into existing `TypedCommand` variants or rejected with explainable diagnostics before any mutation is attempted.
4. All accepted commands flow through existing command handlers, ownership gates, risk tiers, pending approvals, freeze/reclaim behavior, and action history logging.
5. High-risk proposals show structured proposed changes before mutation and require explicit approval.
6. Provider unavailable, invalid response, unsafe target, stale session, and validation failure states are visible in runtime status and conversation UI.
7. Tests use deterministic/mock planner fixtures so CI does not require live network or model access.
8. UAT verifies realistic session-aware proposals, approval/rejection, freeze/reclaim, and diagnostics before Phase 10 is marked complete.

## Task Breakdown

### 10.1 Define agent orchestration contract

Task: `td-665cc6`

Define the Phase 10 contract before adding planner behavior. Cover the session context packet, proposal schema, provider settings/status, response validation, error states, risk/approval behavior, and non-goals.

Contract document: `.planning/phases/10-session-aware-agent-orchestration/10-session-aware-agent-orchestration-02-CONTRACT.md`.

Acceptance:

- A Phase 10 contract document records the context inputs, proposal outputs, command constraints, provider settings, runtime status projection, diagnostics, and non-canonical runtime resources.
- The contract states that provider/model state, credentials, streaming handles, request transcripts, and provider-specific IDs are runtime resources, not `SessionDocument` fields.
- Proposal output includes at least rationale, command candidates, affected object IDs, confidence, risk hints, validation diagnostics, and review copy.
- Error states are specified for provider unavailable, provider timeout, invalid provider response, stale context, unsupported command, unsafe target, and validation failure.
- The contract explains how the current deterministic parser is kept as a fallback or test fixture without remaining the primary collaboration path.

### 10.2 Build session context packet and planner interface

Task: `td-0e888f`

Add backend types and services that derive a bounded session context packet and route agent requests through a provider-agnostic planner interface before command validation.

Acceptance:

- The context packet summarizes graph topology, node parameters, routes, scenes, variations, macros, hardware bindings where relevant, ownership, pending actions, runtime health, and recent action history.
- The context packet is bounded, deterministic, and avoids dumping unnecessary full session JSON into a provider request.
- A planner trait/service can return structured proposal objects from either a deterministic/mock planner or a configured provider-backed planner.
- Provider unavailability and configuration errors update agent runtime status without mutating session state.
- Rust tests cover context derivation, redaction/bounding behavior, planner success, planner failure, and stale-context rejection.

### 10.3 Validate agent proposals through command and ownership gates

Task: `td-84b783`

Normalize planner output into typed app commands and keep all mutation behind the existing command, ownership, risk, approval, and history pipeline.

Acceptance:

- Planner output cannot directly mutate `SessionDocument`.
- Unsupported command kinds, unknown IDs, invalid route directions, out-of-range parameter values, frozen-agent state, and locked user-owned targets are rejected with user-readable reasons.
- Low and medium risk commands apply only after command-specific validation succeeds.
- High-risk commands create pending actions with structured proposed diffs and do not mutate until approved.
- Approval and rejection update action history and runtime projections consistently.
- Tests cover realistic multi-command proposals, partial rejection, high-risk approval, high-risk rejection, frozen-agent behavior, and stale/invalid targets.

### 10.4 Add proposal review UI and agent runtime status

Task: `td-a842b4`

Upgrade the conversation sidecar from simple command previews to structured plan review while keeping graph and performance controls co-equal.

Acceptance:

- Conversation UI shows plan summary, rationale, command list, affected objects, risk tiers, confidence, blocked/rejected reasons, and pending approvals.
- High-risk pending actions show enough proposed before/after information for a human to approve or reject without reading logs.
- Agent runtime status distinguishes available, planning, awaiting approval, frozen, unavailable, degraded, and error states.
- Freeze and reclaim controls remain visible and fast.
- UI does not encourage chat-only operation; proposed changes link back to graph/performance objects where practical.
- Frontend tests cover proposal rendering, diagnostics, pending approval display, freeze/reclaim state, and unavailable provider state.

### 10.5 Add deterministic/mock planner tests and realistic proposal fixtures

Task: `td-4a9cfe`

Create deterministic fixtures for realistic planner responses so implementation and CI can verify orchestration behavior without live provider access.

Acceptance:

- Fixtures cover add-and-route source, adjust macro-targeted parameters, recall scene, save variation, disable node, remove route/node, and mixed valid/invalid proposals.
- Rust tests cover context packet snapshots, proposal normalization, validation diagnostics, risk classification, pending actions, approval/rejection, and action history.
- Frontend tests cover conversation state updates and proposal review UI using mocked planner responses.
- Tests do not require network access, provider credentials, SuperCollider, visual sidecar, or hardware devices.

### 10.6 Verify agent orchestration UAT and update docs

Task: `td-ab3c29`

Run and document Phase 10 UAT against the session-aware planner path, then update docs only to the behavior actually verified.

Acceptance:

- Phase 10 UAT evidence records setup/provider mode, prompts, context summary, proposed commands, expected behavior, observed behavior, and pass/fail result.
- UAT verifies at least one graph mutation proposal, one parameter/macro proposal, one scene or variation proposal, one high-risk approval, one rejection, freeze behavior, and reclaim behavior.
- UAT verifies provider unavailable or invalid-response diagnostics.
- README, ROADMAP, REQUIREMENTS, and STATE are updated only to the level of behavior actually verified.

## Suggested Dependency Order

1. 10.1 orchestration contract.
2. 10.2 session context packet and planner interface.
3. 10.3 proposal normalization, validation, and safety gates.
4. 10.5 deterministic fixtures and tests, developed alongside 10.2 and 10.3.
5. 10.4 proposal review UI and runtime status.
6. 10.6 UAT and docs.

## UAT Notes

Use a deterministic/mock planner for baseline UAT so the safety and UI behavior are reproducible. If a real provider is available, run an additional provider-backed pass, but do not mark behavior release-complete unless setup, diagnostics, and fallback behavior are documented.

Recommended prompts:

- "Add a soft noise layer, route it into the existing output, and keep it under my control."
- "Make the current scene calmer by lowering energy and saving it as a variation."
- "Remove the experimental source node."
- "Recall the intro scene, but do not touch locked user nodes."
- "Try to change a locked user-owned node while the agent is frozen."

Record the context mode, proposal payload shape, accepted/rejected commands, pending action IDs, approval/rejection outcome, runtime status changes, and final session result.

## Non-Goals

- No autonomous direct mutation of raw session JSON.
- No provider-specific schema in `SessionDocument`.
- No credential persistence in session files.
- No unrestricted shell, filesystem, network, or runtime control by the agent.
- No agent-authored SuperCollider, Bevy, shader, or arbitrary code execution.
- No multiplayer or remote collaboration.
- No fine-grained ownership policy redesign beyond the existing freeze/reclaim/approval model.
- No release packaging work beyond documentation updates required by verified Phase 10 behavior.
