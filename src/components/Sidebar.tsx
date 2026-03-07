import React from "react";
import { useTranslation } from "react-i18next";
import {
  FlaskConical,
  History,
  Info,
  Mic,
  Settings,
  Bug,
} from "lucide-react";
import type { AppSettings } from "@/bindings";
import HandyTextLogo from "./icons/HandyTextLogo";
import { useSettings } from "../hooks/useSettings";
import {
  HistorySettings,
  DebugSettings,
  AboutSettings,
  HomeSettings,
  LabsSettings,
  UnifiedSettings,
} from "./settings";

export type SidebarSection = keyof typeof SECTIONS_CONFIG;

interface IconProps {
  width?: number | string;
  height?: number | string;
  size?: number | string;
  className?: string;
  strokeWidth?: number | string;
  color?: string;
}

interface SectionConfig {
  labelKey: string;
  icon: React.ComponentType<IconProps>;
  component: React.ComponentType;
  enabled: (settings: AppSettings | null) => boolean;
  badge?: "beta" | "new";
}

export const SECTIONS_CONFIG = {
  home: {
    labelKey: "sidebar.home",
    icon: Mic,
    component: HomeSettings,
    enabled: () => true,
  },
  history: {
    labelKey: "sidebar.history",
    icon: History,
    component: HistorySettings,
    enabled: () => true,
  },
  settings: {
    labelKey: "sidebar.settings",
    icon: Settings,
    component: UnifiedSettings,
    enabled: () => true,
  },
  labs: {
    labelKey: "sidebar.labs",
    icon: FlaskConical,
    component: LabsSettings,
    enabled: () => true,
    badge: "beta" as const,
  },
  about: {
    labelKey: "sidebar.about",
    icon: Info,
    component: AboutSettings,
    enabled: () => true,
  },
  debug: {
    labelKey: "sidebar.debug",
    icon: Bug,
    component: DebugSettings,
    enabled: (settings) => settings?.debug_mode ?? false,
  },
} as const satisfies Record<string, SectionConfig>;

interface SidebarProps {
  activeSection: SidebarSection;
  onSectionChange: (section: SidebarSection) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  activeSection,
  onSectionChange,
}) => {
  const { t } = useTranslation();
  const { settings } = useSettings();

  const availableSections = Object.entries(SECTIONS_CONFIG)
    .filter(([_, config]) => config.enabled(settings))
    .map(([id, config]) => ({ id: id as SidebarSection, ...config }));

  return (
    <div className="flex flex-col w-40 h-full border-r border-mid-gray/20 items-center px-2">
      <HandyTextLogo width={120} className="m-4" />
      <div className="flex flex-col w-full items-center gap-1 pt-2 border-t border-mid-gray/20">
        {availableSections.map((section) => {
          const Icon = section.icon;
          const isActive = activeSection === section.id;
          const badge = "badge" in section ? section.badge : undefined;

          return (
            <div
              key={section.id}
              className={`flex gap-2 items-center p-2 w-full rounded-lg cursor-pointer transition-colors ${
                isActive
                  ? "bg-logo-primary/80 text-logo-primary"
                  : "hover:bg-mid-gray/20 hover:opacity-100 opacity-85"
              }`}
              onClick={() => onSectionChange(section.id)}
            >
              <Icon width={24} height={24} className="shrink-0" />
              <div className="flex items-center gap-1.5 min-w-0 flex-1">
                <p
                  className="text-sm font-medium truncate"
                  title={t(section.labelKey)}
                >
                  {t(section.labelKey)}
                </p>
                {badge && (
                  <span className="px-1 py-0.5 text-[9px] font-medium bg-purple-500/20 text-purple-400 rounded shrink-0">
                    {badge}
                  </span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
