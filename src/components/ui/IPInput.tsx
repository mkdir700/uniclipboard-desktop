import React, { useState, useRef, useEffect } from "react";
import { ExclamationCircleIcon } from "@heroicons/react/24/outline";

interface IPInputProps {
  value: string;
  onChange: (value: string) => void;
  label?: string;
  description?: string;
  className?: string;
  disabled?: boolean;
  errorMessage?: string | null;
}

const IPInput: React.FC<IPInputProps> = ({
  value,
  onChange,
  label,
  description,
  className = "",
  disabled = false,
  errorMessage,
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const [touched, setTouched] = useState(false);
  
  // 将 IP 地址分解为四个部分
  const splitIP = (ip: string): string[] => {
    const parts = ip.split(".");
    return [
      parts[0] || "",
      parts[1] || "",
      parts[2] || "",
      parts[3] || ""
    ];
  };
  
  const [ipParts, setIpParts] = useState<string[]>(splitIP(value));
  
  // 创建四个输入框的引用
  const inputRefs = [
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null)
  ];
  
  // 当外部 value 变化时更新内部状态
  useEffect(() => {
    setIpParts(splitIP(value));
  }, [value]);
  
  // 验证单个 IP 部分是否有效
  const isValidIPPart = (part: string): boolean => {
    if (part === "") return false;
    const num = parseInt(part, 10);
    return !isNaN(num) && num >= 0 && num <= 255;
  };
  
  // 验证整个 IP 地址是否有效
  const isValidIP = (): boolean => {
    return ipParts.every(part => isValidIPPart(part));
  };
  
  // 更新 IP 部分并通知父组件
  const updateIP = (newParts: string[]): void => {
    setIpParts(newParts);
    const newIP = newParts.join(".");
    onChange(newIP);
  };
  
  // 处理单个 IP 部分的变化
  const handlePartChange = (index: number, value: string): void => {
    // 只允许输入数字
    if (!/^\d*$/.test(value)) return;
    
    // 限制最大长度为 3
    if (value.length > 3) return;
    
    // 限制最大值为 255
    const num = parseInt(value, 10);
    if (!isNaN(num) && num > 255) return;
    
    // 更新 IP 部分
    const newParts = [...ipParts];
    newParts[index] = value;
    updateIP(newParts);
  };
  
  // 处理键盘事件
  const handleKeyDown = (index: number, e: React.KeyboardEvent<HTMLInputElement>): void => {
    // 处理 Tab 键：如果按下 Tab 键且没有按下 Shift，则阻止默认行为并手动聚焦下一个输入框
    if (e.key === "Tab" && !e.shiftKey && index < 3) {
      e.preventDefault();
      inputRefs[index + 1].current?.focus();
    }
    
    // 处理 Shift+Tab 键：如果按下 Shift+Tab，则阻止默认行为并手动聚焦上一个输入框
    if (e.key === "Tab" && e.shiftKey && index > 0) {
      e.preventDefault();
      inputRefs[index - 1].current?.focus();
    }
    
    // 处理右箭头键：如果光标在最右边且按下右箭头键，则聚焦下一个输入框
    if (e.key === "ArrowRight") {
      const input = e.target as HTMLInputElement;
      if (input.selectionStart === input.value.length && index < 3) {
        inputRefs[index + 1].current?.focus();
      }
    }
    
    // 处理左箭头键：如果光标在最左边且按下左箭头键，则聚焦上一个输入框
    if (e.key === "ArrowLeft") {
      const input = e.target as HTMLInputElement;
      if (input.selectionStart === 0 && index > 0) {
        const prevInput = inputRefs[index - 1].current;
        if (prevInput) {
          prevInput.focus();
          // 将光标放在上一个输入框的末尾
          const length = prevInput.value.length;
          prevInput.setSelectionRange(length, length);
        }
      }
    }
    
    // 处理点号：如果按下点号，则聚焦下一个输入框
    if (e.key === "." && index < 3) {
      e.preventDefault();
      inputRefs[index + 1].current?.focus();
    }
    
    // 处理删除键：如果输入框为空且按下删除键，则聚焦上一个输入框
    if (e.key === "Backspace" && ipParts[index] === "" && index > 0) {
      inputRefs[index - 1].current?.focus();
    }
  };
  
  // 处理粘贴事件
  const handlePaste = (e: React.ClipboardEvent<HTMLInputElement>): void => {
    e.preventDefault();
    const pastedText = e.clipboardData.getData("text");
    
    // 尝试解析粘贴的文本为 IP 地址
    const ipRegex = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/;
    const match = pastedText.match(ipRegex);
    
    if (match) {
      const newParts = [match[1], match[2], match[3], match[4]];
      // 验证每个部分是否有效
      if (newParts.every(part => {
        const num = parseInt(part, 10);
        return !isNaN(num) && num >= 0 && num <= 255;
      })) {
        updateIP(newParts);
      }
    }
  };
  
  // 处理聚焦事件
  const handleFocus = (index: number): void => {
    setIsFocused(true);
    // 选中整个文本
    inputRefs[index].current?.select();
  };
  
  // 处理失焦事件
  const handleBlur = (): void => {
    setIsFocused(false);
    setTouched(true);
    
    // 如果 IP 地址有效，则格式化显示（去除前导零）
    if (isValidIP()) {
      const formattedParts = ipParts.map(part => 
        part ? parseInt(part, 10).toString() : ""
      );
      updateIP(formattedParts);
    }
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
        <div className="relative min-w-[180px]">
          <div className={`flex items-center bg-gray-700 border rounded-lg px-1 py-1 transition-colors duration-200 ${
            isFocused
              ? "border-violet-400"
              : showError
              ? "border-red-500"
              : "border-gray-700 hover:border-gray-600"
          } ${disabled ? "opacity-60 cursor-not-allowed" : ""}`}>
            {ipParts.map((part, index) => (
              <React.Fragment key={index}>
                <input
                  ref={inputRefs[index]}
                  type="text"
                  value={part}
                  onChange={(e) => handlePartChange(index, e.target.value)}
                  onKeyDown={(e) => handleKeyDown(index, e)}
                  onFocus={() => handleFocus(index)}
                  onBlur={handleBlur}
                  onPaste={index === 0 ? handlePaste : undefined}
                  disabled={disabled}
                  className="w-9 bg-transparent border-none outline-none text-center text-sm text-white font-mono"
                  maxLength={3}
                />
                {index < 3 && <span className="text-gray-400 mx-0.5">.</span>}
              </React.Fragment>
            ))}
          </div>
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

export default IPInput;
