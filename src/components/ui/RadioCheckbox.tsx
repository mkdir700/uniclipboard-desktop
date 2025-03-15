import React from "react";

interface RadioCheckboxProps {
  checked: boolean;
  onChange: () => void;
  label?: string;
  className?: string;
}

const RadioCheckbox: React.FC<RadioCheckboxProps> = ({
  checked,
  onChange,
  label,
  className = "",
}) => {
  return (
    <label className={`flex items-center cursor-pointer ${className}`}>
      <div className="relative flex items-center justify-center">
        <input
          type="checkbox"
          className="sr-only"
          checked={checked}
          onChange={onChange}
        />
        <div
          className={`h-4 w-4 rounded border ${
            checked ? "border-violet-500" : "border-gray-600 bg-gray-700"
          }`}
        ></div>
        {checked && (
          <div className="absolute w-2 h-2 bg-violet-500 rounded-full"></div>
        )}
      </div>
      {label && <span className="ml-2 text-sm text-gray-300">{label}</span>}
    </label>
  );
};

export default RadioCheckbox;
