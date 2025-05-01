import React, { useState, useRef, useEffect } from "react";
import { ExclamationCircleIcon, EyeIcon, EyeSlashIcon } from "@heroicons/react/24/outline";

interface PasswordInputProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  description?: string;
  className?: string;
  placeholder?: string;
  disabled?: boolean;
  errorMessage?: string | null;
  showConfirmButton?: boolean;
  onConfirm?: () => void;
  confirmButtonText?: string;
  autoFocus?: boolean;
  id?: string;
}

const PasswordInput: React.FC<PasswordInputProps> = ({
  value,
  onChange,
  label,
  description,
  className = "",
  placeholder = "",
  disabled = false,
  errorMessage,
  showConfirmButton = false,
  onConfirm,
  confirmButtonText = "确认",
  autoFocus = false,
  id,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [touched, setTouched] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const inputId = id || `password-input-${Math.random().toString(36).substring(2, 9)}`;
  
  useEffect(() => {
    if (autoFocus && inputRef.current) {
      inputRef.current.focus();
    }
  }, [autoFocus]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  const handleBlur = () => {
    setIsFocused(false);
    setTouched(true);
  };

  const togglePasswordVisibility = () => {
    setShowPassword(!showPassword);
    // 保持输入框的焦点
    setTimeout(() => {
      inputRef.current?.focus();
    }, 0);
  };

  const handleConfirm = () => {
    if (onConfirm) {
      onConfirm();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && showConfirmButton && onConfirm) {
      onConfirm();
    }
  };

  const showError = touched && errorMessage;
  const rightPadding = showError ? 'pr-10' : '';
  
  return (
    <div className={`flex flex-col ${className}`}>
      <div className="flex items-center justify-between w-full">
        {(label || description) && (
          <div className="flex-grow mr-4">
            {label && (
              <label htmlFor={inputId} className="text-sm font-medium text-white">
                {label}
              </label>
            )}
            {description && (
              <p className="text-xs text-gray-400 mt-0.5" id={`${inputId}-description`}>
                {description}
              </p>
            )}
          </div>
        )}
        <div className={`relative ${showConfirmButton ? "flex space-x-2 items-center" : "min-w-[120px]"}`}>
          <div className="relative w-full">
            <input
              ref={inputRef}
              id={inputId}
              type={showPassword ? "text" : "password"}
              value={value}
              onChange={handleChange}
              placeholder={placeholder}
              disabled={disabled}
              className={`bg-gray-700 border rounded-lg px-3 py-1.5 text-sm text-white w-full transition-colors duration-200 
                ${isFocused ? "border-violet-400 outline-none" : showError ? "border-red-500" : "border-gray-700 hover:border-gray-600"}
                ${disabled ? "opacity-60 cursor-not-allowed" : ""} 
                ${rightPadding} pr-10`}
              onFocus={() => setIsFocused(true)}
              onBlur={handleBlur}
              onKeyDown={handleKeyDown}
              aria-invalid={showError ? "true" : "false"}
              aria-describedby={description ? `${inputId}-description` : undefined}
              {...(errorMessage && touched ? { "aria-errormessage": `${inputId}-error` } : {})}
            />
            {/* 显示/隐藏密码按钮 */}
            <button
              type="button"
              onClick={togglePasswordVisibility}
              className={`absolute inset-y-0 right-0 flex items-center pr-3 text-gray-400 hover:text-white transition-colors duration-200 
                ${showError ? "right-7" : ""}`}
              tabIndex={-1}
              disabled={disabled}
              aria-label={showPassword ? "隐藏密码" : "显示密码"}
            >
              {showPassword ? (
                <EyeSlashIcon className="h-5 w-5" aria-hidden="true" />
              ) : (
                <EyeIcon className="h-5 w-5" aria-hidden="true" />
              )}
            </button>
            {showError && (
              <div className="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                <ExclamationCircleIcon className="h-5 w-5 text-red-500" aria-hidden="true" />
              </div>
            )}
          </div>
          
          {showConfirmButton && (
            <button
              type="button"
              onClick={handleConfirm}
              disabled={disabled}
              className={`bg-violet-600 text-white text-sm font-medium px-3 py-1.5 rounded-lg whitespace-nowrap min-w-[60px] transition-colors duration-200
                ${disabled ? "opacity-60 cursor-not-allowed" : "hover:bg-violet-700 active:bg-violet-800"}`}
            >
              {confirmButtonText}
            </button>
          )}
        </div>
      </div>
      {showError && (
        <div className="mt-1 text-xs text-red-500 self-end" id={`${inputId}-error`} role="alert">
          {errorMessage}
        </div>
      )}
    </div>
  );
};

export default PasswordInput;