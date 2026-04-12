# Audiovisual Co-Creation Instrument — Foundation Document

## 1. Purpose

This project is a **graph-native, co-creative human/AI audiovisual instrument** built as a desktop application with Tauri.

It is designed for live creation, performance, and iterative collaboration between a human performer and one or more AI agents. The system should feel like an inhabitable instrument environment rather than a DAW clone, chat wrapper, or one-shot generator.

This document defines the project foundation before implementation details harden. It exists to keep product, architecture, and UX aligned.

---

## 2. Product Distinction

### 2.1 Relationship to Mindrave

This project must remain distinct from **Mindrave**.

**Mindrave** should remain:

* terminal-native
* text-first
* algorave/live-code oriented
* lower visual density, higher symbolic density
* structured around command, code-like interaction, and terminal performance energy

**This new project** should be:

* graph-native
* GUI-first
* structured around visible signal flow and live control surfaces
* audiovisual by design
* oriented toward inhabitable patch space, performance, and co-creative manipulation

This is **not** “Mindrave with windows.”

### 2.2 Product Thesis

The system is:

* a conversation-driven instrument
* a visible session graph
* a live control environment
* a shared performance space between human and agent

A useful framing:

**Conversation conducts. Graph structures. Runtimes perform.**

---

## 3. Core Design Principles

### 3.1 Performance-Native, Not Production-Native

The application should optimize for:

* live mutation
* improvisation
* scene changes
* expressive control
* collaborative response
* variation and branching

It should not try to replace a full DAW.

### 3.2 Visible Structure Matters

If the system is built from signal flow, modulation, and control relationships, the user should be able to see and manipulate those relationships.

The graph is not decoration. It is a core part of the instrument.

### 3.3 Conversation Is Important but Not Exclusive

The user can direct the system through natural language, but the product does not live only in chat.

The app should support three equally important modes of interaction:

* conversational direction
* graph inspection/manipulation
* direct performance control

### 3.4 Shared Control Is First-Class

The system must treat control as something that can be shared, delegated, reclaimed, or negotiated between human and agent.

This is a differentiator, not implementation detail.

### 3.5 Canonical State Lives in the App

The application owns the source of truth for the session.

Audio and visual engines are runtime targets, not the primary authors of state.

### 3.6 Agents Work on Meaningful Primitives

Agents should operate on stable, meaningful building blocks rather than unrestricted arbitrary internals by default.

This makes the system:

* composable
* explainable
* debuggable
* performant
* agent-friendly

---

## 4. UX Direction

### 4.1 GUI Character

The interface should evolve according to the logic of a GUI instrument, not by porting terminal patterns into a windowed shell.

Key UX traits:

* visible signal flow
* modular spatial layout
* live control surfaces
* inspectable mappings and ownership
* clear scene/variation organization
* readable agent activity

### 4.2 Primary Interface Planes

The application should be organized around three linked views of the same session:

#### A. Conversation View

Used for:

* intent capture
* proposals
* diffs and actions
* summarizing agent behavior
* direction and refinement

#### B. Graph View

Used for:

* audio nodes
* visual nodes
* routing
* buses
* scenes/groups
* macro bindings
* modulation relationships
* ownership overlays

#### C. Performance View

Used for:

* macro controls
* scene triggers
* variation selection
* morph controls
* MIDI learn
* agent enable/disable
* safety and reset controls

### 4.3 Inspiration from Modular/Node-Based Systems

The project can borrow from tools like Bespoke in the sense that the interface should reflect the system’s internal musical structure.

However, the system should not require users to hand-patch every idea. The AI layer must be able to create, transform, and expose structure without collapsing into unintelligible complexity.

---

## 5. Runtime Strategy

## 5.1 High-Level Runtime Model

The application should use a **shared canonical session model** and expose that model to dedicated runtime adapters.

Recommended structure:

* Tauri app owns session graph and orchestration
* audio runtime adapter executes sound
* visual runtime adapter executes visuals
* event bus coordinates change, control, and synchronization

This gives the product one coherent instrument model with multiple execution backends.

### 5.2 Audio Runtime Recommendation

**SuperCollider is the recommended v1 audio runtime**, with a clear architectural constraint:

SuperCollider should be treated as the **audio execution engine**, not the total product model.

It should be used for:

* synth definitions
* effect definitions
* buses and routing
* modulation primitives
* grouping and instancing
* real-time parameter control
* performant audio execution

It should not be treated as the canonical owner of the product session.

### 5.3 SuperCollider Constraint Model

The architecture should assume:

* the app owns the session graph
* the app owns agent actions and semantics
* the SC adapter translates graph state into audio resources
* the agent works with reusable primitives and routable structures

This means the system should favor:

* robust synth/effect building blocks
* routable and reusable modules
* graph-level orchestration of instances and connections
* scene and macro-based mutation

