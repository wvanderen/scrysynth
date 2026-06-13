var outDir = thisProcess.argv[0];

SynthDef(\scrysynth_v1_source_oscillator, {
    |out_bus = 0, level = 1, frequency = 220, wave_shape = 0|
    var sine = SinOsc.ar(frequency);
    var saw = Saw.ar(frequency);
    var square = Pulse.ar(frequency, 0.5);
    var selected = Select.ar(wave_shape.clip(0, 2), [sine, saw, square]);
    Out.ar(out_bus, (selected * level).dup);
}).writeDefFile(outDir);

SynthDef(\scrysynth_v1_source_noise, {
    |out_bus = 0, level = 1, noise_color = 0|
    var white = WhiteNoise.ar;
    var pink = PinkNoise.ar;
    var selected = Select.ar(noise_color.clip(0, 1), [white, pink]);
    Out.ar(out_bus, (selected * level).dup);
}).writeDefFile(outDir);

SynthDef(\scrysynth_v1_fx_lowpass, {
    |in_bus = 0, out_bus = 0, cutoff_hz = 1200, resonance = 0.5, mix = 1, bypassed = 0|
    var input = In.ar(in_bus, 2);
    var filtered = RLPF.ar(input, cutoff_hz.clip(20, 20000), resonance.clip(0.05, 1));
    var wet = XFade2.ar(input, filtered, (mix.clip(0, 1) * 2) - 1);
    Out.ar(out_bus, Select.ar(bypassed.clip(0, 1), [wet, input]));
}).writeDefFile(outDir);

SynthDef(\scrysynth_v1_fx_delay, {
    |in_bus = 0, out_bus = 0, delay_time_s = 0.25, feedback = 0.25, mix = 1, bypassed = 0|
    var input = In.ar(in_bus, 2);
    var local = LocalIn.ar(2);
    var delayed = DelayC.ar(input + (local * feedback.clip(0, 0.95)), 2, delay_time_s.clip(0, 2));
    var wet = XFade2.ar(input, delayed, (mix.clip(0, 1) * 2) - 1);
    LocalOut.ar(delayed);
    Out.ar(out_bus, Select.ar(bypassed.clip(0, 1), [wet, input]));
}).writeDefFile(outDir);

SynthDef(\scrysynth_v1_mixer, {
    |out_bus = 0, input_count = 0, in_bus_1 = -1, in_bus_2 = -1, in_bus_3 = -1, in_bus_4 = -1,
    in_bus_5 = -1, in_bus_6 = -1, in_bus_7 = -1, in_bus_8 = -1, level = 1, mix = 1|
    var buses = [in_bus_1, in_bus_2, in_bus_3, in_bus_4, in_bus_5, in_bus_6, in_bus_7, in_bus_8];
    var signals = Mix.fill(8, { |index|
        var active = (index < input_count) * (buses[index] >= 0);
        In.ar(buses[index].max(0), 2) * active;
    });
    Out.ar(out_bus, signals * level * mix);
}).writeDefFile(outDir);

SynthDef(\scrysynth_v1_output, {
    |in_bus = 0, hardware_out = 0, channels = 2, level = 1|
    var input = In.ar(in_bus, 2) * level;
    Out.ar(hardware_out, input);
}).writeDefFile(outDir);

0.exit;
