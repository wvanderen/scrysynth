---
phase: 01-session-core-recall
plan: 05
type: execute
wave: 1
depends_on: []
files_modified:
  - src/store/session-projections.ts
  - src/store/session-projections.test.ts
  - src/App.tsx
  - package.json
  - src-tauri/Cargo.toml
  - src-tauri/src/lib.rs
  - src-tauri/capabilities/default.json
autonomous: true
gap_closure: true
requirements: [SESS-02, SESS-03, PERS-01]

must_haves:
  truths:
    - "Clicking empty graph space clears the node inspector to show the empty state message"
    - "Save button opens a native desktop save-file dialog, not a browser prompt"
    - "Open button opens a native desktop open-file dialog, not a browser prompt"
  artifacts:
    - path: "src/store/session-projections.ts"
      provides: "deriveSelectedNode returns null when no selection"
      contains: "return null"
    - path: "src/App.tsx"
      provides: "Native file dialog handlers replacing window.prompt"
      contains: "@tauri-apps/plugin-dialog"
    - path: "src-tauri/Cargo.toml"
      provides: "Tauri dialog plugin dependency"
      contains: "tauri-plugin-dialog"
  key_links:
    - from: "src/App.tsx"
      to: "@tauri-apps/plugin-dialog"
      via: "import { save, open } from plugin-dialog"
      pattern: "from.*plugin-dialog"
    - from: "src-tauri/src/lib.rs"
      to: "tauri_plugin_dialog"
      via: "plugin registration"
      pattern: "tauri_plugin_dialog::init"
---

<objective>
Fix two behavioral gaps in the session workspace: selection clearing and native file dialogs.

Purpose: The verification report found that (1) the node inspector never shows its empty state because deriveSelectedNode falls back to the first node, and (2) save/open use browser prompts instead of native desktop file dialogs. Both prevent the Phase 1 goal from being fully achieved.

Output: A session workspace where clicking empty space clears the inspector, and save/open use OS-native file pickers.
</objective>

<execution_context>
@$HOME/.config/opencode/get-shit-done/workflows/execute-plan.md
@$HOME/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-session-core-recall/01-VERIFICATION.md
@.planning/phases/01-session-core-recall/01-session-core-recall-03-SUMMARY.md

<interfaces>
<!-- Exact code that needs changing — executor reads these FIRST -->

From src/store/session-projections.ts — deriveSelectedNode (lines 145-151):
```typescript
function deriveSelectedNode(session: SessionDocument, selectedNodeId: string | null): Node | null {
  if (!selectedNodeId) {
    return session.nodes[0] ?? null;  // BUG: should return null
  }

  return session.nodes.find((node) => node.id === selectedNodeId) ?? session.nodes[0] ?? null;  // BUG: should return null for unfound IDs
}
```

From src/App.tsx — handlers (lines 63-79):
```typescript
const handleSaveSession = () => {
  const path = window.prompt("Save session to path", DEFAULT_SAVE_PATH);
  if (!path) {
    return;
  }
  void saveSession(path);
};

const handleOpenSession = () => {
  const path = window.prompt("Open session from path", DEFAULT_SAVE_PATH);
  if (!path) {
    return;
  }
  void openSession(path);
};
```

From src-tauri/src/lib.rs — run() (line 248-281):
```rust
pub fn run() {
    let _ = write_generated_typescript_contract();

    tauri::Builder::default()
        .manage(Mutex::new(SessionStore::new_default()))
        .plugin(tauri_plugin_opener::init())
        // ... handler registration ...
```

From src-tauri/Cargo.toml — dependencies (lines 20-29):
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
ts-rs = "12"
uuid = { version = "1", features = ["serde", "v4"] }
midir = "0.10"
rosc = "0.11"
```

From src-tauri/capabilities/default.json:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default"
  ]
}
```

