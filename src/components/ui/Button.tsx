import React from "react";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger" | "ghost" | "danger-ghost";
  size?: "sm" | "md" | "lg";
  /** For icon-only buttons, provide an aria-label for accessibility */
  "aria-label"?: string;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  className = "",
  variant = "primary",
  size = "md",
  ...props
}) => {
  // Base classes for all buttons - consistent foundation
  const baseClasses = [
    // Typography and layout
    "font-medium inline-flex items-center justify-center gap-2",
    // Border and shape - using design tokens
    "rounded-button border",
    // Interactive states
    "transition-colors duration-150",
    "focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary-light",
    // Disabled state
    "disabled:opacity-50 disabled:cursor-not-allowed",
    // Cursor
    "cursor-pointer",
  ].join(" ");

  // Variant classes - each variant has consistent styles
  const variantClasses: Record<string, string> = {
    primary: [
      "text-white",
      "bg-background-ui border-background-ui",
      "hover:bg-background-ui/80 hover:border-background-ui/80",
      "active:bg-background-ui/70",
    ].join(" "),
    secondary: [
      "text-text",
      "bg-mid-gray/10 border-mid-gray/20",
      "hover:bg-primary-light/20 hover:border-primary-light/50",
      "active:bg-primary-light/30",
    ].join(" "),
    danger: [
      "text-white",
      "bg-red-600 border-red-600",
      "hover:bg-red-700 hover:border-red-700",
      "active:bg-red-800",
    ].join(" "),
    ghost: [
      "text-text",
      "bg-transparent border-transparent",
      "hover:bg-mid-gray/10 hover:border-mid-gray/20",
      "active:bg-mid-gray/20",
    ].join(" "),
    "danger-ghost": [
      "text-red-500",
      "bg-transparent border-transparent",
      "hover:bg-red-500/10 hover:border-red-500/20",
      "active:bg-red-500/20",
    ].join(" "),
  };

  // Size classes - consistent padding and text sizes
  const sizeClasses: Record<string, string> = {
    sm: "px-2 py-1 text-xs min-h-[28px]",
    md: "px-4 py-1.5 text-sm min-h-[32px]",
    lg: "px-5 py-2 text-base min-h-[40px]",
  };

  return (
    <button
      className={`${baseClasses} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
};
