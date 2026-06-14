# Phase 7.7 Real scsynth UAT Evidence

Task: `td-d38373`
Date: 2026-06-13
Machine context: macOS development workstation, repo `/Users/eggfam/dev/scrysynth`, branch `main`.

## Environment

- SuperCollider executable checked: `/Applications/SuperCollider.app/Contents/Resources/scsynth`
- Result: executable exists at the expected macOS bundle path.
- Runtime override used for app launch:

```sh
SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth
```

## Automated Checks

All required automated checks passed on 2026-06-13:

```sh
npm test
```

Result: 3 test files passed, 22 tests passed.

```sh
npm run build
```

Result: TypeScript and Vite production build passed.

```sh
cargo test --manifest-path src-tauri/Cargo.toml
```

Result: Rust test suite passed. Observed totals included 62 library tests plus integration suites for agent commands, approval flow, audio graph commands, audio runtime, macro commands, MIDI learn, performance commands, session persistence, and visual runtime.

## Tauri Dev Launch

Initial sandboxed command:

```sh
SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth npm run tauri dev
```

Result: failed in the Codex sandbox because Vite could not bind localhost:

```text
listen EPERM: operation not permitted 127.0.0.1:1420
listen EPERM: operation not permitted 127.0.0.1:1421
```

Rerun with elevated local permission:

```sh
SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth npm run tauri dev
```

Result: app launched successfully. Vite reported `http://127.0.0.1:1420/`, Cargo ran `target/debug/scrysynth`, and macOS `System Events` listed a visible `scrysynth` process.

During resumed UAT, the first real Play attempt exposed a boot race:

```text
Runtime server error during boot: scsynth did not confirm OSC /sync: OSC IO error: Connection refused (os error 61)
```

Direct `scsynth` launch with the same port succeeded and printed `SuperCollider 3 server ready.` The app adapter was updated to retry boot `/sync` probes for a bounded startup window, and the boot window was increased to 15 seconds for macOS CoreAudio startup. After that fix, Play reached:

```text
Ready / Healthy
Patch patch-v1-5afec9c9e60717e8 active.
```

## Manual Runtime Checklist

Graph used: default session graph with two nodes:

- `source:main_out`
- `output:master_in`

| Check | Result |
|-------|--------|
| Start audio runtime from desktop app | Passed after boot retry fix. UI reported `Ready / Healthy` with patch `patch-v1-5afec9c9e60717e8` active. |
| Audible output from default graph | Passed by human confirmation. |
| Live parameter update from inspector or macro path | Passed by human confirmation using the selected source node inspector path. |
| Stop audio runtime | Passed on human retest after the frontend projection fix. Before the fix, Stop appeared disabled with a loading cursor while audio runtime was `ready / healthy`. |
| Panic audio runtime | Passed by human confirmation. |
| Restart after panic | Passed by human confirmation. |

## Stop Button Defect

Root cause: the backend reports an active audio patch as lifecycle `ready`, but the frontend projection only allowed Stop when lifecycle was `booting`, `running`, or `recovering`. That made the successful active-patch state look non-stoppable in the transport strip.

Fix:

- `src/store/session-projections.ts` now treats `ready` audio runtime state as stoppable.
- `ready` audio runtime state is no longer treated as startable.
- `src/store/session-projections.test.ts` includes a regression test for `ready / healthy` with an active patch.

Human retest result: passed. Stop is available in the active `ready / healthy` patch state and stops the audio runtime.

## Known Limitations After This Pass

- Phase 7 real SuperCollider execution is complete for the default source-to-output graph UAT.
- Automated tests cover compiler/resource planning, OSC sync behavior, lifecycle state transitions, diagnostics, and panic recovery seams, but they do not replace the audible SuperCollider UAT.
- The visual sidecar remains outside Phase 7 and is still not bundled.

## Latest Automated Checks

After the boot retry and Stop projection fixes:

```sh
npm test
```

Result: 3 test files passed, 23 tests passed.

```sh
npm run build
```

Result: TypeScript and Vite production build passed.

```sh
cargo test --manifest-path src-tauri/Cargo.toml
```

Result: Rust test suite passed.

## Completion Result

Phase 7 may be marked complete in `README.md`, `.planning/STATE.md`, and `.planning/ROADMAP.md`. The human pass recorded:

1. Exact command and `SCRYSYNTH_SCSYNTH_PATH` value used.
2. Graph used for the sound check.
3. Audible output result.
4. Live parameter update result.
5. Stop result.
6. Panic result.
7. Restart-after-panic result.
8. Any remaining runtime limitations.
