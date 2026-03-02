import React from "react";
import { ToggleSwitch } from "@/components/ui";

type BadgeType = "beta" | "new";

interface FeatureCardProps {
  title: string;
  description: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  badge?: BadgeType;
  icon?: React.ReactNode;
  children?: React.ReactNode;
  disabled?: boolean;
}

const BADGE_STYLES: Record<BadgeType, string> = {
  beta: "bg-purple-500/20 text-purple-400",
  new: "bg-green-500/20 text-green-400",
};

/**
 * FeatureCard - A card component for Labs features with toggle.
 * Children are shown inline when the feature is enabled.
 */
export const FeatureCard: React.FC<FeatureCardProps> = ({
  title,
  description,
  enabled,
  onToggle,
  badge,
  icon,
  children,
  disabled = false,
}) => {
  return (
    <div
      className={`rounded-lg border border-mid-gray/20 bg-mid-gray/5 overflow-hidden ${disabled ? "opacity-50 cursor-not-allowed" : ""}`}
    >
      <div className="p-4">
        <div className="flex items-start gap-3">
          {icon && (
            <div className="mt-0.5 text-mid-gray shrink-0">{icon}</div>
          )}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <h3 className="font-medium text-text">{title}</h3>
              {badge && (
                <span
                  className={`px-1.5 py-0.5 text-[10px] font-medium rounded ${BADGE_STYLES[badge]}`}
                >
                  {badge}
                </span>
              )}
            </div>
            <p className="text-sm text-mid-gray">{description}</p>
            {/* Inline children when enabled */}
            {enabled && children}
          </div>
          <div className="shrink-0">
            <ToggleSwitch
              label=""
              description=""
              checked={enabled}
              onChange={onToggle}
              disabled={disabled}
            />
          </div>
        </div>
      </div>
    </div>
  );
};
