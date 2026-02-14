import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Search } from "lucide-react";
import {
  useSettingsSearch,
  type SearchableItem,
} from "@/hooks/useSettingsSearch";
import type { SidebarSection } from "@/components/Sidebar";

interface SettingsSearchProps {
  onNavigate: (section: SidebarSection) => void;
}

function SettingsSearch({ onNavigate }: SettingsSearchProps) {
  const { t } = useTranslation();
  const { results, query, setQuery, isOpen, setIsOpen } =
    useSettingsSearch();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const isMac =
    typeof navigator !== "undefined" &&
    navigator.userAgent.toLowerCase().includes("mac");

  // Auto-focus input when opened
  useEffect(() => {
    if (isOpen) {
      setSelectedIndex(0);
      // Small delay to ensure the DOM is ready
      requestAnimationFrame(() => {
        inputRef.current?.focus();
      });
    }
  }, [isOpen]);

  // Reset selection when results change
  useEffect(() => {
    setSelectedIndex(0);
  }, [results]);

  // Scroll selected item into view
  useEffect(() => {
    if (!listRef.current) return;
    const items = listRef.current.querySelectorAll("[data-search-item]");
    const selected = items[selectedIndex];
    if (selected) {
      selected.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  const handleSelect = useCallback(
    (item: SearchableItem) => {
      onNavigate(item.section);
      setIsOpen(false);
    },
    [onNavigate, setIsOpen],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev < results.length - 1 ? prev + 1 : 0,
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev > 0 ? prev - 1 : results.length - 1,
          );
          break;
        case "Enter":
          e.preventDefault();
          if (results[selectedIndex]) {
            handleSelect(results[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          setIsOpen(false);
          break;
      }
    },
    [results, selectedIndex, handleSelect, setIsOpen],
  );

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
      onClick={() => setIsOpen(false)}
    >
      <div
        className="w-full max-w-lg"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="bg-background/80 backdrop-blur-xl border border-mid-gray/20 rounded-xl shadow-2xl overflow-hidden">
          {/* Search input */}
          <div className="flex items-center gap-3 px-4 py-3 border-b border-mid-gray/20">
            <Search className="w-5 h-5 text-mid-gray shrink-0" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t("settings.search.placeholder", "Search settings...")}
              className="flex-1 bg-transparent text-text text-sm outline-none placeholder:text-mid-gray"
            />
          </div>

          {/* Results list */}
          <div
            ref={listRef}
            className="max-h-[40vh] overflow-y-auto py-1"
          >
            {results.length === 0 ? (
              <div className="px-4 py-6 text-center text-sm text-mid-gray">
                {t("settings.search.noResults", "No results found")}
              </div>
            ) : (
              results.map((item, index) => (
                <div
                  key={item.id}
                  data-search-item
                  className={`flex items-start gap-3 px-4 py-2.5 cursor-pointer transition-colors ${
                    index === selectedIndex
                      ? "bg-primary-light/15"
                      : "hover:bg-mid-gray/10"
                  }`}
                  onClick={() => handleSelect(item)}
                  onMouseEnter={() => setSelectedIndex(index)}
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold text-text truncate">
                        {item.label}
                      </span>
                      <span className="shrink-0 text-[10px] font-medium px-1.5 py-0.5 rounded bg-mid-gray/15 text-text-secondary">
                        {item.sectionLabel}
                      </span>
                    </div>
                    {item.description && (
                      <p className="text-xs text-text-secondary mt-0.5 truncate">
                        {item.description}
                      </p>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>

          {/* Footer hint */}
          <div className="flex items-center justify-between px-4 py-2 border-t border-mid-gray/20 text-[11px] text-mid-gray">
            <div className="flex items-center gap-3">
              <span>
                <kbd className="px-1 py-0.5 rounded bg-mid-gray/15 font-mono text-[10px]">
                  {"\u2191"}
                </kbd>{" "}
                <kbd className="px-1 py-0.5 rounded bg-mid-gray/15 font-mono text-[10px]">
                  {"\u2193"}
                </kbd>{" "}
                {t("settings.search.navigate", "navigate")}
              </span>
              <span>
                <kbd className="px-1 py-0.5 rounded bg-mid-gray/15 font-mono text-[10px]">
                  {"\u21B5"}
                </kbd>{" "}
                {t("settings.search.select", "select")}
              </span>
              <span>
                {/* eslint-disable-next-line i18next/no-literal-string */}
                <kbd className="px-1 py-0.5 rounded bg-mid-gray/15 font-mono text-[10px]">
                  esc
                </kbd>{" "}
                {t("settings.search.close", "close")}
              </span>
            </div>
            <span>
              {isMac ? "\u2318" : "Ctrl+"}K
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default SettingsSearch;
