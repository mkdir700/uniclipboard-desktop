import React, { useState, useRef, useEffect } from "react";
import { ChevronDownIcon } from "@heroicons/react/24/outline";

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
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const selectedOption = options.find((option) => option.value === value);

  // 关闭下拉菜单的函数
  const closeDropdown = () => {
    setIsOpen(false);
  };

  // 处理点击外部关闭下拉菜单
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        closeDropdown();
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  // 处理键盘导航
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setHighlightedIndex((prev) =>
            prev < options.length - 1 ? prev + 1 : prev
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setHighlightedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case "Enter":
          e.preventDefault();
          if (options[highlightedIndex]) {
            onChange(options[highlightedIndex].value);
            closeDropdown();
          }
          break;
        case "Escape":
          e.preventDefault();
          closeDropdown();
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, highlightedIndex, options, onChange]);

  // 当选项变化时重置高亮索引
  useEffect(() => {
    if (isOpen) {
      const selectedIndex = options.findIndex(
        (option) => option.value === value
      );
      setHighlightedIndex(selectedIndex >= 0 ? selectedIndex : 0);
    }
  }, [isOpen, options, value]);

  return (
    <div
      className={`flex items-center justify-between ${className} ${
        disabled ? "opacity-60 cursor-not-allowed" : ""
      }`}
    >
      {(label || description) && (
        <div>
          {label && <h4 className="text-sm font-medium text-white">{label}</h4>}
          {description && (
            <p className="text-xs text-gray-400 mt-0.5">{description}</p>
          )}
        </div>
      )}
      <div className={width} ref={containerRef}>
        <div
          className={`relative w-full ${disabled ? "pointer-events-none" : ""}`}
          tabIndex={disabled ? -1 : 0}
        >
          {/* 自定义选择器按钮 */}
          <div
            className={`flex items-center justify-between w-full bg-gray-700 border border-gray-700 rounded-lg px-3 py-1.5 text-sm text-white ${
              disabled
                ? ""
                : "cursor-pointer focus:outline-none focus:ring-1 focus:ring-violet-400 hover:border-gray-600"
            }`}
            onClick={() => !disabled && setIsOpen((prev) => !prev)}
          >
            <span>{selectedOption?.label || "选择一个选项"}</span>
            <ChevronDownIcon
              className={`h-4 w-4 transition-transform ${
                isOpen ? "transform rotate-180" : ""
              }`}
            />
          </div>

          {/* 下拉菜单 */}
          {isOpen && !disabled && (
            <div className="absolute z-10 w-full mt-1 bg-gray-800 border border-gray-700 rounded-lg shadow-lg max-h-60 overflow-auto">
              {options.map((option, index) => (
                <div
                  key={option.value}
                  className={`px-3 py-1.5 text-sm cursor-pointer ${
                    index === highlightedIndex
                      ? "bg-violet-600 text-white"
                      : option.value === value
                      ? "bg-gray-700 text-white"
                      : "text-gray-200 hover:bg-gray-700"
                  }`}
                  onClick={() => {
                    onChange(option.value);
                    closeDropdown();
                  }}
                  onMouseEnter={() => setHighlightedIndex(index)}
                >
                  {option.label}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default Select;
