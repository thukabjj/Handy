import React, { useCallback, useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export interface DropdownOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface DropdownOptionGroup {
  label: string;
  options: DropdownOption[];
}

export type DropdownOptions = DropdownOption[] | DropdownOptionGroup[];

// Type guard to check if options are grouped
function isGroupedOptions(
  options: DropdownOptions,
): options is DropdownOptionGroup[] {
  return options.length > 0 && "options" in options[0];
}

// Flatten grouped options for keyboard navigation
function flattenOptions(options: DropdownOptions): DropdownOption[] {
  if (isGroupedOptions(options)) {
    return options.flatMap((group) => group.options);
  }
  return options;
}

interface DropdownProps {
  options: DropdownOptions;
  className?: string;
  selectedValue: string | null;
  onSelect: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  onRefresh?: () => void;
  /** Accessible label for the dropdown */
  "aria-label"?: string;
}

export const Dropdown: React.FC<DropdownProps> = ({
  options,
  selectedValue,
  onSelect,
  className = "",
  placeholder = "Select an option...",
  disabled = false,
  onRefresh,
  "aria-label": ariaLabel,
}) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const listboxRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const listboxId = useId();

  // Flatten options for keyboard navigation
  const flatOptions = flattenOptions(options);

  // Get enabled options for keyboard navigation
  const enabledOptions = flatOptions.filter((opt) => !opt.disabled);

  // Click outside handler
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setFocusedIndex(-1);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Reset focus index when dropdown closes
  useEffect(() => {
    if (!isOpen) {
      setFocusedIndex(-1);
    }
  }, [isOpen]);

  const selectedOption = flatOptions.find(
    (option) => option.value === selectedValue,
  );

  const handleSelect = useCallback(
    (value: string) => {
      onSelect(value);
      setIsOpen(false);
      setFocusedIndex(-1);
      // Return focus to the trigger button
      buttonRef.current?.focus();
    },
    [onSelect],
  );

  // Helper to open dropdown with initial focus
  const openDropdown = useCallback(() => {
    if (onRefresh) onRefresh();
    setIsOpen(true);
    const selectedIndex = enabledOptions.findIndex(
      (opt) => opt.value === selectedValue,
    );
    setFocusedIndex(selectedIndex >= 0 ? selectedIndex : 0);
  }, [enabledOptions, selectedValue, onRefresh]);

  const handleToggle = useCallback(() => {
    if (disabled) return;
    if (!isOpen) {
      openDropdown();
    } else {
      setIsOpen(false);
    }
  }, [disabled, isOpen, openDropdown]);

  // Keyboard navigation handler
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (disabled) return;

      switch (event.key) {
        case "Enter":
        case " ":
          event.preventDefault();
          if (isOpen && focusedIndex >= 0 && enabledOptions[focusedIndex]) {
            handleSelect(enabledOptions[focusedIndex].value);
          } else if (!isOpen) {
            openDropdown();
          }
          break;

        case "ArrowDown":
          event.preventDefault();
          if (!isOpen) {
            openDropdown();
          } else {
            setFocusedIndex((prev) =>
              prev < enabledOptions.length - 1 ? prev + 1 : 0,
            );
          }
          break;

        case "ArrowUp":
          event.preventDefault();
          if (!isOpen) {
            openDropdown();
          } else {
            setFocusedIndex((prev) =>
              prev > 0 ? prev - 1 : enabledOptions.length - 1,
            );
          }
          break;

        case "Escape":
          event.preventDefault();
          setIsOpen(false);
          setFocusedIndex(-1);
          buttonRef.current?.focus();
          break;

        case "Home":
          event.preventDefault();
          if (isOpen) {
            setFocusedIndex(0);
          }
          break;

        case "End":
          event.preventDefault();
          if (isOpen) {
            setFocusedIndex(enabledOptions.length - 1);
          }
          break;

        case "Tab":
          // Close dropdown on tab, let focus move naturally
          setIsOpen(false);
          setFocusedIndex(-1);
          break;
      }
    },
    [disabled, isOpen, focusedIndex, enabledOptions, handleSelect, openDropdown],
  );

  // Scroll focused option into view
  useEffect(() => {
    if (isOpen && focusedIndex >= 0 && listboxRef.current) {
      const focusedElement = listboxRef.current.children[
        focusedIndex
      ] as HTMLElement;
      focusedElement?.scrollIntoView({ block: "nearest" });
    }
  }, [focusedIndex, isOpen]);

  return (
    <div className={`relative ${className}`} ref={dropdownRef}>
      <button
        ref={buttonRef}
        type="button"
        role="combobox"
        aria-expanded={isOpen}
        aria-haspopup="listbox"
        aria-controls={listboxId}
        aria-label={ariaLabel}
        aria-activedescendant={
          isOpen && focusedIndex >= 0
            ? `${listboxId}-option-${focusedIndex}`
            : undefined
        }
        className={`px-2 py-1 text-sm font-semibold bg-mid-gray/10 border border-mid-gray/80 rounded min-w-[200px] text-left flex items-center justify-between transition-all duration-150 focus:outline-none focus-visible:ring-2 focus-visible:ring-logo-primary ${
          disabled
            ? "opacity-50 cursor-not-allowed"
            : "hover:bg-logo-primary/10 cursor-pointer hover:border-logo-primary"
        }`}
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        disabled={disabled}
      >
        <span className="truncate">{selectedOption?.label || placeholder}</span>
        <svg
          className={`w-4 h-4 ml-2 transition-transform duration-200 ${isOpen ? "transform rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>
      {isOpen && !disabled && (
        <div
          ref={listboxRef}
          id={listboxId}
          role="listbox"
          aria-label={ariaLabel}
          className="absolute top-full left-0 right-0 mt-1 bg-background border border-mid-gray/80 rounded shadow-lg z-50 max-h-60 overflow-y-auto"
        >
          {flatOptions.length === 0 ? (
            <div className="px-2 py-1 text-sm text-mid-gray" role="option" aria-disabled="true">
              {t("common.noOptionsFound")}
            </div>
          ) : isGroupedOptions(options) ? (
            // Render grouped options
            (() => {
              let globalIndex = 0;
              return options.map((group) => (
                <div key={group.label} role="group" aria-label={group.label}>
                  <div className="px-2 py-1 text-xs font-semibold text-mid-gray/70 uppercase tracking-wider bg-mid-gray/5 border-b border-mid-gray/20 sticky top-0">
                    {group.label}
                  </div>
                  {group.options.map((option) => {
                    const currentIndex = globalIndex++;
                    const isEnabled = !option.disabled;
                    const enabledIndex = enabledOptions.findIndex(
                      (opt) => opt.value === option.value,
                    );
                    return (
                      <div
                        key={option.value}
                        id={`${listboxId}-option-${currentIndex}`}
                        role="option"
                        aria-selected={selectedValue === option.value}
                        aria-disabled={option.disabled}
                        tabIndex={-1}
                        className={`w-full px-3 py-1.5 text-sm text-left transition-colors duration-150 cursor-pointer ${
                          focusedIndex === enabledIndex
                            ? "bg-logo-primary/20"
                            : "hover:bg-logo-primary/10"
                        } ${
                          selectedValue === option.value ? "font-semibold" : ""
                        } ${option.disabled ? "opacity-50 cursor-not-allowed" : ""}`}
                        onClick={() => isEnabled && handleSelect(option.value)}
                        onMouseEnter={() =>
                          isEnabled && setFocusedIndex(enabledIndex)
                        }
                      >
                        <span className="truncate">{option.label}</span>
                      </div>
                    );
                  })}
                </div>
              ));
            })()
          ) : (
            // Render flat options
            enabledOptions.map((option, index) => (
              <div
                key={option.value}
                id={`${listboxId}-option-${index}`}
                role="option"
                aria-selected={selectedValue === option.value}
                aria-disabled={option.disabled}
                tabIndex={-1}
                className={`w-full px-2 py-1 text-sm text-left transition-colors duration-150 cursor-pointer ${
                  focusedIndex === index
                    ? "bg-logo-primary/20"
                    : "hover:bg-logo-primary/10"
                } ${
                  selectedValue === option.value ? "font-semibold" : ""
                } ${option.disabled ? "opacity-50 cursor-not-allowed" : ""}`}
                onClick={() => !option.disabled && handleSelect(option.value)}
                onMouseEnter={() => setFocusedIndex(index)}
              >
                <span className="truncate">{option.label}</span>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
};
