const DictumIcon = ({
  width,
  height,
  className,
}: {
  width?: number | string;
  height?: number | string;
  className?: string;
}) => (
  <svg
    width={width || 24}
    height={height || 24}
    viewBox="0 0 24 24"
    fill="none"
    className={className}
    xmlns="http://www.w3.org/2000/svg"
  >
    <defs>
      <linearGradient id="dictum-icon-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
        <stop offset="0%" stopColor="var(--color-primary, #1E3A8A)" />
        <stop offset="60%" stopColor="var(--color-primary-light, #3B82F6)" />
        <stop offset="100%" stopColor="var(--color-secondary, #F97316)" />
      </linearGradient>
    </defs>
    {/* Abstract speech/quote mark icon */}
    <path
      d="M12 2C6.48 2 2 5.58 2 10c0 2.24 1.12 4.27 2.94 5.72L4 20l4.28-2.14C9.47 18.27 10.7 18.5 12 18.5c5.52 0 10-3.58 10-8S17.52 2 12 2z"
      fill="url(#dictum-icon-gradient)"
      opacity="0.9"
    />
    {/* Inner speech wave lines */}
    <path
      d="M8 9.5h8M8 12.5h5"
      stroke="white"
      strokeWidth="1.5"
      strokeLinecap="round"
    />
  </svg>
);

export default DictumIcon;
