---
phase: 10-session-aware-agent-orchestration
plan: 02
type: contract
status: drafted
created: 2026-06-21
td_task: td-665cc6
---

# Phase 10 Agent Orchestration Contract

## Purpose

Phase 10 adds a session-aware planning path for the agent without moving authorship out of the Scrysynth app. The planner can read a bounded context packet and return structured proposals, but only app-owned typed commands that pass validation, ownership, risk, approval, and history gates can mutate the canonical `SessionDocument`.

The current deterministic parser remains useful as a fallback and test fixture. It must not remain the primary collaboration path once the planner interface is available.

## Canonical Boundary

Canonical session state remains the data already represented by `SessionDocument`: transport, audio runtime state, graph nodes, routes, buses, macros, scenes, variations, ownership, runtime status refs, visual runtime state, agent runtime state, freeze state, pending actions, action history, and hardware bindings.

The following are runtime resources and must not become `SessionDocument` fields:

- Provider credentials, API keys, OAuth tokens, and account identifiers.
- Provider-specific request ids, conversation ids, thread ids, tool-call ids, trace ids, and retry ids.
- Model clients, streaming handles, network sockets, retry queues, prompt caches, token accounting details, and raw provider transcripts.
- Untrusted provider response bodies beyond the normalized proposal and diagnostics needed for review.
- Generated runtime code, SuperCollider snippets, Bevy scene code, shaders, shell commands, filesystem paths outside approved app resources, and arbitrary tool handles.

Runtime resources may be supervised by services outside the session document and projected into status or diagnostics as bounded strings, enums, counts, and timestamps.

## Session Context Packet

The planner receives a bounded, deterministic context packet derived from the current session snapshot and request metadata. It should be small enough to review in tests and stable enough for snapshot coverage.

Required context inputs:

- `session_ref`: session id, schema version, title, updated timestamp, and a context revision or hash.
- `request`: user text, actor id, correlation id, request timestamp, and optional selected object ids from the UI.
- `transport`: tempo, play state, and beat position.
- `graph`: nodes with id, type, enabled state, ports, exposed parameters with min/max/unit/current/default values, audio primitive kind, runtime target summary, scene membership, ownership controller, and lock state.
- `routes`: route ids, source/target node ids, source/target port ids, and bus id.
- `buses`: bus ids, names, channel count, type, and enabled state.
- `macros`: macro ids, names, range, legacy target parameter ids, and structured audio/visual targets.
- `scenes` and `variations`: ids, names, active node ids, macro overrides, scene ids, and parameter override counts or bounded details.
- `ownership`: node ownership and locks, plus ownership rules relevant to the agent runtime.
- `pending_actions`: pending action ids, command kind, risk tier, affected ids, status, and age.
- `runtime_health`: audio, visual, hardware, and agent availability summarized from runtime state and `RuntimeStatusRef`.
- `recent_history`: the most recent bounded action history entries with actor, command kind, affected ids, and diff descriptions.

Bounding rules:

- Do not send full session JSON by default.
- Limit arrays by deterministic ordering and explicit caps. If caps truncate data, include counts and a `truncated` diagnostic.
- Include only the fields needed for planning, validation explanation, and user review.
- Redact secrets and runtime-only handles before the packet is created.
- Include a context revision/hash in every proposal response so stale responses can be rejected.

## Planner Interface

The app-owned orchestration service calls a provider-agnostic planner interface. Implementations may be deterministic/mock, local, or provider-backed, but they must all return the same normalized envelope.

Required request fields:

- `context`: the session context packet.
- `user_prompt`: the user's current request.
- `actor`: the agent actor and correlation id that will be used if commands are accepted.
- `planner_mode`: deterministic, mock fixture, local provider, or remote provider.
- `constraints`: supported command kinds, max command count, risk policy, and whether high-risk commands must be proposed only.

Required response fields:

- `proposal_id`: app-generated id or validated provider id scoped to this request.
- `context_revision`: the revision/hash from the context packet.
- `summary`: one-line human-readable plan summary.
- `rationale`: concise explanation of why the commands satisfy the request.
- `commands`: ordered command candidates, each with a command kind, candidate payload, affected object ids, confidence, risk hint, review copy, and optional proposed diff.
- `diagnostics`: validation or provider diagnostics known before command execution.
- `status`: proposed, partially_valid, rejected, provider_error, or unavailable.

Proposal command candidates are not canonical mutations. They are normalized into existing `TypedCommand` variants, then routed through existing handlers.

## Proposal Output Shape

Each command candidate must include:

- `candidate_id`: stable within the proposal.
- `kind`: supported command family, currently graph edit or performance.
- `typed_command`: normalized `TypedCommand` candidate when available.
- `affected_node_ids`, plus affected route, bus, macro, scene, variation, and hardware binding ids where relevant.
- `confidence`: number from 0.0 through 1.0.
- `risk_hint`: low, medium, or high from the planner, treated as advisory only.
- `review_copy`: a human-readable description suitable for the conversation sidecar.
- `proposed_diff`: before/after text or structured summary when the command changes existing state.
- `validation`: accepted, blocked, or needs_approval, with diagnostics.

