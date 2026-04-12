import { useCallback, useRef, useState } from "react";

type MacroSliderProps = {
  macroId: string;
  macroName: string;
  rangeStart: number;
  rangeEnd: number;
  onValueChange: (macroId: string, value: number) => void;
};

export function MacroSlider({
  macroId,
  macroName,
  rangeStart,
  rangeEnd,
  onValueChange,
}: MacroSliderProps) {
  const [localValue, setLocalValue] = useState(0);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const v = parseFloat(e.target.value);
      setLocalValue(v);

      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      debounceRef.current = setTimeout(() => {
        onValueChange(macroId, v);
      }, 300);
    },
    [macroId, onValueChange],
  );

  const scaledValue = rangeStart + (localValue * (rangeEnd - rangeStart));

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 4,
        padding: "8px 12px",
        background: "#112725",
        border: "1px solid #2d4442",
        borderRadius: 10,
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ color: "#f2eee5", fontSize: 13, fontWeight: 600 }}>{macroName}</span>
        <span style={{ color: "#d9c8a0", fontSize: 11 }}>
          {scaledValue.toFixed(2)} ({rangeStart}–{rangeEnd})
        </span>
      </div>
      <input
        type="range"
        min={0}
        max={1}
        step={0.001}
        value={localValue}
        onChange={handleChange}
        style={{
          width: "100%",
          height: 24,
          accentColor: "#f7c66a",
          cursor: "pointer",
        }}
      />
    </div>
  );
}
