---
phase: 09-hardware-input-runtime-wiring
type: uat-evidence
status: passed
task: td-8062dd
verified_at: 2026-06-19T16:09:55-05:00
---

# Phase 9 Hardware Input Runtime UAT

## Scope

This pass verified the app-owned hardware runtime path behind the Tauri desktop workspace:

- MIDI and OSC listener settings are applied through `SessionStore`, the same state owner used by the Tauri commands.
- A live CoreMIDI virtual source can be selected, listened to, learned, and routed after learn mode ends.
- A live local OSC sender can be listened to, learned, and routed after learn mode ends.
- Learned bindings drive macro updates, scene recall, transport play, transport stop, and panic.
- Panic from a learned hardware binding stops an active SuperCollider patch and leaves the audio runtime in a restartable panic-recovered state.

This was not a separate frontend click-through automation pass. `npm run tauri dev` launched the dev process, but this environment could not expose a reliable accessibility window for Computer Use to click. The behavioral evidence below comes from a scratch UAT runner linked against the Tauri app crate and exercising the app runtime/command-layer state directly with live MIDI, OSC, and SuperCollider resources.

## Local Setup

Repository: `/Users/eggfam/dev/scrysynth`

Hardware sources:

- MIDI: CoreMIDI virtual output named `Scrysynth Phase 9 UAT Virtual MIDI`, created with `midir::os::unix::VirtualOutput`.
- OSC: UDP sender targeting `127.0.0.1`; final pass used listen port `55970`.
- Audio runtime: `/Applications/SuperCollider.app/Contents/Resources/scsynth`.

Scratch runner:

- Location: `/private/tmp/scrysynth-phase9-uat`
- Dependencies: local `scrysynth` crate, `midir` 0.10, `rosc` 0.11.
- The runner creates the MIDI source, reserves a free OSC port, selects the virtual MIDI input through hardware settings, starts MIDI and OSC listeners, performs learn operations, sends live values outside learn mode, and asserts the resulting session/runtime state.

## Command

The first sandboxed run compiled but failed with `Error: InitError` while initializing CoreMIDI. The successful UAT was rerun outside the sandbox:

```sh
CARGO_TARGET_DIR=/Users/eggfam/dev/scrysynth/src-tauri/target \
SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth \
cargo run --manifest-path /private/tmp/scrysynth-phase9-uat/Cargo.toml --offline
```

Observed tool output:

```text
MIDI input ports:
  midi-input-0 => Scrysynth Phase 9 UAT Virtual MIDI
Hardware listeners: MIDI=Listening selected=Some("Scrysynth Phase 9 UAT Virtual MIDI"); OSC=Listening 127.0.0.1:55970
PASS learn MIDI CC ch0 #7 -> macro energy: MidiCc { channel: 0, controller: 7 } -> Macro { macro_id: "8b8daf54-00e3-4e76-a8a3-67b2408136af" }
PASS MIDI macro live route: level=0.252
PASS learn MIDI CC ch0 #8 -> transport stop: MidiCc { channel: 0, controller: 8 } -> TransportStop
PASS learn OSC /scrysynth/uat/energy -> macro energy: OscAddress { address: "/scrysynth/uat/energy" } -> Macro { macro_id: "8b8daf54-00e3-4e76-a8a3-67b2408136af" }
PASS OSC macro live route: level=0.420
PASS learn OSC /scrysynth/uat/scene -> intro scene: OscAddress { address: "/scrysynth/uat/scene" } -> SceneRecall { scene_id: "c9d318ed-3401-4d65-8b19-e5b303d1f9c3" }
PASS learn OSC /scrysynth/uat/play -> transport play: OscAddress { address: "/scrysynth/uat/play" } -> TransportPlay
PASS OSC transport play: transport=true audio=Ready/Healthy patch=Some("patch-v1-be42ba9bd41741b2") error=None
PASS MIDI transport stop: transport=false audio=Idle/Unknown
PASS learn OSC /scrysynth/uat/panic -> transport panic: OscAddress { address: "/scrysynth/uat/panic" } -> TransportPanic
PASS OSC panic: transport=false audio=Idle/PanicRecovered panic_count=1 visual=Panicked/Degraded
PASS OSC scene recall: active_scene=Some("c9d318ed-3401-4d65-8b19-e5b303d1f9c3") level=0.650
Final binding count: 6
```

