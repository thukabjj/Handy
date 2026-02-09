import React from "react";

interface SkeletonProps {
  className?: string;
  /** Width of the skeleton. Can be a Tailwind class or custom value */
  width?: string;
  /** Height of the skeleton. Can be a Tailwind class or custom value */
  height?: string;
  /** Whether to show as a circular skeleton */
  circle?: boolean;
  /** Animation style */
  animation?: "pulse" | "shimmer" | "none";
}

/**
 * Skeleton loading placeholder component
 *
 * Usage:
 * <Skeleton width="w-32" height="h-4" /> - Text line
 * <Skeleton width="w-full" height="h-10" /> - Button
 * <Skeleton width="w-12" height="h-12" circle /> - Avatar
 */
export const Skeleton: React.FC<SkeletonProps> = ({
  className = "",
  width = "w-full",
  height = "h-4",
  circle = false,
  animation = "pulse",
}) => {
  const baseClasses = "bg-mid-gray/20";

  const animationClasses = {
    pulse: "animate-pulse",
    shimmer:
      "relative overflow-hidden before:absolute before:inset-0 before:-translate-x-full before:animate-[shimmer_2s_infinite] before:bg-gradient-to-r before:from-transparent before:via-white/20 before:to-transparent",
    none: "",
  };

  const shapeClasses = circle ? "rounded-full" : "rounded";

  return (
    <div
      className={`${baseClasses} ${animationClasses[animation]} ${shapeClasses} ${width} ${height} ${className}`}
      aria-hidden="true"
      role="presentation"
    />
  );
};

/**
 * Skeleton container for a card-like loading state
 */
export const SkeletonCard: React.FC<{ className?: string }> = ({
  className = "",
}) => {
  return (
    <div
      className={`p-4 rounded-card border border-mid-gray/20 space-y-3 ${className}`}
    >
      <Skeleton width="w-2/3" height="h-5" />
      <Skeleton width="w-full" height="h-4" />
      <Skeleton width="w-4/5" height="h-4" />
    </div>
  );
};

/**
 * Skeleton for settings items
 */
export const SkeletonSettingRow: React.FC<{ className?: string }> = ({
  className = "",
}) => {
  return (
    <div
      className={`flex items-center justify-between py-2 ${className}`}
      role="presentation"
      aria-hidden="true"
    >
      <div className="space-y-1.5 flex-1">
        <Skeleton width="w-32" height="h-4" />
        <Skeleton width="w-48" height="h-3" />
      </div>
      <Skeleton width="w-20" height="h-8" />
    </div>
  );
};

/**
 * Skeleton for model cards in onboarding
 */
export const SkeletonModelCard: React.FC<{ className?: string }> = ({
  className = "",
}) => {
  return (
    <div
      className={`p-4 rounded-card border border-mid-gray/20 ${className}`}
      role="presentation"
      aria-hidden="true"
    >
      <div className="flex items-start gap-4">
        <Skeleton width="w-12" height="h-12" circle />
        <div className="flex-1 space-y-2">
          <Skeleton width="w-24" height="h-5" />
          <Skeleton width="w-48" height="h-4" />
          <div className="flex gap-4 mt-3">
            <Skeleton width="w-16" height="h-3" />
            <Skeleton width="w-16" height="h-3" />
          </div>
        </div>
        <Skeleton width="w-24" height="h-9" />
      </div>
    </div>
  );
};