The app computes final risk with `classify_risk` or its successor. The planner's risk hint can raise attention but cannot lower app-computed risk or bypass approval.

## Command Constraints

Accepted proposals must normalize into current app command boundaries:

- `TypedCommand::GraphEdit` for `AddNode`, `RemoveNode`, `SetNodeEnabled`, `SetParameterValue`, `AddRoute`, `RemoveRoute`, `AssignNodeToBus`, and `ClearNodeBusAssignment`.
- `TypedCommand::Performance` for `RecallScene`, `SaveVariation`, and `RestoreVariation`.

Validation must reject:

- Unknown command kinds or fields.
- Unknown node, port, route, bus, macro, scene, variation, parameter, or hardware binding ids.
- Invalid route direction, signal mismatch, missing ports, duplicate route ids, or impossible bus assignment.
- Parameter values outside min/max ranges.
- Commands targeting locked nodes or user-owned nodes from an agent actor.
- Agent commands while the agent is frozen.
- Responses whose context revision does not match the current session.
- Attempts to mutate raw `SessionDocument`, runtime resources, files, processes, network state, or external runtimes directly.

The app may apply valid low/medium-risk commands immediately after command-specific validation. High-risk commands must become pending actions and must not mutate until approved.

## Risk And Approval Behavior

Risk classification remains app-owned:

- Low risk: bounded parameter changes and variation restore behavior that command validation accepts.
- Medium risk: additive graph changes, node enablement, routing, bus assignment, and scene recall.
- High risk: destructive graph changes, route removal, node removal, and bus assignment clearing.

Approval behavior:

- Low and medium commands can apply immediately when ownership and command validation pass.
- High-risk commands create `PendingAction` records with status `pending`.
- Pending action review must show risk, affected ids, review copy, and proposed diff when available.
- Approval applies the stored typed command through the same command handler path and records action history.
- Rejection marks or removes the pending action according to the existing model and records a user-visible outcome.
- Freeze prevents agent-originated commands from applying or being newly approved as agent actions until unfrozen.
- Reclaim transfers agent-owned targets back to user or selected controller without relying on provider state.

## Provider Settings And Runtime Status

Provider configuration is runtime-owned. It may include provider kind, model name, endpoint selection, timeout, max response size, fixture name, and whether network-backed planning is enabled. Secrets and credentials must be stored outside session files.

The UI and store need an agent runtime projection that can distinguish at least:

- `available`: planner can accept requests.
- `planning`: a request is in flight.
- `awaiting_approval`: one or more pending actions exist.
- `frozen`: human has frozen the agent.
- `unavailable`: planner is not configured or provider is disabled.
- `degraded`: planner fallback is active or provider returned recoverable issues.
- `error`: last planning request failed.

Phase 10 may extend the current `AgentRuntimeState` beyond `is_available`, `pending_action_count`, and `is_frozen`, but any provider-specific detail must remain a bounded diagnostic projection rather than canonical provider state.

## Diagnostics And Error States

Every failure path should produce a user-readable diagnostic and a stable code for tests.

Required diagnostic codes:

- `provider_unavailable`: no configured planner/provider can accept the request.
- `provider_timeout`: the planner exceeded the configured timeout.
- `invalid_provider_response`: response was malformed, too large, missing required fields, or failed schema validation.
- `stale_context`: response context revision does not match the current session.
- `unsupported_command`: proposal uses a command kind outside the app contract.
- `unsafe_target`: proposal targets locked, user-owned, missing, or otherwise disallowed objects.
- `validation_failure`: normalized command failed graph, performance, ownership, range, or route validation.
- `agent_frozen`: the agent is frozen and cannot propose/apply mutations.
- `partial_proposal`: some commands are valid while others are blocked.

Diagnostics must be visible in conversation UI and agent runtime status when they affect user workflow. They should also be testable without live provider access.

## Deterministic Parser Role

`parse_agent_intent` remains as:

- A compatibility fallback when the planner is unavailable and fallback is explicitly enabled.
- A deterministic fixture path for tests and local demos.
- A simple smoke path for browser preview mode until the mock planner replaces it.

It should not be treated as session-aware planning. New orchestration work should route through the planner interface first, then normalize to `TypedCommand`, then use the existing safety gates.

## Non-Goals

- No autonomous direct mutation of raw session JSON.
- No provider-specific schema or credentials in `SessionDocument`.
- No unrestricted shell, filesystem, network, SuperCollider, Bevy, shader, or code execution by the agent.
- No chat-only authoring mode that hides graph, performance, ownership, or approval surfaces.
- No multiplayer, remote collaboration, or CRDT ownership redesign in Phase 10.
- No replacement of existing typed command handlers with provider-authored behavior.
