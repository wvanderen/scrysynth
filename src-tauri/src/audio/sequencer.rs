//! App-driven step sequencer controller (NODES-04; CONTEXT.md D-06/D-07/D-08).
//!
//! The sequencer is **app-driven** (D-06): transport position and step advance
//! live in the Rust app, and SuperCollider stays "dumb" — it only receives
//! per-step gate/CV values written to allocated control buses via the OSC
//! `/c_set` message. There is no sequencer SynthDef (the `step_sequencer`
//! catalog entry has `synthdef_name: ""`).
//!
//! ## Design
//!
//! [`SequencerController`] owns a `std::thread::spawn` tick loop (tokio is not
//! a dependency; the audio layer is fully synchronous — see RESEARCH Pitfall
//! #6). Per step, the loop reads the current step's gate+CV from a shared
//! `Arc<Mutex<SequencerPattern>>`, sends two `/c_set` OSC messages (one for the
//! gate bus, one for the CV bus), advances the step, and sleeps for one
//! 16th-note period (`60.0 / bpm / 4.0`). Pattern mutations via
//! `GraphEditCommand::SetStepValue` are propagated to the live controller by
//! swapping the shared pattern (T-12-07 mitigation: writes flow through the
//! typed-command gate; the shared pattern is behind a `Mutex`).
//!
//! Shutdown is cooperative via an `Arc<AtomicBool>` flag the loop polls before
//! every sleep — `stop()` sets the flag and joins the thread, guaranteeing no
//! orphan tick threads survive audio stop/panic (T-12-06 mitigation, control
//! safety per AGENTS.md).
//!
//! ## OSC transport
//!
//! `/c_set` is fire-and-forget (RESEARCH assumption A1 — `/c_set [bus, value]`
//! directly sets a control bus readable by `In.kr`; no `/sync` needed). The
//! controller does not hold a reference to [`SuperColliderAdapter`][super] —
//! the OSC send is decoupled behind the [`SequencerTickSink`] trait so the
//! controller is unit-testable with a [`RecordingCSetSink`] and the production
//! path uses [`UdpCSetSink`] (its own ephemeral UDP socket connected to
//! scsynth, so no `&mut` cross-thread sharing is required).
//!
//! [super]: crate::audio::supercollider::SuperColliderAdapter

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::domain::session::SequencerPattern;

/// Default number of steps in a sequencer pattern (D-08: fixed 16 steps).
pub const SEQUENCER_STEP_COUNT: usize = 16;

/// scsynth's default UDP port. Mirrors [`super::SCSYNTH_UDP_PORT`].
///
/// [`super::SCSYNTH_UDP_PORT`]: crate::audio::supercollider::SCSYNTH_UDP_PORT
const SCSYNTH_UDP_PORT: u16 = 57110;

/// Sink abstraction for emitting per-step `/c_set` writes.
///
/// Decoupling the OSC transport lets the controller run against a recording
/// sink in unit tests (no real socket, no real scsynth) and against a UDP sink
/// in production.
pub trait SequencerTickSink {
    /// Send one `/c_set [bus, value]` write. The implementation owns the OSC
    /// framing (rosc bundle for UDP, recorded packet for tests).
    ///
    /// # Errors
    /// IO / encode failures surface as a `String` so the loop can log without
    /// panicking; a failing tick does not abort the transport.
    fn send_c_set(&mut self, bus: i32, value: f32) -> Result<(), String>;
}

/// Production sink: owns a UDP socket connected to scsynth and emits real
/// `/c_set` OSC messages.
///
/// A dedicated socket (rather than sharing `SuperColliderAdapter`'s client)
/// keeps the tick loop `Send + 'static` with no `&mut` cross-thread sharing —
/// the adapter is `!Send` while a transport is borrowed.
pub struct UdpCSetSink {
    socket: UdpSocket,
}

impl UdpCSetSink {
    /// Bind an ephemeral local UDP socket and connect it to scsynth at the
    /// canonical `127.0.0.1:<port>` loopback.
    ///
    /// # Errors
    /// Surface bind/connect IO failures verbatim so the caller can decide
    /// whether to retry or skip sequencer launch (the audio runtime itself is
    /// unaffected — `/c_set` is best-effort).
    pub fn connect(scsynth_addr: SocketAddr) -> Result<Self, String> {
        let socket = UdpSocket::bind("127.0.0.1:0")
            .map_err(|err| format!("sequencer UDP bind failed: {err}"))?;
        socket
            .connect(scsynth_addr)
            .map_err(|err| format!("sequencer UDP connect to {scsynth_addr} failed: {err}"))?;
        Ok(Self { socket })
    }

