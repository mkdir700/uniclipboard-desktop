import React from "react";

interface SelectOption {
  value: string;
  label: string;
}

interface SelectProps {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  label?: string;
  description?: string;
  className?: string;
  width?: string;
  disabled?: boolean;
}

const Select: React.FC<SelectProps> = ({
  options,
  value,
  onChange,
  label,
  description,
  className = "",
  width = "w-full",
  disabled = false,
}) => {
  return (
    <div className={`flex items-center justify-between ${className} ${disabled ? 'opacity-60 cursor-not-allowed' : ''}`}>
      {(label || description) && (
        <div>
          {label && <h4 className="text-sm font-medium text-white">{label}</h4>}
          {description && (
            <p className="text-xs text-gray-400 mt-0.5">{description}</p>
          )}
        </div>
      )}
      <div className={width}>
        <select
          className={`w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white focus:outline-none ${
            disabled ? '' : 'focus:ring-1 focus:ring-violet-400'
          }`}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          disabled={disabled}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};

export default Select;
