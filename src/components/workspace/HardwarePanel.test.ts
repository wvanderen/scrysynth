import { describe, expect, it } from "vitest";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";

import type {
  HardwareBinding,
  HardwareRuntimeSettings,
  HardwareRuntimeStatus,
  MacroDefinition,
  MidiInputPort,
  SceneDefinition,
} from "../../generated/session-types";
import {
  HardwarePanel,
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

const settings: HardwareRuntimeSettings = {
  midi: { selectedInputId: "midi-0", autoStart: false },
  osc: { bindHost: "127.0.0.1", listenPort: 9123, autoStart: true },
};

const status: HardwareRuntimeStatus = {
  midi: {
    lifecycle: "listening",
    selectedInputId: "midi-0",
    selectedDisplayName: "Launch Control",
    availableInputCount: 1,
    lastError: null,
  },
  osc: {
    lifecycle: "listening",
    bindHost: "127.0.0.1",
    listenPort: 9123,
    lastError: null,
  },
  learn: {
    lifecycle: "learning",
    target: { kind: "transportPlay" },
    source: null,
  },
  diagnostics: [
    {
      code: "listener_restart_required",
      message: "OSC listener restart required.",
      recoverable: true,
      detail: "Apply settings, then start hardware again.",
    },
  ],
};

const midiPorts: MidiInputPort[] = [
  { id: "midi-0", displayName: "Launch Control", isSelected: true },
  { id: "midi-1", displayName: "Keys", isSelected: false },
];

const bindings: HardwareBinding[] = [
  {
    id: "binding-midi",
    source: { kind: "midiCc", config: { channel: 0, controller: 7 } },
    target: { kind: "macro", config: { macro_id: "macro-energy" } },
    transform: { inputMin: 0, inputMax: 127, outputMin: 0, outputMax: 1 },
  },
  {
    id: "binding-osc",
    source: { kind: "oscAddress", config: { address: "/scrysynth/panic" } },
    target: { kind: "transportPanic" },
    transform: { inputMin: 0, inputMax: 1, outputMin: 0, outputMax: 1 },
  },
];

function renderPanel(overrides: Partial<React.ComponentProps<typeof HardwarePanel>> = {}) {
  return renderToStaticMarkup(React.createElement(HardwarePanel, {
    bindings,
    settings,
    status,
    midiInputPorts: midiPorts,
    macros: [macro],
    scenes: [scene],
    isLoading: false,
    onRefresh: () => {},
    onUpdateSettings: () => {},
    onStartListeners: () => {},
    onStopListeners: () => {},
    onStartLearn: () => {},
    onCancelLearn: () => {},
    onRemoveBinding: () => {},
    ...overrides,
  }));
}

describe("hardware panel projections", () => {
  it("labels listener states with concise display text and severity", () => {
    expect(listenerLabel("idle")).toBe("Idle");
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

  it("renders settings status diagnostics and OSC listener enablement", () => {
    const html = renderPanel();

    expect(html).toContain("MIDI input");
    expect(html).toContain("Launch Control");
    expect(html).toContain("OSC host");
    expect(html).toContain("127.0.0.1");
    expect(html).toContain("OSC port");
    expect(html).toContain("9123");
    expect(html).toContain("Start OSC listener with hardware");
    expect(html).toContain("checked");
    expect(html).toContain("Listening");
    expect(html).toContain("Learning");
    expect(html).toContain("OSC listener restart required.");
    expect(html).toContain("Apply settings, then start hardware again.");
  });

  it("renders macro scene and transport learn controls plus cancel", () => {
    const html = renderPanel();

    expect(html).toContain("Learn Targets");
    expect(html).toContain("Energy");
    expect(html).toContain("Main Scene");
    expect(html).toContain("Play");
    expect(html).toContain("Stop");
    expect(html).toContain("Panic");
    expect(html).toContain("Cancel Learn");
  });

  it("renders MIDI and OSC binding rows with clear remove controls", () => {
    const html = renderPanel();

    expect(html).toContain("MIDI CC Ch0 #7");
    expect(html).toContain("OSC /scrysynth/panic");
    expect(html).toContain("MIDI");
    expect(html).toContain("OSC");
    expect(html).toContain("transport panic");
    expect(html.match(/Remove/g)?.length).toBe(2);
  });

  it("renders idle learn status without an unknown label", () => {
    const html = renderPanel({
      status: {
        ...status,
        learn: { lifecycle: "idle", target: null, source: null },
        diagnostics: [],
      },
    });

    expect(html).toContain("Idle");
    expect(html).not.toContain("Unknown");
  });

  it("shows the selected MIDI input name while stopped even without backend display detail", () => {
    const html = renderPanel({
      status: {
        ...status,
        midi: {
          lifecycle: "stopped",
          selectedInputId: "midi-0",
          selectedDisplayName: null,
          availableInputCount: 1,
          lastError: "MIDI support could not be initialized",
        },
        diagnostics: [],
      },
    });

    expect(html).toContain("Launch Control");
    expect(html).not.toContain("MIDI support could not be initialized");
  });
});
