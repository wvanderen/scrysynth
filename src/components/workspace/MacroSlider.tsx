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
    <div className="macro-slider">
      <div className="macro-slider-header">
        <span>{macroName}</span>
        <span>
          {scaledValue.toFixed(2)} ({rangeStart}–{rangeEnd})
        </span>
      </div>
      <input
        className="macro-slider-input"
        type="range"
        min={0}
        max={1}
        step={0.001}
        value={localValue}
        onChange={handleChange}
      />
    </div>
  );
}
