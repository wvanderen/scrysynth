#!/usr/bin/env python3
"""Generate deterministic Scrysynth v1 SuperCollider SynthDef resources.

This is a small SynthDef v2 writer for the six checked-in v1 definitions. It is
intended as a reproducible fallback for environments where `sclang` cannot run,
while mirroring `scrysynth_v1_synthdefs.scd`.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import struct


RATE_SCALAR = 0
RATE_CONTROL = 1
RATE_AUDIO = 2


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

    def finish(self) -> SynthDefSpec:
        return SynthDefSpec(
            name=self.name,
            constants=tuple(self.constants),
            parameters=tuple(self.parameters),
            ugens=tuple(self.ugens),
        )


def source_oscillator() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_source_oscillator")
    out_bus, level, frequency, wave_shape = builder.controls(
        [
            ("out_bus", 0.0),
            ("level", 1.0),
            ("frequency", 220.0),
            ("wave_shape", 0.0),
        ]
    )

    selected_index = builder.clip(wave_shape, 0.0, 2.0)
    sine = builder.ugen("SinOsc", RATE_AUDIO, [frequency, builder.const(0.0)])[0]
    saw = builder.ugen("Saw", RATE_AUDIO, [frequency])[0]
    square = builder.ugen("Pulse", RATE_AUDIO, [frequency, builder.const(0.5)])[0]
    selected = builder.ugen("Select", RATE_AUDIO, [selected_index, sine, saw, square])[0]
    scaled = builder.mul_add(RATE_AUDIO, selected, level, builder.const(0.0))
    builder.out(out_bus, [scaled, scaled])
    return builder.finish()


def source_noise() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_source_noise")
    out_bus, level, noise_color = builder.controls(
        [
            ("out_bus", 0.0),
            ("level", 1.0),
            ("noise_color", 0.0),
        ]
    )

    selected_index = builder.clip(noise_color, 0.0, 1.0)
    white = builder.ugen("WhiteNoise", RATE_AUDIO, [])[0]
    pink = builder.ugen("PinkNoise", RATE_AUDIO, [])[0]
    selected = builder.ugen("Select", RATE_AUDIO, [selected_index, white, pink])[0]
    scaled = builder.mul_add(RATE_AUDIO, selected, level, builder.const(0.0))
    builder.out(out_bus, [scaled, scaled])
    return builder.finish()


def lowpass() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_fx_lowpass")
    in_bus, out_bus, cutoff_hz, resonance, mix, bypassed = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("cutoff_hz", 1200.0),
            ("resonance", 0.5),
            ("mix", 1.0),
            ("bypassed", 0.0),
        ]
    )

    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    clipped_cutoff = builder.clip(cutoff_hz, 20.0, 20000.0)
    clipped_resonance = builder.clip(resonance, 0.05, 1.0)
    filtered_left = builder.ugen(
        "RLPF", RATE_AUDIO, [input_left, clipped_cutoff, clipped_resonance]
    )[0]
    filtered_right = builder.ugen(
        "RLPF", RATE_AUDIO, [input_right, clipped_cutoff, clipped_resonance]
    )[0]
    mix_pan = builder.mul_add(
        RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0)
    )
    wet_left = builder.ugen(
        "XFade2", RATE_AUDIO, [input_left, filtered_left, mix_pan, builder.const(1.0)]
    )[0]
    wet_right = builder.ugen(
        "XFade2", RATE_AUDIO, [input_right, filtered_right, mix_pan, builder.const(1.0)]
    )[0]
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def delay() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_fx_delay")
    in_bus, out_bus, delay_time_s, feedback, mix, bypassed = builder.controls(
        [
            ("in_bus", 0.0),
            ("out_bus", 0.0),
            ("delay_time_s", 0.25),
            ("feedback", 0.25),
            ("mix", 1.0),
            ("bypassed", 0.0),
        ]
    )

    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    local_left, local_right = builder.ugen(
        "LocalIn",
        RATE_AUDIO,
        [builder.const(0.0), builder.const(0.0)],
        outputs=2,
    )
    clipped_feedback = builder.clip(feedback, 0.0, 0.95)
    feedback_left = builder.mul_add(RATE_AUDIO, local_left, clipped_feedback, builder.const(0.0))
    feedback_right = builder.mul_add(RATE_AUDIO, local_right, clipped_feedback, builder.const(0.0))
    delay_input_left = builder.mul_add(
        RATE_AUDIO, input_left, builder.const(1.0), feedback_left
    )
    delay_input_right = builder.mul_add(
        RATE_AUDIO, input_right, builder.const(1.0), feedback_right
    )
    clipped_delay = builder.clip(delay_time_s, 0.0, 2.0)
    delayed_left = builder.ugen(
        "DelayC", RATE_AUDIO, [delay_input_left, builder.const(2.0), clipped_delay]
    )[0]
    delayed_right = builder.ugen(
        "DelayC", RATE_AUDIO, [delay_input_right, builder.const(2.0), clipped_delay]
    )[0]
    mix_pan = builder.mul_add(
        RATE_CONTROL, builder.clip(mix, 0.0, 1.0), builder.const(2.0), builder.const(-1.0)
    )
    wet_left = builder.ugen(
        "XFade2", RATE_AUDIO, [input_left, delayed_left, mix_pan, builder.const(1.0)]
    )[0]
    wet_right = builder.ugen(
        "XFade2", RATE_AUDIO, [input_right, delayed_right, mix_pan, builder.const(1.0)]
    )[0]
    builder.ugen("LocalOut", RATE_AUDIO, [delayed_left, delayed_right], outputs=0)
    bypass_index = builder.clip(bypassed, 0.0, 1.0)
    output_left = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_left, input_left])[0]
    output_right = builder.ugen("Select", RATE_AUDIO, [bypass_index, wet_right, input_right])[0]
    builder.out(out_bus, [output_left, output_right])
    return builder.finish()


def mixer() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_mixer")
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
    sum_left = builder.ugen(
        "Sum4", RATE_AUDIO, [sum_left_a, sum_left_b, builder.const(0.0), builder.const(0.0)]
    )[0]
    sum_right = builder.ugen(
        "Sum4", RATE_AUDIO, [sum_right_a, sum_right_b, builder.const(0.0), builder.const(0.0)]
    )[0]
    level_mix = builder.mul_add(RATE_CONTROL, level, mix, builder.const(0.0))
    builder.out(
        out_bus,
        [
            builder.mul_add(RATE_AUDIO, sum_left, level_mix, builder.const(0.0)),
            builder.mul_add(RATE_AUDIO, sum_right, level_mix, builder.const(0.0)),
        ],
    )
    return builder.finish()


def output() -> SynthDefSpec:
    builder = SynthDefBuilder("scrysynth_v1_output")
    in_bus, hardware_out, _channels, level = builder.controls(
        [
            ("in_bus", 0.0),
            ("hardware_out", 0.0),
            ("channels", 2.0),
            ("level", 1.0),
        ]
    )

    input_left, input_right = builder.ugen("In", RATE_AUDIO, [in_bus], outputs=2)
    builder.out(
        hardware_out,
        [
            builder.mul_add(RATE_AUDIO, input_left, level, builder.const(0.0)),
            builder.mul_add(RATE_AUDIO, input_right, level, builder.const(0.0)),
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
        source_oscillator(),
        source_noise(),
        lowpass(),
        delay(),
        mixer(),
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
