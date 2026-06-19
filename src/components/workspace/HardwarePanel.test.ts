import { describe, expect, it } from "vitest";

import type { HardwareBinding, MacroDefinition, SceneDefinition } from "../../generated/session-types";
import {
  bindingSourceKind,
  bindingTargetLabel,
  listenerLabel,
  statusTone,
} from "./HardwarePanel";

const macro: MacroDefinition = {
  id: "macro-energy",
  name: "Energy",
  targetParameterIds: [],
  targets: [],
  rangeStart: 0,
  rangeEnd: 1,
};

const scene: SceneDefinition = {
  id: "scene-main",
  name: "Main Scene",
  activeNodeIds: [],
  macroOverrides: [],
};

describe("hardware panel projections", () => {
  it("labels listener states with concise display text and severity", () => {
    expect(listenerLabel("stopped")).toBe("Stopped");
    expect(listenerLabel("listening")).toBe("Listening");
    expect(listenerLabel("learning")).toBe("Learning");
    expect(listenerLabel("captured")).toBe("Captured");
    expect(listenerLabel("error")).toBe("Error");
    expect(listenerLabel("unavailable")).toBe("Unavailable");

    expect(statusTone("listening")).toBe("ok");
    expect(statusTone("learning")).toBe("warn");
    expect(statusTone("unavailable")).toBe("error");
    expect(statusTone("stopped")).toBe("idle");
  });

  it("identifies MIDI and OSC binding rows separately", () => {
    const midiBinding: HardwareBinding = {
      id: "binding-midi",
      source: { kind: "midiCc", config: { channel: 0, controller: 7 } },
      target: { kind: "macro", config: { macro_id: "macro-energy" } },
      transform: { inputMin: 0, inputMax: 127, outputMin: 0, outputMax: 1 },
    };
    const oscBinding: HardwareBinding = {
      id: "binding-osc",
      source: { kind: "oscAddress", config: { address: "/scrysynth/panic" } },
      target: { kind: "transportPanic" },
      transform: { inputMin: 0, inputMax: 1, outputMin: 0, outputMax: 1 },
    };

    expect(bindingSourceKind(midiBinding)).toBe("MIDI");
    expect(bindingSourceKind(oscBinding)).toBe("OSC");
  });

  it("resolves macro scene and transport target labels for binding rows", () => {
    expect(bindingTargetLabel(
      { kind: "macro", config: { macro_id: "macro-energy" } },
      [macro],
      [scene],
    )).toBe("Energy");

    expect(bindingTargetLabel(
      { kind: "sceneRecall", config: { scene_id: "scene-main" } },
      [macro],
      [scene],
    )).toBe("Main Scene");

    expect(bindingTargetLabel({ kind: "transportStop" }, [macro], [scene])).toBe("transport stop");
  });
});