Result: passed.

## Verification Matrix

| Area | Expected behavior | Observed behavior | Result |
|------|-------------------|-------------------|--------|
| Listener setup | App runtime can select the virtual MIDI source and start both MIDI and OSC listeners. | `midi-input-0` mapped to `Scrysynth Phase 9 UAT Virtual MIDI`; MIDI and OSC both reached `Listening`. | Pass |
| MIDI macro learn | A live MIDI event can be learned for the default `energy` macro. | CC channel 0 controller 7 captured as a `Macro` binding. | Pass |
| MIDI macro live route | Moving/sending the learned CC outside learn mode updates canonical macro-targeted parameter state. | CC value 32 routed to source `level=0.252`. | Pass |
| MIDI transport learn | A live MIDI event can be learned for a transport target. | CC channel 0 controller 8 captured as `TransportStop`. | Pass |
| MIDI transport live route | Sending the learned stop CC outside learn mode stops transport/audio state. | Transport became `false`; audio runtime returned to `Idle/Unknown` with no active patch. | Pass |
| OSC macro learn | A live OSC address can be learned for the default `energy` macro. | `/scrysynth/uat/energy` captured as a `Macro` binding. | Pass |
| OSC macro live route | Sending the learned OSC address outside learn mode updates canonical parameter state. | OSC value `0.42` routed to source `level=0.420`. | Pass |
| OSC transport play learn | A live OSC address can be learned for transport play. | `/scrysynth/uat/play` captured as `TransportPlay`. | Pass |
| OSC transport play live route | Sending play outside learn mode starts transport and the real audio runtime. | Transport became `true`; SuperCollider runtime reached `Ready/Healthy` with active patch `patch-v1-be42ba9bd41741b2`. | Pass |
| OSC panic learn | A live OSC address can be learned for panic. | `/scrysynth/uat/panic` captured as `TransportPanic`. | Pass |
| OSC panic live route | Sending panic while audio is active stops runtime behavior safely. | Transport became `false`; audio became `Idle/PanicRecovered`, active patch cleared, panic recovery count incremented to `1`; visual runtime became `Panicked/Degraded`. | Pass |
| OSC scene learn | A live OSC address can be learned for scene recall. | `/scrysynth/uat/scene` captured as `SceneRecall`. | Pass |
| OSC scene live route | Sending the learned scene address outside learn mode recalls scene state. | Active visual scene set to the learned scene ID; scene node membership applied; macro override set source `level=0.650`. | Pass |

## Desktop Launch Sanity

Command:

```sh
SCRYSYNTH_SCSYNTH_PATH=/Applications/SuperCollider.app/Contents/Resources/scsynth npm run tauri dev
```

Observed result:

- Vite served `http://127.0.0.1:1420/`.
- Cargo built and launched `target/debug/scrysynth`.
- macOS `System Events` listed a visible `scrysynth` process.
- Computer Use could not target the dev-launched process because it had no bundle identifier and reported no accessible windows, so no GUI click-through evidence is claimed here.

Result: launch sanity passed; GUI automation not verified.

## Regression Verification

Commands:

```sh
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Observed result:

- `npm test` passed: 4 files, 38 tests.
- `npm run build` passed. Vite emitted a large-chunk warning for the generated production bundle.
- The first sandboxed Rust run failed because OSC tests could not bind localhost UDP sockets. Rerunning outside the sandbox passed: 89 lib tests, integration tests, `visual_sidecar_uat`, and doctests.

## Remaining Limits

- This UAT verifies the app runtime state and command path used by the desktop workspace, but not a manual click-by-click UI workflow.
- The scratch runner explicitly called `stop_hardware_learn` after each capture to leave learn mode before live routing. The frontend polling path now mirrors that behavior after it observes a newly captured binding, covered by `npm test`.
- Physical hardware was not used. The MIDI pass used a CoreMIDI virtual output, which is acceptable for Phase 9 when physical hardware is unavailable.
