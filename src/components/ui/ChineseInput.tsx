import React, { useState, useRef, useEffect } from "react";

interface ChineseInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
}

const ChineseInput: React.FC<ChineseInputProps> = ({
  value,
  onChange,
  placeholder = "",
  className = "",
  disabled = false,
}) => {
  const [isComposing, setIsComposing] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // 同步外部值与内部DOM
  useEffect(() => {
    if (inputRef.current && !isComposing && inputRef.current.value !== value) {
      inputRef.current.value = value;
    }
  }, [value, isComposing]);

  return (
    <input
      ref={inputRef}
      type="text"
      defaultValue={value}
      onChange={(e) => {
        // 只有在不是中文输入法组合状态时才更新父组件
        if (!isComposing) {
          onChange(e.target.value);
        }
      }}
      onCompositionStart={() => {
        setIsComposing(true);
      }}
      onCompositionEnd={(e) => {
        setIsComposing(false);
        // 在组合结束时提交最终值
        onChange((e.target as HTMLInputElement).value);
      }}
      placeholder={placeholder}
      disabled={disabled}
      className={className}
      autoComplete="off"
      autoCorrect="off"
      autoCapitalize="off"
      spellCheck="false"
      data-form-type="other"
    />
  );
};

export default ChineseInput;
