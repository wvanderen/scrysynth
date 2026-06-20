# Cockpit UI System

Scrysynth uses a neutral dark cockpit palette for the app shell and shared UI primitives. Green and amber are semantic state colors, not base theme colors.

## Tokens

Base tokens live in `src/App.css` under `:root`.

- Surfaces: `--surface-canvas`, `--surface-shell`, `--surface-panel`, `--surface-panel-solid`, `--surface-panel-raised`, `--surface-control`.
- Borders: `--border-subtle`, `--border-strong`, `--border-focus`.
- Text: `--text-primary`, `--text-secondary`, `--text-muted`, `--text-danger`.
- Semantic accents: `--accent-select` for selection/focus, `--accent-health` for healthy/ready, `--accent-warning` for warning/pending, `--accent-danger` for panic/error/destructive, `--accent-agent` for agent/shared-control identity.
- Density: `--radius-panel`, `--radius-card`, `--radius-control`, `--radius-pill`, plus compact spacing tokens `--space-1` through `--space-5`.

## Shared Classes

- Panels and drawers use neutral surfaces through existing region classes such as `.session-toolbar`, `.graph-panel`, `.inspector-panel`, `.conversation-view`, `.performance-view`, `.activity-panel`, `.transport-strip`, and `.primitive-palette`.
- New generic regions should use `.cockpit-panel`, `.cockpit-rail`, or `.cockpit-drawer` when there is not already a more specific component class.
- Compact commands use `.compact-button`; destructive commands compose `.destructive-button`.
- Dense horizontal controls use `.dense-control-row`.
- Status indicators use `.status-dot` with `.status-dot-green`, `.status-dot-yellow`, `.status-dot-red`, or `.status-dot-gray`.
- Badges reserve color by meaning: `.actor-user` and `.badge-user` for selection/performer blue, `.actor-agent`, `.badge-agent`, and `.badge-shared` for sparse agent/shared-control violet, `.risk-low` for green, `.risk-medium` for amber, and `.risk-high` for red.

## Replacement Targets

The cockpit refactor should replace remaining inline style islands with shared classes before expanding major regions:

- `src/components/workspace/MacroSlider.tsx`: card surface, label colors, and range accent still use hard-coded legacy values.
- `src/components/workspace/MacroEditor.tsx`: editor inputs, popover surfaces, and mapping rows still use hard-coded legacy values.
- `src/components/workspace/MidiLearnOverlay.tsx`: overlay uses fallback custom properties and local hard-coded color values that should map to the cockpit tokens.
