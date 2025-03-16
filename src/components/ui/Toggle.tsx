import React from "react";

interface ToggleProps {
  checked: boolean;
  onChange: () => void;
  label?: string;
  description?: string;
  className?: string;
  disabled?: boolean;
}

const Toggle: React.FC<ToggleProps> = ({
  checked,
  onChange,
  label,
  description,
  className = "",
  disabled = false,
}) => {
  return (
    <div className={`flex items-center justify-between ${className}`}>
      {(label || description) && (
        <div>
          {label && <h4 className="text-sm font-medium text-white">{label}</h4>}
          {description && (
            <p className="text-xs text-gray-400 mt-0.5">{description}</p>
          )}
        </div>
      )}
      <label
        className={`flex items-center ${
          disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"
        }`}
      >
        <div className="relative">
          <input
            type="checkbox"
            className="sr-only"
            checked={checked}
            onChange={onChange}
            disabled={disabled}
          />
          <div
            className={`toggle-bg w-11 h-6 rounded-full transition-colors duration-200 ease-in-out ${
              checked ? "bg-violet-500" : "bg-gray-600"
            }`}
          >
            <div
              className={`absolute top-1 left-1 bg-white w-4 h-4 rounded-full transition-transform duration-200 ease-in-out ${
                checked ? "transform translate-x-5" : ""
              }`}
            ></div>
          </div>
        </div>
      </label>
    </div>
  );
};

export default Toggle;
