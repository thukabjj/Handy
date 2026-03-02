import React from "react";

interface DisabledNoticeProps {
  children: React.ReactNode;
  className?: string;
}

/**
 * Shared component for displaying a notice when a feature is disabled.
 */
export const DisabledNotice: React.FC<DisabledNoticeProps> = ({
  children,
  className = "",
}) => (
  <div
    className={`p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20 ${className}`}
  >
    <p className="text-sm text-mid-gray">{children}</p>
  </div>
);
