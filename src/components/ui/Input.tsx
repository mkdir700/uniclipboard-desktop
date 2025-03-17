import React, { useState } from "react";
import { ExclamationCircleIcon } from "@heroicons/react/24/outline";

interface InputProps {
  value: string | number;
  onChange: (value: string) => void;
  label?: string;
  description?: string;
  className?: string;
  placeholder?: string;
  type?: string;
  disabled?: boolean;
  min?: number;
  max?: number;
  errorMessage?: string | null;
}

const Input: React.FC<InputProps> = ({
  value,
  onChange,
  label,
  description,
  className = "",
  placeholder = "",
  type = "text",
  disabled = false,
  min,
  max,
  errorMessage,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [touched, setTouched] = useState(false);
  
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  const handleBlur = () => {
    setIsFocused(false);
    setTouched(true);
  };

  const showError = touched && errorMessage;
  
  return (
    <div className={`flex flex-col ${className}`}>
      <div className="flex items-center justify-between w-full">
        {(label || description) && (
          <div className="flex-grow mr-4">
            {label && <h4 className="text-sm font-medium text-white">{label}</h4>}
            {description && (
              <p className="text-xs text-gray-400 mt-0.5">{description}</p>
            )}
          </div>
        )}
        <div className="relative min-w-[120px]">
          <input
            type={type}
            value={value}
            onChange={handleChange}
            placeholder={placeholder}
            disabled={disabled}
            min={min}
            max={max}
            className={`bg-gray-700 border rounded-lg px-3 py-1.5 text-sm text-white w-full transition-colors duration-200 ${
              isFocused
                ? "border-violet-400 outline-none"
                : showError
                ? "border-red-500"
                : "border-gray-700 hover:border-gray-600"
            } ${disabled ? "opacity-60 cursor-not-allowed" : ""}`}
            onFocus={() => setIsFocused(true)}
            onBlur={handleBlur}
          />
          {showError && (
            <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
              <ExclamationCircleIcon className="h-5 w-5 text-red-500" aria-hidden="true" />
            </div>
          )}
        </div>
      </div>
      {showError && (
        <div className="mt-1 text-xs text-red-500 self-end">
          {errorMessage}
        </div>
      )}
    </div>
  );
};

export default Input;
