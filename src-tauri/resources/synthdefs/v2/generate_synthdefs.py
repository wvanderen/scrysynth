#!/usr/bin/env python3
"""Generate deterministic Scrysynth v2 SuperCollider SynthDef resources.

This is a sclang-free SynthDef v2 byte writer for the catalog-driven v2 node
library. It extends `../v1/generate_synthdefs.py` (same SynthDefBuilder /
synthdef_bytes model — do NOT hand-roll the serializer) with:

  * the full synthesis chain (oscillator/noise/filter/envelope/lfo/vca/quantizer/
    mixer/output),
  * the effect family (delay/reverb/distortion/chorus/flanger),
  * per-parameter CV-bus control args read via `In.kr`/`In.ar` and summed into
    the base parameter value (NODES-05). Unconnected CV buses default to a
    silent bus index so an unwritten bus reads 0.0 (no modulation).

Mirror of `crate::catalog::entries::CATALOG` — control names here must match the
args `crate::audio::synthdefs::plan_sc_resources` emits.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct


RATE_SCALAR = 0
RATE_CONTROL = 1
RATE_AUDIO = 2

# Bus index conventions shared with `crate::audio::synthdefs`:
# control buses start at FIRST_CONTROL_BUS_OFFSET (1024); an unwritten bus reads
# 0.0 in a fresh scsynth, so defaulting CV-bus args here keeps unconnected inputs
# silent (NODES-05 wiring; compiler overrides these when a CV route exists).
SILENT_CONTROL_BUS = 1024.0
SILENT_AUDIO_BUS = 512.0  # high audio bus index, never allocated to a real signal


@dataclass(frozen=True)
class InputRef:
    ugen_index: int
    output_index: int


@dataclass(frozen=True)
class UGenSpec:
    name: str
    rate: int
    inputs: tuple[InputRef, ...]
    output_rates: tuple[int, ...]
    special_index: int = 0


@dataclass(frozen=True)
class SynthDefSpec:
    name: str
    constants: tuple[float, ...]
    parameters: tuple[tuple[str, float], ...]
    ugens: tuple[UGenSpec, ...]


class SynthDefBuilder:
    def __init__(self, name: str) -> None:
        self.name = name
        self._constant_indexes: dict[float, int] = {}
        self.constants: list[float] = []
        self.parameters: list[tuple[str, float]] = []
        self.ugens: list[UGenSpec] = []

    def const(self, value: float) -> InputRef:
        key = float(value)
        index = self._constant_indexes.get(key)
        if index is None:
            index = len(self.constants)
            self._constant_indexes[key] = index
            self.constants.append(key)
        return InputRef(-1, index)

    def controls(self, parameters: list[tuple[str, float]]) -> list[InputRef]:
        start = len(self.parameters)
        self.parameters.extend(parameters)
        return self.ugen(
            "Control",
            RATE_CONTROL,
            [],
            outputs=len(parameters),
            special_index=start,
            output_rate=RATE_CONTROL,
        )

    def ugen(
        self,
        name: str,
        rate: int,
        inputs: list[InputRef],
        *,
        outputs: int = 1,
        special_index: int = 0,
        output_rate: int | None = None,
    ) -> list[InputRef]:
        index = len(self.ugens)
        resolved_output_rate = rate if output_rate is None else output_rate
        self.ugens.append(
            UGenSpec(
                name=name,
                rate=rate,
                inputs=tuple(inputs),
                output_rates=(resolved_output_rate,) * outputs,
                special_index=special_index,
            )
        )
        return [InputRef(index, output_index) for output_index in range(outputs)]

    def clip(self, value: InputRef, minimum: float, maximum: float) -> InputRef:
        return self.ugen(
            "Clip",
            RATE_CONTROL,
            [value, self.const(minimum), self.const(maximum)],
        )[0]

    def mul_add(self, rate: int, value: InputRef, mul: InputRef, add: InputRef) -> InputRef:
        return self.ugen("MulAdd", rate, [value, mul, add])[0]

    def out(self, bus: InputRef, channels: list[InputRef]) -> None:
        self.ugen("Out", RATE_AUDIO, [bus, *channels], outputs=0)

    def out_kr(self, bus: InputRef, channels: list[InputRef]) -> None:
        """Write a control-rate signal to a control bus (Out.kr)."""
        self.ugen("Out", RATE_CONTROL, [bus, *channels], outputs=0)

    def read_cv_kr(self, bus: InputRef) -> InputRef:
        """In.kr(bus) — read a control-rate CV bus."""
        return self.ugen("In", RATE_CONTROL, [bus], outputs=1)[0]

    def read_cv_ar(self, bus: InputRef) -> InputRef:
        """In.ar(bus, 1) — read an audio-rate CV bus (FM path)."""
        return self.ugen("In", RATE_AUDIO, [bus], outputs=1)[0]

    def finish(self) -> SynthDefSpec:
        return SynthDefSpec(
            name=self.name,
            constants=tuple(self.constants),
            parameters=tuple(self.parameters),
            ugens=tuple(self.ugens),
        )


def oscillator() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_oscillator")
    out_bus, frequency, wave_shape, level, frequency_cv_bus, level_cv_bus = builder.controls(
        [
            ("out_bus", 0.0),
            ("frequency", 220.0),
            ("wave_shape", 0.0),
            ("level", 1.0),
            ("frequency_cv_bus", SILENT_AUDIO_BUS),
            ("level_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    # Audio-rate FM (D-03 representative) + control-rate level CV summed in.
    fm = builder.read_cv_ar(frequency_cv_bus)
    base_freq = builder.mul_add(RATE_AUDIO, frequency, builder.const(1.0), fm)
    level_cv = builder.read_cv_kr(level_cv_bus)
    effective_level = builder.mul_add(RATE_CONTROL, level, builder.const(1.0), level_cv)
    selected_index = builder.clip(wave_shape, 0.0, 3.0)
    sine = builder.ugen("SinOsc", RATE_AUDIO, [base_freq, builder.const(0.0)])[0]
    saw = builder.ugen("Saw", RATE_AUDIO, [base_freq])[0]
    square = builder.ugen("Pulse", RATE_AUDIO, [base_freq, builder.const(0.5)])[0]
    triangle = builder.ugen("LFTri", RATE_AUDIO, [base_freq, builder.const(0.0)])[0]
    selected = builder.ugen("Select", RATE_AUDIO, [selected_index, sine, saw, square, triangle])[0]
    scaled = builder.mul_add(RATE_AUDIO, selected, effective_level, builder.const(0.0))
    builder.out(out_bus, [scaled, scaled])
    return builder.finish()


def noise() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_noise")
    out_bus, noise_color, level, level_cv_bus = builder.controls(
        [
            ("out_bus", 0.0),
            ("noise_color", 0.0),
            ("level", 1.0),
            ("level_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    level_cv = builder.read_cv_kr(level_cv_bus)
    effective_level = builder.mul_add(RATE_CONTROL, level, builder.const(1.0), level_cv)
    selected_index = builder.clip(noise_color, 0.0, 1.0)
    white = builder.ugen("WhiteNoise", RATE_AUDIO, [])[0]
    pink = builder.ugen("PinkNoise", RATE_AUDIO, [])[0]
    selected = builder.ugen("Select", RATE_AUDIO, [selected_index, white, pink])[0]
    scaled = builder.mul_add(RATE_AUDIO, selected, effective_level, builder.const(0.0))
    builder.out(out_bus, [scaled, scaled])
    return builder.finish()


def filter() -> SynthDefSpec:
    """Param-driven LPF/HPF/BPF/BRF via Select over the *LPF family (D-02)."""
    builder = SynthDefBuilder("scrysynth_v2_filter")
    (
        in_bus,
        out_bus,
        cutoff_hz,
        resonance,
        filter_mode,
        mix,
        bypassed,
        cutoff_cv_bus,
        resonance_cv_bus,
    ) = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("cutoff_hz", 1200.0),
            ("resonance", 0.5),
            ("filter_mode", 0.0),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("cutoff_cv_bus", SILENT_CONTROL_BUS),
            ("resonance_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    cutoff_cv = builder.read_cv_kr(cutoff_cv_bus)
    resonance_cv = builder.read_cv_kr(resonance_cv_bus)
    effective_cutoff = builder.mul_add(RATE_CONTROL, cutoff_hz, builder.const(1.0), cutoff_cv)
    effective_resonance = builder.mul_add(RATE_CONTROL, resonance, builder.const(1.0), resonance_cv)
    clipped_cutoff = builder.clip(effective_cutoff, 20.0, 20000.0)
    clipped_resonance = builder.clip(effective_resonance, 0.05, 1.0)
    mode_index = builder.clip(filter_mode, 0.0, 3.0)
    lpf_l = builder.ugen("LPF", RATE_AUDIO, [input_left, clipped_cutoff])[0]
    hpf_l = builder.ugen("HPF", RATE_AUDIO, [input_left, clipped_cutoff])[0]
    bpf_l = builder.ugen("BPF", RATE_AUDIO, [input_left, clipped_cutoff, clipped_resonance])[0]
    brf_l = builder.ugen("BRF", RATE_AUDIO, [input_left, clipped_cutoff, clipped_resonance])[0]
    filtered_left = builder.ugen("Select", RATE_AUDIO, [mode_index, lpf_l, hpf_l, bpf_l, brf_l])[0]
    lpf_r = builder.ugen("LPF", RATE_AUDIO, [input_right, clipped_cutoff])[0]
    hpf_r = builder.ugen("HPF", RATE_AUDIO, [input_right, clipped_cutoff])[0]
    bpf_r = builder.ugen("BPF", RATE_AUDIO, [input_right, clipped_cutoff, clipped_resonance])[0]
    brf_r = builder.ugen("BRF", RATE_AUDIO, [input_right, clipped_cutoff, clipped_resonance])[0]
    filtered_right = builder.ugen("Select", RATE_AUDIO, [mode_index, lpf_r, hpf_r, bpf_r, brf_r])[0]
    mix_pan = builder.mul_add(
        RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0)
    )
    wet_left = builder.ugen("XFade2", RATE_AUDIO, [input_left, filtered_left, mix_pan, builder.const(1.0)])[0]
    wet_right = builder.ugen("XFade2", RATE_AUDIO, [input_right, filtered_right, mix_pan, builder.const(1.0)])[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def envelope() -> SynthDefSpec:
    """Control-rate ADSR envelope source; writes its signal to out_cv_bus."""
    builder = SynthDefBuilder("scrysynth_v2_envelope")
    out_cv_bus, gate, attack, decay, sustain, release = builder.controls(
        [
            ("out_cv_bus", SILENT_CONTROL_BUS),
            ("gate", 0.0),
            ("attack", 0.01),
            ("decay", 0.2),
            ("sustain", 0.8),
            ("release", 0.3),
        ]
    )
    env = builder.ugen(
        "EnvGen",
        RATE_CONTROL,
        [gate, builder.const(1.0), builder.const(1.0), attack, decay, sustain, release, builder.const(-4.0)],
    )[0]
    builder.out_kr(out_cv_bus, [env])
    return builder.finish()


def lfo() -> SynthDefSpec:
    """Control-rate LFO; wave_shape selects LFCub/LFSaw/LFPulse/LFTri."""
    builder = SynthDefBuilder("scrysynth_v2_lfo")
    out_cv_bus, frequency, wave_shape, level, frequency_cv_bus = builder.controls(
        [
            ("out_cv_bus", SILENT_CONTROL_BUS),
            ("frequency", 0.5),
            ("wave_shape", 0.0),
            ("level", 1.0),
            ("frequency_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    freq_cv = builder.read_cv_kr(frequency_cv_bus)
    effective_freq = builder.mul_add(RATE_CONTROL, frequency, builder.const(1.0), freq_cv)
    selected_index = builder.clip(wave_shape, 0.0, 3.0)
    cub = builder.ugen("LFCub", RATE_CONTROL, [effective_freq, builder.const(0.0)])[0]
    saw = builder.ugen("LFSaw", RATE_CONTROL, [effective_freq])[0]
    pulse = builder.ugen("LFPulse", RATE_CONTROL, [effective_freq, builder.const(0.0), builder.const(0.5)])[0]
    tri = builder.ugen("LFTri", RATE_CONTROL, [effective_freq, builder.const(0.0)])[0]
    selected = builder.ugen("Select", RATE_CONTROL, [selected_index, cub, saw, pulse, tri])[0]
    scaled = builder.mul_add(RATE_CONTROL, selected, level, builder.const(0.0))
    builder.out_kr(out_cv_bus, [scaled])
    return builder.finish()


def vca() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_vca")
    in_bus, out_bus, level, level_cv_bus = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("level", 1.0),
            ("level_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    level_cv = builder.read_cv_kr(level_cv_bus)
    effective_level = builder.mul_add(RATE_CONTROL, level, builder.const(1.0), level_cv)
    out_left = builder.mul_add(RATE_AUDIO, input_left, effective_level, builder.const(0.0))
    out_right = builder.mul_add(RATE_AUDIO, input_right, effective_level, builder.const(0.0))
    builder.out(out_bus, [out_left, out_right])
    return builder.finish()


def quantizer() -> SynthDefSpec:
    """Snap a control signal to the nearest of `steps` equal divisions."""
    builder = SynthDefBuilder("scrysynth_v2_quantizer")
    in_cv_bus, out_cv_bus, steps = builder.controls(
        [
            ("in_cv_bus", SILENT_CONTROL_BUS),
            ("out_cv_bus", SILENT_CONTROL_BUS),
            ("steps", 12.0),
        ]
    )
    signal = builder.read_cv_kr(in_cv_bus)
    safe_steps = builder.clip(steps, 1.0, 48.0)
    # round(signal * steps) / steps
    scaled = builder.mul_add(RATE_CONTROL, signal, safe_steps, builder.const(0.0))
    rounded = builder.ugen("Round", RATE_CONTROL, [scaled, builder.const(1.0)])[0]
    quantized = builder.mul_add(RATE_CONTROL, rounded, builder.const(1.0), builder.const(0.0))
    builder.out_kr(out_cv_bus, [quantized])
    return builder.finish()


def mixer() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_mixer")
    controls = builder.controls(
        [
            ("out_bus", 0.0),
            ("input_count", 0.0),
            ("in_bus_1", -1.0),
            ("in_bus_2", -1.0),
            ("in_bus_3", -1.0),
            ("in_bus_4", -1.0),
            ("in_bus_5", -1.0),
            ("in_bus_6", -1.0),
            ("in_bus_7", -1.0),
            ("in_bus_8", -1.0),
            ("level", 1.0),
            ("mix", 1.0),
        ]
    )
    out_bus = controls[0]
    input_count = controls[1]
    input_buses = controls[2:10]
    level = controls[10]
    mix = controls[11]

    left_channels: list[InputRef] = []
    right_channels: list[InputRef] = []
    for index, bus in enumerate(input_buses):
        active_pre = builder.mul_add(
            RATE_CONTROL, input_count, builder.const(1.0), builder.const(float(-index))
        )
        active = builder.clip(active_pre, 0.0, 1.0)
        safe_bus = builder.clip(bus, 0.0, 1024.0)
        left, right = builder.ugen("In", RATE_AUDIO, [safe_bus], outputs=2)
        left_channels.append(builder.mul_add(RATE_AUDIO, left, active, builder.const(0.0)))
        right_channels.append(builder.mul_add(RATE_AUDIO, right, active, builder.const(0.0)))

    sum_left_a = builder.ugen("Sum4", RATE_AUDIO, left_channels[:4])[0]
    sum_left_b = builder.ugen("Sum4", RATE_AUDIO, left_channels[4:])[0]
    sum_right_a = builder.ugen("Sum4", RATE_AUDIO, right_channels[:4])[0]
    sum_right_b = builder.ugen("Sum4", RATE_AUDIO, right_channels[4:])[0]
    sum_left = builder.ugen("Sum4", RATE_AUDIO, [sum_left_a, sum_left_b, builder.const(0.0), builder.const(0.0)])[0]
    sum_right = builder.ugen("Sum4", RATE_AUDIO, [sum_right_a, sum_right_b, builder.const(0.0), builder.const(0.0)])[0]
    level_mix = builder.mul_add(RATE_CONTROL, level, mix, builder.const(0.0))
    builder.out(
        out_bus,
        [
            builder.mul_add(RATE_AUDIO, sum_left, level_mix, builder.const(0.0)),
            builder.mul_add(RATE_AUDIO, sum_right, level_mix, builder.const(0.0)),
        ],
    )
    return builder.finish()


def delay() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_delay")
    (
        in_bus,
        out_bus,
        delay_time_s,
        feedback,
        mix,
        bypassed,
        delay_time_cv_bus,
        feedback_cv_bus,
    ) = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("delay_time_s", 0.25),
            ("feedback", 0.25),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("delay_time_cv_bus", SILENT_CONTROL_BUS),
            ("feedback_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    local_left, local_right = builder.ugen("LocalIn", RATE_AUDIO, [builder.const(0.0), builder.const(0.0)], outputs=2)
    feedback_cv = builder.read_cv_kr(feedback_cv_bus)
    effective_feedback = builder.mul_add(RATE_CONTROL, feedback, builder.const(1.0), feedback_cv)
    clipped_feedback = builder.clip(effective_feedback, 0.0, 0.95)
    feedback_left = builder.mul_add(RATE_AUDIO, local_left, clipped_feedback, builder.const(0.0))
    feedback_right = builder.mul_add(RATE_AUDIO, local_right, clipped_feedback, builder.const(0.0))
    delay_input_left = builder.mul_add(RATE_AUDIO, input_left, builder.const(1.0), feedback_left)
    delay_input_right = builder.mul_add(RATE_AUDIO, input_right, builder.const(1.0), feedback_right)
    delay_time_cv = builder.read_cv_kr(delay_time_cv_bus)
    effective_delay = builder.mul_add(RATE_CONTROL, delay_time_s, builder.const(1.0), delay_time_cv)
    clipped_delay = builder.clip(effective_delay, 0.0, 2.0)
    delayed_left = builder.ugen("DelayC", RATE_AUDIO, [delay_input_left, builder.const(2.0), clipped_delay])[0]
    delayed_right = builder.ugen("DelayC", RATE_AUDIO, [delay_input_right, builder.const(2.0), clipped_delay])[0]
    mix_pan = builder.mul_add(RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0))
    wet_left = builder.ugen("XFade2", RATE_AUDIO, [input_left, delayed_left, mix_pan, builder.const(1.0)])[0]
    wet_right = builder.ugen("XFade2", RATE_AUDIO, [input_right, delayed_right, mix_pan, builder.const(1.0)])[0]
    builder.ugen("LocalOut", RATE_AUDIO, [delayed_left, delayed_right], outputs=0)
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def reverb() -> SynthDefSpec:
    """FreeVerb (mix/room/damp) with mix + room CV."""
    builder = SynthDefBuilder("scrysynth_v2_reverb")
    in_bus, out_bus, room, damp, mix, bypassed, room_cv_bus, mix_cv_bus = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("room", 0.5),
            ("damp", 0.5),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("room_cv_bus", SILENT_CONTROL_BUS),
            ("mix_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    room_cv = builder.read_cv_kr(room_cv_bus)
    mix_cv = builder.read_cv_kr(mix_cv_bus)
    effective_room = builder.clip(builder.mul_add(RATE_CONTROL, room, builder.const(1.0), room_cv), 0.0, 1.0)
    effective_mix = builder.clip(builder.mul_add(RATE_CONTROL, mix, builder.const(1.0), mix_cv), 0.0, 1.0)
    wet_left = builder.ugen("FreeVerb", RATE_AUDIO, [input_left, effective_mix, effective_room, damp])[0]
    wet_right = builder.ugen("FreeVerb", RATE_AUDIO, [input_right, effective_mix, effective_room, damp])[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def distortion() -> SynthDefSpec:
    """tanh waveshaper with drive + mix CV."""
    builder = SynthDefBuilder("scrysynth_v2_distortion")
    in_bus, out_bus, drive, mix, bypassed, drive_cv_bus, mix_cv_bus = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("drive", 0.5),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("drive_cv_bus", SILENT_CONTROL_BUS),
            ("mix_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    drive_cv = builder.read_cv_kr(drive_cv_bus)
    mix_cv = builder.read_cv_kr(mix_cv_bus)
    effective_drive = builder.clip(builder.mul_add(RATE_CONTROL, drive, builder.const(1.0), drive_cv), 0.0, 1.0)
    effective_mix = builder.clip(builder.mul_add(RATE_CONTROL, mix, builder.const(1.0), mix_cv), 0.0, 1.0)
    driven_left = builder.mul_add(RATE_AUDIO, input_left, effective_drive, builder.const(0.0))
    driven_right = builder.mul_add(RATE_AUDIO, input_right, effective_drive, builder.const(0.0))
    shaped_left = builder.ugen("tanh", RATE_AUDIO, [driven_left])[0]
    shaped_right = builder.ugen("tanh", RATE_AUDIO, [driven_right])[0]
    mix_pan = builder.mul_add(RATE_CONTROL, effective_mix, builder.const(2.0), builder.const(-1.0))
    wet_left = builder.ugen("XFade2", RATE_AUDIO, [input_left, shaped_left, mix_pan, builder.const(1.0)])[0]
    wet_right = builder.ugen("XFade2", RATE_AUDIO, [input_right, shaped_right, mix_pan, builder.const(1.0)])[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def chorus() -> SynthDefSpec:
    """Modulated CombC chorus with depth + rate CV."""
    builder = SynthDefBuilder("scrysynth_v2_chorus")
    in_bus, out_bus, depth, rate, mix, bypassed, depth_cv_bus, rate_cv_bus = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("depth", 0.3),
            ("rate", 0.5),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("depth_cv_bus", SILENT_CONTROL_BUS),
            ("rate_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    depth_cv = builder.read_cv_kr(depth_cv_bus)
    rate_cv = builder.read_cv_kr(rate_cv_bus)
    effective_depth = builder.clip(builder.mul_add(RATE_CONTROL, depth, builder.const(1.0), depth_cv), 0.0, 1.0)
    effective_rate = builder.clip(builder.mul_add(RATE_CONTROL, rate, builder.const(1.0), rate_cv), 0.0, 10.0)
    mod = builder.ugen("SinOsc", RATE_AUDIO, [effective_rate, builder.const(0.0)])[0]
    max_delay = builder.const(0.05)
    delay_time_left = builder.mul_add(RATE_AUDIO, mod, effective_depth, builder.const(0.025))
    delay_time_right = builder.mul_add(RATE_AUDIO, mod, effective_depth, builder.const(0.030))
    wet_left = builder.ugen("CombC", RATE_AUDIO, [input_left, max_delay, delay_time_left, builder.const(0.0)])[0]
    wet_right = builder.ugen("CombC", RATE_AUDIO, [input_right, max_delay, delay_time_right, builder.const(0.0)])[0]
    mix_pan = builder.mul_add(RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0))
    out_left = builder.ugen("XFade2", RATE_AUDIO, [input_left, wet_left, mix_pan, builder.const(1.0)])[0]
    out_right = builder.ugen("XFade2", RATE_AUDIO, [input_right, wet_right, mix_pan, builder.const(1.0)])[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, out_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, out_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def flanger() -> SynthDefSpec:
    """CombC + AllpassN feedback flanger with depth/rate/feedback CV."""
    builder = SynthDefBuilder("scrysynth_v2_flanger")
    (
        in_bus,
        out_bus,
        depth,
        rate,
        feedback,
        mix,
        bypassed,
        depth_cv_bus,
        rate_cv_bus,
        feedback_cv_bus,
    ) = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("depth", 0.3),
            ("rate", 0.5),
            ("feedback", 0.3),
            ("mix", 1.0),
            ("bypassed", 0.0),
            ("depth_cv_bus", SILENT_CONTROL_BUS),
            ("rate_cv_bus", SILENT_CONTROL_BUS),
            ("feedback_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    depth_cv = builder.read_cv_kr(depth_cv_bus)
    rate_cv = builder.read_cv_kr(rate_cv_bus)
    feedback_cv = builder.read_cv_kr(feedback_cv_bus)
    effective_depth = builder.clip(builder.mul_add(RATE_CONTROL, depth, builder.const(1.0), depth_cv), 0.0, 1.0)
    effective_rate = builder.clip(builder.mul_add(RATE_CONTROL, rate, builder.const(1.0), rate_cv), 0.0, 10.0)
    effective_feedback = builder.clip(builder.mul_add(RATE_CONTROL, feedback, builder.const(1.0), feedback_cv), 0.0, 0.95)
    mod = builder.ugen("SinOsc", RATE_AUDIO, [effective_rate, builder.const(0.0)])[0]
    delay_time = builder.mul_add(RATE_AUDIO, mod, effective_depth, builder.const(0.005))
    delayed_left = builder.ugen("CombC", RATE_AUDIO, [input_left, builder.const(0.01), delay_time, effective_feedback])[0]
    delayed_right = builder.ugen("CombC", RATE_AUDIO, [input_right, builder.const(0.01), delay_time, effective_feedback])[0]
    wet_left = builder.ugen("AllpassN", RATE_AUDIO, [delayed_left, builder.const(0.01), builder.const(0.005), builder.const(0.0)])[0]
    wet_right = builder.ugen("AllpassN", RATE_AUDIO, [delayed_right, builder.const(0.01), builder.const(0.005), builder.const(0.0)])[0]
    mix_pan = builder.mul_add(RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0))
    out_left = builder.ugen("XFade2", RATE_AUDIO, [input_left, wet_left, mix_pan, builder.const(1.0)])[0]
    out_right = builder.ugen("XFade2", RATE_AUDIO, [input_right, wet_right, mix_pan, builder.const(1.0)])[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, out_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, out_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def output() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v2_output")
    in_bus, hardware_out, channels, level, level_cv_bus = builder.controls(
        [
            ("in_bus", 0.0),
            ("hardware_out", 0.0),
            ("channels", 2.0),
            ("level", 1.0),
            ("level_cv_bus", SILENT_CONTROL_BUS),
        ]
    )
    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    level_cv = builder.read_cv_kr(level_cv_bus)
    effective_level = builder.mul_add(RATE_CONTROL, level, builder.const(1.0), level_cv)
    builder.out(
        hardware_out,
        [
            builder.mul_add(RATE_AUDIO, input_left, effective_level, builder.const(0.0)),
            builder.mul_add(RATE_AUDIO, input_right, effective_level, builder.const(0.0)),
        ],
    )
    return builder.finish()


def pstring(value: str) -> bytes:
    encoded = value.encode("utf-8")
    if len(encoded) > 255:
        raise ValueError(f"pstring is too long: {value}")
    return bytes([len(encoded)]) + encoded


def int8(value: int) -> bytes:
    return struct.pack(">b", value)


def int16(value: int) -> bytes:
    return struct.pack(">h", value)


def int32(value: int) -> bytes:
    return struct.pack(">i", value)


def float32(value: float) -> bytes:
    return struct.pack(">f", value)


def synthdef_bytes(spec: SynthDefSpec) -> bytes:
    data = bytearray()
    data.extend(b"SCgf")
    data.extend(int32(2))
    data.extend(int16(1))
    data.extend(pstring(spec.name))

    data.extend(int32(len(spec.constants)))
    for constant in spec.constants:
        data.extend(float32(constant))

    data.extend(int32(len(spec.parameters)))
    for _name, value in spec.parameters:
        data.extend(float32(value))

    data.extend(int32(len(spec.parameters)))
    for index, (name, _value) in enumerate(spec.parameters):
        data.extend(pstring(name))
        data.extend(int32(index))

    data.extend(int32(len(spec.ugens)))
    for ugen in spec.ugens:
        data.extend(pstring(ugen.name))
        data.extend(int8(ugen.rate))
        data.extend(int32(len(ugen.inputs)))
        data.extend(int32(len(ugen.output_rates)))
        data.extend(int16(ugen.special_index))
        for input_ref in ugen.inputs:
            data.extend(int32(input_ref.ugen_index))
            data.extend(int32(input_ref.output_index))
        for output_rate in ugen.output_rates:
            data.extend(int8(output_rate))

    data.extend(int16(0))
    return bytes(data)


def definitions() -> list[SynthDefSpec]:
    return [
        oscillator(),
        noise(),
        filter(),
        envelope(),
        lfo(),
        vca(),
        quantizer(),
        mixer(),
        delay(),
        reverb(),
        distortion(),
        chorus(),
        flanger(),
        output(),
    ]


def main() -> None:
    output_dir = Path(__file__).resolve().parent
    for spec in definitions():
        path = output_dir / f"{spec.name}.scsyndef"
        path.write_bytes(synthdef_bytes(spec))
        print(f"wrote {path.relative_to(output_dir.parent.parent.parent)}")


if __name__ == "__main__":
    main()
