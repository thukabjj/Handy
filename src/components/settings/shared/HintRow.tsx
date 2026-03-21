import React from "react";
import { Info } from "lucide-react";

interface HintRowProps {
  text: string;
  grouped?: boolean;
}

export const HintRow: React.FC<HintRowProps> = ({ text, grouped = true }) => {
  return (
    <div
      className={`px-4 py-2 text-xs text-mid-gray border-t border-mid-gray/20 ${
        grouped ? "" : "rounded-md border border-mid-gray/20"
      }`}
    >
      <div className="flex items-start gap-2">
        <Info className="h-3.5 w-3.5 mt-0.5 opacity-80" />
        <span>{text}</span>
      </div>
    </div>
  );
};