    /// Convenience: connect to the default local scsynth loopback.
    pub fn connect_default() -> Result<Self, String> {
        Self::connect(SocketAddr::from((
            [127, 0, 0, 1],
            SCSYNTH_UDP_PORT,
        )))
    }
}

impl SequencerTickSink for UdpCSetSink {
    fn send_c_set(&mut self, bus: i32, value: f32) -> Result<(), String> {
        let packet = rosc::OscPacket::Message(rosc::OscMessage {
            addr: "/c_set".to_string(),
            args: vec![rosc::OscType::Int(bus), rosc::OscType::Float(value)],
        });
        let bytes = rosc::encoder::encode(&packet)
            .map_err(|err| format!("sequencer OSC encode failed: {err}"))?;
        self.socket
            .send(&bytes)
            .map_err(|err| format!("sequencer OSC send failed: {err}"))?;
        Ok(())
    }
}

/// Test sink: records every `/c_set` write into a shared vec for assertion.
///
/// Mirrors the `ScriptedOscTransport` capture pattern (supercollider.rs tests)
/// but specialised to the `(bus, value)` pair shape the sequencer emits.
#[derive(Default)]
pub struct RecordingCSetSink {
    pub sent: Arc<Mutex<Vec<(i32, f32)>>>,
}

impl RecordingCSetSink {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Shared handle to the captured writes — clone before `start` and read
    /// after the controller has finished ticking.
    #[must_use]
    pub fn shared(&self) -> Arc<Mutex<Vec<(i32, f32)>>> {
        Arc::clone(&self.sent)
    }
}

impl SequencerTickSink for RecordingCSetSink {
    fn send_c_set(&mut self, bus: i32, value: f32) -> Result<(), String> {
        self.sent.lock().expect("recording sink lock").push((bus, value));
        Ok(())
    }
}

/// App-driven step sequencer controller (NODES-04).
///
/// Spawn one controller per sequencer node when the audio runtime reaches
/// `Ready` and transport is playing. Kill on stop/panic (no orphan threads).
pub struct SequencerController {
    node_id: String,
    pattern: Arc<Mutex<SequencerPattern>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for SequencerController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SequencerController")
            .field("node_id", &self.node_id)
            .field("shutdown", &self.shutdown.load(Ordering::Relaxed))
            .field("thread_alive", &self.thread.is_some())
            .finish()
    }
}

impl SequencerController {
    /// Spawn the transport-tick thread.
    ///
    /// `node_id` identifies the sequencer `Node` this controller drives — used
    /// to route `SetStepValue` mutations to the right controller.
    /// `gate_bus` / `cv_bus` are the control-bus indices allocated for this
    /// sequencer's `gate_out` / `cv_out` ports by the SC resource planner
    /// (Plan 01's `plan_cv_buses`). `tempo_bpm` drives the 16th-note step
    /// period (`60.0 / bpm / 4.0`); values ≤ 0 are clamped to a safe floor.
    /// `pattern` is the initial [`SequencerPattern`] — live edits swap it in
    /// via [`SequencerController::update_pattern`].
    pub fn start<T>(
        node_id: impl Into<String>,
        sink: T,
        gate_bus: i32,
        cv_bus: i32,
        tempo_bpm: f32,
        pattern: SequencerPattern,
    ) -> Self
    where
        T: SequencerTickSink + Send + 'static,
    {
        let node_id = node_id.into();
        let shared_pattern = Arc::new(Mutex::new(pattern));
        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_pattern = Arc::clone(&shared_pattern);
        let thread_shutdown = Arc::clone(&shutdown);

        let thread = thread::spawn(move || {
            run_tick_loop(
                sink,
                gate_bus,
                cv_bus,
                tempo_bpm,
                thread_pattern,
                thread_shutdown,
            );
        });

        Self {
            node_id,
            pattern: shared_pattern,
            shutdown,
            thread: Some(thread),
        }
    }