From package.json — dependencies (relevant subset):
```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "@xyflow/react": "^12.10.2",
    "immer": "^11.1.4",
    "react": "^19.1.0",
    "react-dom": "^19.1.0",
    "zod": "^4.3.6",
    "zustand": "^5.0.12"
  }
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Fix deriveSelectedNode to return null when no node is selected</name>
  <files>src/store/session-projections.ts, src/store/session-projections.test.ts</files>
  <read_first>
    - src/store/session-projections.ts (the file being modified — contains deriveSelectedNode)
    - src/store/session-projections.test.ts (tests that may need updating for new null behavior)
  </read_first>
  <action>
    In `src/store/session-projections.ts`, fix the `deriveSelectedNode` function (lines 145-151):

    Change the function body from:
    ```typescript
    function deriveSelectedNode(session: SessionDocument, selectedNodeId: string | null): Node | null {
      if (!selectedNodeId) {
        return session.nodes[0] ?? null;
      }
      return session.nodes.find((node) => node.id === selectedNodeId) ?? session.nodes[0] ?? null;
    }
    ```

    To:
    ```typescript
    function deriveSelectedNode(session: SessionDocument, selectedNodeId: string | null): Node | null {
      if (!selectedNodeId) {
        return null;
      }
      return session.nodes.find((node) => node.id === selectedNodeId) ?? null;
    }
    ```

    Two changes:
    1. Line 147: `return session.nodes[0] ?? null` → `return null` (null selection should produce null, not first node)
    2. Line 150: `?? session.nodes[0] ?? null` → `?? null` (unfound ID should produce null, not first node)

    Then in `src/store/session-projections.test.ts`, add a new test inside the `"session projections"` describe block that verifies the null behavior explicitly. Add after the existing tests in that block:

    ```typescript
    it("deriveSelectedNode returns null when selectedNodeId is null or not found", () => {
      const session = createSession();
      const withNullId = projectSessionState(session, null);
      expect(withNullId.selectedNode).toBeNull();

      const withBadId = projectSessionState(session, "nonexistent-node-id");
      expect(withBadId.selectedNode).toBeNull();
    });
    ```

    Do NOT change any other function in session-projections.ts. Do NOT change the existing test cases — they should still pass because they explicitly select valid node IDs.
  </action>
  <verify>
    <automated>npx vitest run src/store/session-projections.test.ts 2>&1</automated>
  </verify>
  <done>
    - deriveSelectedNode returns null when selectedNodeId is null
    - deriveSelectedNode returns null when the ID is not found in session.nodes
    - New test passes: both null-input and bad-ID-input produce null selectedNode
    - All existing tests still pass (9 original + 1 new = 10)
  </done>
  <acceptance_criteria>
    - src/store/session-projections.ts line `if (!selectedNodeId)` is followed by `return null` (NOT `return session.nodes[0]`)
    - src/store/session-projections.ts the find fallback is `?? null` (NOT `?? session.nodes[0]`)
    - src/store/session-projections.test.ts contains test with description "deriveSelectedNode returns null when selectedNodeId is null or not found"
    - `npx vitest run src/store/session-projections.test.ts` exits 0
  </acceptance_criteria>
</task>

<task type="auto">
  <name>Task 2: Replace window.prompt with native Tauri file dialogs</name>
  <files>src/App.tsx, package.json, src-tauri/Cargo.toml, src-tauri/src/lib.rs, src-tauri/capabilities/default.json</files>
  <read_first>
    - src/App.tsx (contains window.prompt calls to replace)
    - src-tauri/Cargo.toml (add dialog plugin dependency)
    - src-tauri/src/lib.rs (register dialog plugin)
    - src-tauri/capabilities/default.json (add dialog permission)
    - package.json (add frontend dialog package)
  </read_first>
  <action>
    **Step 1: Add frontend dependency**
    Run: `npm install @tauri-apps/plugin-dialog`

    **Step 2: Add Rust dependency**
    In `src-tauri/Cargo.toml`, add after the `tauri-plugin-opener` line in `[dependencies]`:
    ```toml
    tauri-plugin-dialog = "2"
    ```

    **Step 3: Register plugin in lib.rs**
    In `src-tauri/src/lib.rs`, add `.plugin(tauri_plugin_dialog::init())` right after the existing `.plugin(tauri_plugin_opener::init())` line (line 254). The result should be:
    ```rust
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    ```

    **Step 4: Add dialog permission**
    In `src-tauri/capabilities/default.json`, add `"dialog:default"` to the permissions array:
    ```json
    "permissions": [
      "core:default",
      "opener:default",
      "dialog:default"
    ]
    ```

    **Step 5: Replace window.prompt in App.tsx**
    In `src/App.tsx`:

    1. Add import at the top (after the existing imports):
       ```typescript
       import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
       ```

    2. Replace the `handleSaveSession` function (lines 63-70) with:
       ```typescript
       const handleSaveSession = async () => {
         const path = await saveDialog({
           defaultPath: DEFAULT_SAVE_PATH,
           filters: [{ name: "Session", extensions: ["json"] }],
         });
         if (!path) {
           return;
         }
         void saveSession(path);
       };
       ```

    3. Replace the `handleOpenSession` function (lines 72-79) with:
       ```typescript
       const handleOpenSession = async () => {
         const result = await openDialog({
           filters: [{ name: "Session", extensions: ["json"] }],
           multiple: false,
         });
         if (!result) {
           return;
         }
         const path = typeof result === "string" ? result : result.path;
         void openSession(path);
       };
       ```

    Do NOT change any other part of App.tsx. Do NOT remove the DEFAULT_SAVE_PATH constant. Do NOT change any other handler.
  </action>
  <verify>
    <automated>npx tsc --noEmit 2>&1 | head -20 && grep -c "plugin-dialog" src/App.tsx && grep -c "tauri_plugin_dialog" src-tauri/src/lib.rs</automated>
  </verify>
  <done>
    - package.json contains @tauri-apps/plugin-dialog
    - src-tauri/Cargo.toml contains tauri-plugin-dialog = "2"
    - src-tauri/src/lib.rs has .plugin(tauri_plugin_dialog::init())
    - src-tauri/capabilities/default.json has "dialog:default" in permissions
    - App.tsx imports from @tauri-apps/plugin-dialog
    - App.tsx has zero occurrences of window.prompt
    - TypeScript compiles without errors
    - `npm run build` succeeds
  </done>
  <acceptance_criteria>
    - `grep "window.prompt" src/App.tsx` returns nothing (exit code 1)
    - `grep "@tauri-apps/plugin-dialog" src/App.tsx` returns a match
    - `grep "tauri-plugin-dialog" src-tauri/Cargo.toml` returns a match
    - `grep "tauri_plugin_dialog" src-tauri/src/lib.rs` returns a match
    - `grep "dialog:default" src-tauri/capabilities/default.json` returns a match
    - `npm run build` exits 0
  </acceptance_criteria>
</task>

</tasks>

<verification>
```bash
# Frontend tests pass with new null behavior
npx vitest run src/store/session-projections.test.ts

# TypeScript compiles
npx tsc --noEmit

# Production build succeeds
npm run build

# No window.prompt remains
grep "window.prompt" src/App.tsx
# Expected: exit code 1 (no matches)

# Dialog plugin registered
grep "tauri_plugin_dialog" src-tauri/src/lib.rs
# Expected: 1 match
```
</verification>

<success_criteria>
- deriveSelectedNode returns null for null input and unfound IDs (not first-node fallback)
- Test explicitly verifies null selection behavior
- Save and Open buttons trigger native OS file dialogs instead of browser prompts
- No window.prompt calls remain in App.tsx
- All existing tests pass
- Production build succeeds
</success_criteria>

<output>
After completion, create `.planning/phases/01-session-core-recall/01-session-core-recall-05-SUMMARY.md`
</output>