The system should avoid relying on arbitrary unrestricted DSP graph surgery as the normal operating mode.

### 5.4 Visual Runtime Recommendation

The visual runtime should be **separate from the audio runtime**.

The project should not force SuperCollider to serve as the audiovisual master environment.

Instead:

* visuals should be driven by the same session model and event streams
* the visual runtime should be chosen for strong GUI/GPU integration within Tauri
* audio and visuals should remain coordinated through shared abstractions rather than engine-level coupling

### 5.5 Runtime Boundary Principle

Audio and visuals are parallel runtimes connected by:

* shared session state
* macros
* scenes
* modulation concepts
* event streams
* ownership and control mappings

This makes audiovisual behavior coherent without requiring a single engine to solve unrelated problems.

---

## 6. Core Product Model

The app needs a stable vocabulary of objects. These objects should be canonical before implementation gets deep.

### 6.1 Session

A session is the top-level creative container.

A session contains:

* graph state
* global tempo/time settings
* active scenes
* nodes and routes
* macros and bindings
* agent roles and permissions
* ownership rules
* runtime adapter state references
* snapshots and variations
* metadata/personality documents

### 6.2 Node

A node is a functional unit in the session graph.

Node categories may include:

* audio source
* audio effect
* audio router
* modulation source
* control/macro helper
* visual generator
* visual effect
* visual router
* analysis/event emitter

A node should expose:

* identity
* type
* ports
* parameters
* runtime target
* group/scene membership
* visibility state
* ownership metadata

### 6.3 Route

A route connects nodes or buses.

A route may represent:

* audio signal flow
* control signal flow
* modulation flow
* event/message flow
* visual pipeline flow

### 6.4 Bus

A bus is a reusable routing layer for grouping, summing, or processing signals.

Bus concepts should exist in the app model even if runtime backends implement them differently.

### 6.5 Macro

A macro is a named high-level control surface that affects one or more lower-level parameters.

A macro should support:

* label and description
* value range
* mappings to many parameters
* scaling/curves
* smoothing
* user access mode
* agent access mode
* visual display behavior
* optional MIDI binding

### 6.6 Binding

A binding maps one thing to another.

Examples:

* macro -> audio params
* macro -> visual params
* beat event -> visual pulse
* LFO node -> filter cutoff
* scene activation -> node enable state

### 6.7 Scene

A scene is a named performance state or section.

A scene may define:

* active/inactive nodes
* routing changes
* macro defaults
* visual state
* agent behavior shifts
* transition rules

Scenes should support live switching and morphing where appropriate.

### 6.8 Variation

A variation is a branch or snapshot derived from a session or scene.

Variations support:

* trying alternate routings
* replacing modules
* swapping modulation sources
* comparing musical directions
* preserving performance history

### 6.9 Ownership Rule

An ownership rule determines who can control a parameter, macro, or structure.

Ownership modes may include:

* user-only
* agent-only
* shared
* user-priority shared
* agent-priority shared
* nudge-only
* temporary delegation

Ownership rules should support:

* priority behavior
* override behavior
* recovery behavior
* automation conflict handling
* persistence across scenes/variations

### 6.10 Agent Role

An agent role defines a domain of responsibility.

Possible roles:

* conductor
* rhythm
* harmony
* texture
* bass
* transition
* visuals
* performance assistant

Roles should define:

* scope
* permissions
* target objects
* allowed mutations
* explainability expectations

---

## 7. Agent Behavior Model

### 7.1 Role of the Agent

The AI is not only a prompt responder. It is a co-creative performer and systems operator inside the session.

Agent actions may include:

* creating nodes
* routing nodes
* exposing macros
* proposing scenes
* generating variations
* taking temporary control of parameters
* reacting to user feedback
* coordinating audio and visual changes

### 7.2 Agent Operating Level

By default, the agent should operate at the level of:

* nodes
* buses
* routes
* macros
* bindings
* scenes
* variations
* ownership assignments

The system should avoid making raw low-level implementation details the default authoring surface unless explicitly needed.

### 7.3 Explainability Requirement

The app should make agent actions legible.

The user should be able to inspect:

* what changed
* why it changed
* what controls were exposed
* what is currently agent-controlled
* what can be overridden

### 7.4 Human Override

Human intervention must always be easy and reliable.

The user should be able to:

* override an agent-controlled parameter
* freeze agent changes
* reclaim a node or macro
* disable an agent role
* revert to a prior variation or snapshot

---

## 8. Runtime Adapter Contract

Before selecting implementation details, every runtime adapter should be expected to support a baseline contract.

### 8.1 Audio Adapter Must Support

* node instantiation
* node teardown
* parameter updates
* routing/bus assignment
* group management
* scene application
* macro mapping application
* event emission back to the app
* health/error reporting

### 8.2 Visual Adapter Must Support