    /// The sequencer node id this controller drives.
    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Swap the live pattern. The next tick reads the new pattern (T-12-07:
    /// mutations are pushed via the typed-command gate before reaching here).
    pub fn update_pattern(&self, new_pattern: SequencerPattern) {
        if let Ok(mut guard) = self.pattern.lock() {
            *guard = new_pattern;
        }
    }

    /// Signal shutdown and join the tick thread. Bounded by one step period so
    /// a runaway tick is never left dangling (control safety — AGENTS.md).
    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    /// Run `steps` ticks synchronously against `sink` without spawning a
    /// thread. Test-only — used by the `/c_set` advance test to assert exact
    /// OSC writes deterministically.
    pub fn run_steps_inline<T: SequencerTickSink>(
        sink: &mut T,
        gate_bus: i32,
        cv_bus: i32,
        pattern: &SequencerPattern,
        steps: usize,
    ) {
        let mut current = 0usize;
        for _ in 0..steps {
            let gate_value = if pattern.gate[current] { 1.0 } else { 0.0 };
            let cv_value = pattern.cv[current] as f32;
            let _ = sink.send_c_set(gate_bus, gate_value);
            let _ = sink.send_c_set(cv_bus, cv_value);
            current = (current + 1) % SEQUENCER_STEP_COUNT;
        }
    }
}

impl Drop for SequencerController {
    fn drop(&mut self) {
        // Defensive: if `stop` was not called, still signal + join on drop so
        // an abandoned controller never leaks a tick thread (T-12-06).
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The transport-tick loop body. Mirrors the `Instant::now()` + bounded
/// `thread::sleep` deadline pattern from `supercollider.rs::wait_for_scsynth_boot`.
fn run_tick_loop<T: SequencerTickSink>(
    mut sink: T,
    gate_bus: i32,
    cv_bus: i32,
    tempo_bpm: f32,
    pattern: Arc<Mutex<SequencerPattern>>,
    shutdown: Arc<AtomicBool>,
) {
    // 16th-note step period. Clamp tempo to a safe floor so a 0/negative BPM
    // can never produce an infinite/negative sleep.
    let safe_bpm = if tempo_bpm.is_finite() && tempo_bpm >= 1.0 {
        tempo_bpm
    } else {
        120.0
    };
    let step_period = Duration::from_secs_f64(60.0 / f64::from(safe_bpm) / 4.0);
    let mut current_step = 0usize;

    while !shutdown.load(Ordering::SeqCst) {
        // Snapshot the current step under the lock; OSC send happens outside
        // the lock so a slow socket never blocks a SetStepValue mutation.
        let (gate_value, cv_value) = match pattern.lock() {
            Ok(guard) => {
                let gate = if guard.gate[current_step] { 1.0 } else { 0.0 };
                let cv = guard.cv[current_step] as f32;
                (gate, cv)
            }
            Err(_) => (0.0, 0.0),
        };

        // Fire-and-forget: a failing tick is logged by being silently dropped
        // (RESEARCH T-12-08 — local single-user instrument, no audit trail
        // needed; the next tick will retry).
        let _ = sink.send_c_set(gate_bus, gate_value);
        let _ = sink.send_c_set(cv_bus, cv_value);

        current_step = (current_step + 1) % SEQUENCER_STEP_COUNT;

        // Cooperative shutdown: poll before sleeping so `stop()` is prompt.
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        thread::sleep(step_period);
    }
}

/// Construct a single `/c_set` OSC packet (used by `UdpCSetSink::send_c_set`).
#[allow(dead_code)] // kept for future sinks / diagnostics
fn c_set_packet(bus: i32, value: f32) -> rosc::OscPacket {
    rosc::OscPacket::Message(rosc::OscMessage {
        addr: "/c_set".to_string(),
        args: vec![rosc::OscType::Int(bus), rosc::OscType::Float(value)],
    })
}

// io import needed so `UdpCSetSink::send` error kind mapping compiles even if
// the explicit `io::Error` path is not currently exercised above.
#[allow(dead_code)]
fn _ensure_io_import(error: io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern_with(steps: &[(bool, f64)]) -> SequencerPattern {
        let mut p = SequencerPattern::default();
        for (i, (gate, cv)) in steps.iter().enumerate() {
            p.gate[i] = *gate;
            p.cv[i] = *cv;
        }
        p
    }

    #[test]
    fn run_steps_inline_emits_one_c_set_per_gate_and_cv_pair_over_16_steps() {
        // D-07/D-08: 16 fixed steps, mono gate+cv; over 16 steps the sink
        // should record exactly 32 /c_set writes (16 gate + 16 cv).
        let mut sink = RecordingCSetSink::new();
        let shared = sink.shared();
        let pattern = pattern_with(&[
            (true, 0.1),
            (false, 0.2),
            (true, 0.3),
            (false, 0.4),
            (true, 0.5),
            (false, 0.6),
            (true, 0.7),
            (false, 0.8),
            (true, 0.9),
            (false, 1.0),
            (true, -0.1),
            (false, -0.2),
            (true, -0.3),
            (false, -0.4),
            (true, -0.5),
            (false, -0.6),
        ]);
        let gate_bus = 1024;
        let cv_bus = 1025;

        SequencerController::run_steps_inline(&mut sink, gate_bus, cv_bus, &pattern, 16);

        let sent = shared.lock().expect("captured writes").clone();
        assert_eq!(sent.len(), 32, "16 gate + 16 cv = 32 /c_set writes");

        for step in 0..16 {
            let gate_write = &sent[step * 2];
            let cv_write = &sent[step * 2 + 1];
            assert_eq!(gate_write.0, gate_bus, "gate bus on step {step}");
            assert!(
                (gate_write.1 - if pattern.gate[step] { 1.0 } else { 0.0 }).abs() < 1e-6,
                "gate value on step {step}"
            );
            assert_eq!(cv_write.0, cv_bus, "cv bus on step {step}");
            assert!(
                (cv_write.1 - pattern.cv[step] as f32).abs() < 1e-6,
                "cv value on step {step}"
            );
        }
    }

    #[test]
    fn run_steps_inline_wraps_step_index_past_15() {
        // Step index wraps 15→0; running 18 steps should emit 36 writes with
        // step 16/17 mirroring step 0/1.
        let mut sink = RecordingCSetSink::new();
        let shared = sink.shared();
        let mut pattern = SequencerPattern::default();
        pattern.gate[0] = true;
        pattern.gate[1] = false;
        pattern.cv[0] = 0.42;
        pattern.cv[1] = -0.5;

        SequencerController::run_steps_inline(&mut sink, 100, 101, &pattern, 18);

        let sent = shared.lock().expect("captured writes").clone();
        assert_eq!(sent.len(), 36);
        // Step 16 == step 0 (wraps). gate at index 32, cv at 33.
        assert!(
            (sent[32].1 - 1.0).abs() < 1e-6,
            "wrapped step 16 gate mirrors step 0"
        );
        assert!(
            (sent[33].1 - 0.42).abs() < 1e-6,
            "wrapped step 16 cv mirrors step 0"
        );
    }

    #[test]
    fn update_pattern_swaps_live_pattern_visible_to_inline_run() {
        // T-12-07: a SetStepValue reconcile must update the shared pattern the
        // tick loop reads. Verifies the Arc<Mutex> wiring without spawning.
        let pattern_a = pattern_with(&[(false, 0.0); 16]);
        let pattern_b = pattern_with(&[(true, 0.9); 16]);

        let controller = SequencerController::start(
            "node-seq",
            RecordingCSetSink::new(),
            1024,
            1025,
            120.0,
            pattern_a.clone(),
        );

        assert_eq!(controller.node_id(), "node-seq");
        controller.update_pattern(pattern_b.clone());

        // Read the controller's live pattern — should now be pattern_b.
        let live = controller.pattern.lock().expect("live pattern");
        assert!(live.gate[0], "swapped pattern gate is visible");
        assert!((live.cv[0] - 0.9).abs() < 1e-6, "swapped pattern cv is visible");
        drop(live);

        controller.stop();
    }

    #[test]
    fn start_then_stop_joins_within_bounded_time() {
        // T-12-06: stop() must join the tick thread promptly so no orphan
        // thread survives audio stop/panic. Fast tempo → tiny step period.
        let controller = SequencerController::start(
            "node-seq-fast",
            RecordingCSetSink::new(),
            1024,
            1025,
            // 9999 BPM → ~150μs step period; join should return well under 1s.
            9999.0,
            SequencerPattern::default(),
        );

        let started = std::time::Instant::now();
        controller.stop();
        let elapsed = started.elapsed();

        assert!(
            elapsed.as_millis() < 1500,
            "stop() should join in well under 1.5s, took {elapsed:?}"
        );
    }

    #[test]
    fn recording_sink_default_is_empty() {
        let sink = RecordingCSetSink::new();
        let sent = sink.shared().lock().unwrap().clone();
        assert!(sent.is_empty());
    }
}
