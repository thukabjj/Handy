import React, { useCallback, useEffect, useRef, useState } from "react";

const SPEED_PRESETS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2] as const;

interface SpeedControlProps {
  speed: number;
  onChange: (speed: number) => void;
}

export const SpeedControl: React.FC<SpeedControlProps> = ({
  speed,
  onChange,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  const currentIndex = SPEED_PRESETS.indexOf(speed as (typeof SPEED_PRESETS)[number]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setFocusedIndex(-1);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleSelect = useCallback(
    (value: number) => {
      onChange(value);
      setIsOpen(false);
      setFocusedIndex(-1);
    },
    [onChange],
  );

  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      switch (event.key) {
        case "Enter":
        case " ":
          event.preventDefault();
          if (isOpen && focusedIndex >= 0) {
            handleSelect(SPEED_PRESETS[focusedIndex]);
          } else {
            setIsOpen(true);
            setFocusedIndex(currentIndex >= 0 ? currentIndex : 0);
          }
          break;
        case "ArrowDown":
          event.preventDefault();
          if (!isOpen) {
            setIsOpen(true);
            setFocusedIndex(currentIndex >= 0 ? currentIndex : 0);
          } else {
            setFocusedIndex((prev) =>
              prev < SPEED_PRESETS.length - 1 ? prev + 1 : 0,
            );
          }
          break;
        case "ArrowUp":
          event.preventDefault();
          if (!isOpen) {
            setIsOpen(true);
            setFocusedIndex(currentIndex >= 0 ? currentIndex : 0);
          } else {
            setFocusedIndex((prev) =>
              prev > 0 ? prev - 1 : SPEED_PRESETS.length - 1,
            );
          }
          break;
        case "Escape":
          event.preventDefault();
          setIsOpen(false);
          setFocusedIndex(-1);
          break;
      }
    },
    [isOpen, focusedIndex, currentIndex, handleSelect],
  );

  return (
    <div className="relative" ref={containerRef}>
      <button
        type="button"
        className="px-1.5 py-0.5 text-xs font-semibold rounded border border-mid-gray/40 bg-mid-gray/10 text-text-secondary hover:bg-primary-light/10 hover:border-primary-light/50 transition-colors cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-light"
        onClick={() => {
          setIsOpen((prev) => !prev);
          if (!isOpen) {
            setFocusedIndex(currentIndex >= 0 ? currentIndex : 0);
          }
        }}
        onKeyDown={handleKeyDown}
        aria-label={`Playback speed: ${speed}x`}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
      >
        {/* eslint-disable-next-line i18next/no-literal-string */}
        {speed}x
      </button>
      {isOpen && (
        <div
          role="listbox"
          aria-label="Playback speed"
          className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 bg-background border border-mid-gray/40 rounded shadow-lg z-50 py-0.5 min-w-[56px]"
        >
          {SPEED_PRESETS.map((preset, index) => (
            <div
              key={preset}
              role="option"
              aria-selected={speed === preset}
              className={`px-2 py-0.5 text-xs text-center cursor-pointer transition-colors ${
                focusedIndex === index
                  ? "bg-primary-light/20"
                  : "hover:bg-primary-light/10"
              } ${speed === preset ? "font-semibold text-primary-light" : "text-text-secondary"}`}
              onClick={() => handleSelect(preset)}
              onMouseEnter={() => setFocusedIndex(index)}
            >
              {/* eslint-disable-next-line i18next/no-literal-string */}
              {preset}x
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
