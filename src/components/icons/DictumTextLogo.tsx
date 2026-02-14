import React from "react";

const DictumTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="0 0 200 48"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="dictum-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="var(--color-primary, #1E3A8A)" />
          <stop offset="100%" stopColor="var(--color-primary-light, #3B82F6)" />
        </linearGradient>
      </defs>
      <text
        x="0"
        y="36"
        fontFamily="Inter, -apple-system, BlinkMacSystemFont, sans-serif"
        fontWeight="700"
        fontSize="38"
        letterSpacing="-0.02em"
        fill="url(#dictum-gradient)"
      >
        Dictum
      </text>
    </svg>
  );
};

export default DictumTextLogo;
