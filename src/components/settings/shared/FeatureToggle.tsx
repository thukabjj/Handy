import React from "react";
import { ToggleSwitch } from "@/components/ui";

type BadgeType = "beta" | "new";

interface FeatureToggleProps {
  enabled: boolean;
  onChange: (enabled: boolean) => void;
  title: string;
  description: string;
  badge?: BadgeType;
  grouped?: boolean;
  disabled?: boolean;
}

const BADGE_STYLES: Record<BadgeType, string> = {
  beta: "bg-purple-500/20 text-purple-400",
  new: "bg-green-500/20 text-green-400",
};

/**
 * Shared component for feature toggles with optional badges.
 * Used in Labs section for experimental features.
 *
 * Note: Badge is displayed in the title (e.g., "Feature [beta]")
 * since ToggleSwitch only accepts string labels.
 */
export const FeatureToggle: React.FC<FeatureToggleProps> = ({
  enabled,
  onChange,
  title,
  description,
  badge,
  grouped = false,
  disabled = false,
}) => {
  // Badge is appended to label text since ToggleSwitch expects string
  const label = badge ? `${title} [${badge}]` : title;

  return (
    <ToggleSwitch
      label={label}
      description={description}
      checked={enabled}
      onChange={onChange}
      grouped={grouped}
      disabled={disabled}
    />
  );
};
