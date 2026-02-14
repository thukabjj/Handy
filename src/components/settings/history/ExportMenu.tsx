import React, { useState, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Download } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { commands, type ExportFormat } from "@/bindings";

const EXPORT_FORMATS: { value: ExportFormat; labelKey: string }[] = [
  { value: "txt", labelKey: "settings.history.exportFormats.txt" },
  { value: "srt", labelKey: "settings.history.exportFormats.srt" },
  { value: "vtt", labelKey: "settings.history.exportFormats.vtt" },
  { value: "json", labelKey: "settings.history.exportFormats.json" },
  { value: "markdown", labelKey: "settings.history.exportFormats.markdown" },
];

const FORMAT_EXTENSIONS: Record<ExportFormat, string> = {
  txt: "txt",
  srt: "srt",
  vtt: "vtt",
  json: "json",
  markdown: "md",
};

interface ExportMenuProps {
  entryId: number;
}

export const ExportMenu: React.FC<ExportMenuProps> = ({ entryId }) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleExport = useCallback(
    async (format: ExportFormat) => {
      setExporting(true);
      setIsOpen(false);

      try {
        const filenameResult = await commands.getExportFilename(entryId, format);
        if (filenameResult.status !== "ok") return;

        const filePath = await save({
          defaultPath: filenameResult.data,
          filters: [
            {
              name: t(
                EXPORT_FORMATS.find((f) => f.value === format)?.labelKey ?? "",
              ),
              extensions: [FORMAT_EXTENSIONS[format]],
            },
          ],
        });

        if (!filePath) return;

        const contentResult = await commands.exportTranscription(
          entryId,
          format,
        );
        if (contentResult.status !== "ok") {
          console.error("Export failed:", contentResult.error);
          return;
        }

        await writeTextFile(filePath, contentResult.data);
      } catch (error) {
        console.error("Failed to export:", error);
      } finally {
        setExporting(false);
      }
    },
    [entryId, t],
  );

  return (
    <div className="relative" ref={menuRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={exporting}
        className="text-text/50 hover:text-primary-light transition-colors cursor-pointer disabled:opacity-50"
        title={t("settings.history.exportAs")}
      >
        <Download width={16} height={16} />
      </button>
      {isOpen && (
        <div className="absolute right-0 top-full mt-1 bg-background border border-mid-gray/20 rounded-lg shadow-lg z-50 min-w-[180px] py-1">
          {EXPORT_FORMATS.map((format) => (
            <button
              key={format.value}
              onClick={() => handleExport(format.value)}
              className="w-full px-3 py-1.5 text-sm text-left hover:bg-primary-light/10 transition-colors cursor-pointer"
            >
              {t(format.labelKey)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

interface ExportAllButtonProps {
  entryIds: number[];
}

export const ExportAllButton: React.FC<ExportAllButtonProps> = ({
  entryIds,
}) => {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleExportAll = useCallback(
    async (format: ExportFormat) => {
      setExporting(true);
      setIsOpen(false);

      try {
        const defaultFilename = await commands.getBatchExportFilename(format);

        const filePath = await save({
          defaultPath: defaultFilename,
          filters: [
            {
              name: t(
                EXPORT_FORMATS.find((f) => f.value === format)?.labelKey ?? "",
              ),
              extensions: [FORMAT_EXTENSIONS[format]],
            },
          ],
        });

        if (!filePath) return;

        const contentResult = await commands.exportTranscriptions(
          entryIds,
          format,
        );
        if (contentResult.status !== "ok") {
          console.error("Batch export failed:", contentResult.error);
          return;
        }

        await writeTextFile(filePath, contentResult.data);
      } catch (error) {
        console.error("Failed to export:", error);
      } finally {
        setExporting(false);
      }
    },
    [entryIds, t],
  );

  if (entryIds.length === 0) return null;

  return (
    <div className="relative" ref={menuRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={exporting}
        className="flex items-center gap-2 px-2 py-1 text-xs font-medium rounded-button border bg-mid-gray/10 border-mid-gray/20 hover:bg-primary-light/20 hover:border-primary-light/50 transition-colors cursor-pointer disabled:opacity-50"
      >
        <Download className="w-4 h-4" />
        <span>{t("settings.history.exportAll")}</span>
      </button>
      {isOpen && (
        <div className="absolute right-0 top-full mt-1 bg-background border border-mid-gray/20 rounded-lg shadow-lg z-50 min-w-[180px] py-1">
          {EXPORT_FORMATS.map((format) => (
            <button
              key={format.value}
              onClick={() => handleExportAll(format.value)}
              className="w-full px-3 py-1.5 text-sm text-left hover:bg-primary-light/10 transition-colors cursor-pointer"
            >
              {t(format.labelKey)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