* visual node instantiation
* parameter updates
* scene application
* event-driven response
* macro mapping application
* audiovisual synchronization hooks
* health/error reporting

### 8.3 Shared Adapter Expectations

* no adapter owns canonical creative truth
* adapters consume canonical session state
* adapters report runtime state, not semantic authorship
* adapters can be restarted or recovered without destroying the session model

---

## 9. Suggested System Architecture

### 9.1 Top-Level Layers

#### A. Application Core

Owns:

* session graph
* persistence
* agent orchestration
* ownership engine
* variation manager
* runtime coordination

#### B. UI Layer

Owns:

* conversation interface
* graph rendering
* performance controls
* inspectors
* event/activity log

#### C. Audio Adapter Layer

Owns:

* SuperCollider communication
* synth/effect resource mapping
* bus and route translation
* runtime monitoring

#### D. Visual Adapter Layer

Owns:

* visual runtime communication
* node/effect mapping
* event reactivity
* runtime monitoring

#### E. Shared Event Bus

Owns:

* user intents
* agent actions
* macro changes
* scene changes
* sync events
* runtime feedback

---

## 10. File / Project Structure Direction

Sessions should be durable, inspectable, and shareable.

A repo-like structure is preferred.

Example:

```text
/project-name
  /sessions
    /session-001
      session.json
      params.json
      SOUL.md
      SKILL.md
      AGENTS.md
      NOTES.md
      /audio
        synthdefs/
        patches/
      /visuals
        scenes/
        shaders/
      /variations
        /v1
        /v2
```

### 10.1 session.json

Should contain:

* graph state
* scenes
* routes
* node metadata
* active runtime mappings
* ownership assignments
* global session configuration

### 10.2 params.json

Should contain:

* macro definitions
* parameter metadata
* value ranges
* mapping targets
* control permissions
* MIDI binding metadata

### 10.3 SOUL.md

Should define:

* aesthetic identity
* creative temperament
* artistic behavior tendencies
* stylistic values

### 10.4 SKILL.md

Should define:

* craft heuristics
* sound/visual design patterns
* domain-specific behavior
* performance strategies

### 10.5 AGENTS.md

Should define:

* agent roster
* role boundaries
* permissions
* escalation paths
* collaboration patterns

---

## 11. v1 Scope

### 11.1 In Scope

* local single-user desktop app
* Tauri shell
* canonical session graph
* graph-native GUI foundation
* conversation view
* performance control surface
* SuperCollider audio adapter
* basic audio node/effect/bus model
* macros and bindings
* scenes
* ownership rules for controls
* one primary conductor agent
* limited variation support

### 11.2 Out of Scope for v1

* multiplayer collaboration
* full social/session marketplace
* unrestricted plugin ecosystem
* elaborate visual graph editing if it delays the audio foundation
* highly autonomous self-rewriting low-level DSP behavior
* full DAW-style production workflow
* browser-first deployment

---

## 12. Non-Goals

This product is not trying to be:

* a traditional DAW
* a pure modular patching clone
* a generic chat wrapper over synthesis tools
* an offline one-click AI music generator
* a terminal app with a graph skin

---

## 13. Open Questions

These should remain active until resolved explicitly.

1. What should the minimum visual runtime be for the first audiovisual milestone?
2. How much direct graph editing should users have in v1?
3. Should the agent target a higher-level patch DSL, direct runtime commands, or both?
4. How should scene transitions work: discrete switching, morphing, or hybrid?
5. What is the minimum set of audio node primitives needed for a compelling v1?
6. Which visual abstractions should mirror audio abstractions, and which should remain distinct?
7. How should ownership recovery behave after a user overrides an agent?
8. What agent actions require confirmation versus immediate execution?

---

## 14. Immediate Next Deliverables

### 14.1 Runtime and Session Schema Draft

Define concrete schemas/interfaces for:

* Session
* Node
* Route
* Bus
* Macro
* Binding
* Scene
* Variation
* OwnershipRule
* AgentRole
* RuntimeAdapter

### 14.2 Primitive Library Definition

Define the first supported primitive set for:

* audio source nodes
* audio effect nodes
* modulation nodes
* routing helpers
* macro/control helpers
* early visual nodes

### 14.3 UX Frame Draft

Create an initial screen architecture for:

* conversation view
* graph workspace
* performance controls
* inspectors
* event log

### 14.4 SC Adapter Spec

Write the exact responsibility boundary for the SuperCollider integration layer.

---

## 15. Working Conclusion

The project now has a strong enough foundation to move from exploratory ideation into structured design.

The key commitments are:

* distinct identity from Mindrave
* graph-native GUI direction
* canonical session state in the app
* SuperCollider as audio runtime, not total architecture
* separate visual runtime strategy
* primitive-based agent mutation
* shared control as a core product feature

These constraints are strengths. They make the system buildable, legible, and musically collaborative.
