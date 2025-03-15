import React, { useState } from "react";

interface KeyPoint {
  value: number;
  label?: string;
}

interface SliderProps {
  min: number;
  max: number;
  value: number;
  onChange: (value: number) => void;
  label?: string;
  description?: string;
  unit?: string;
  className?: string;
  keyPoints?: KeyPoint[];
}

const Slider: React.FC<SliderProps> = ({
  min,
  max,
  value,
  onChange,
  label,
  description,
  unit = "",
  className = "",
  keyPoints = [],
}) => {
  const [hovering, setHovering] = useState(false);

  // 计算滑块位置的百分比
  const percentage = ((value - min) / (max - min)) * 100;

  // 处理关键点点击
  const handleKeyPointClick = (pointValue: number) => {
    onChange(pointValue);
  };

  return (
    <div className={`${className}`}>
      {(label || description || unit) && (
        <div className="mb-3">
          <div className="flex items-center justify-between">
            {label && (
              <h4 className="text-sm font-medium text-white">{label}</h4>
            )}
            {unit && (
              <div className="bg-gray-700 px-2 py-0.5 rounded text-xs font-medium text-violet-300">
                {value}
                {unit}
              </div>
            )}
          </div>
          {description && (
            <p className="text-xs text-gray-400 mt-0.5 mb-1">{description}</p>
          )}
          <div
            className="relative w-full h-7 flex items-center"
            onMouseEnter={() => setHovering(true)}
            onMouseLeave={() => setHovering(false)}
          >
            {/* 背景轨道 */}
            <div className="absolute w-full h-2 bg-gray-700 rounded-full"></div>

            {/* 已填充的轨道 */}
            <div
              className="absolute h-2 bg-violet-500 rounded-full transition-all duration-150"
              style={{ width: `${percentage}%` }}
            ></div>

            {/* 关键点 */}
            {keyPoints.map((point, index) => {
              const pointPercentage = ((point.value - min) / (max - min)) * 100;
              return (
                <div
                  key={index}
                  className="absolute"
                  style={{ left: `${pointPercentage}%` }}
                >
                  <div
                    className="w-3 h-3 bg-gray-800 border-2 border-violet-400 rounded-full cursor-pointer transform -translate-x-1/2 hover:scale-110 transition-transform duration-150"
                    onClick={() => handleKeyPointClick(point.value)}
                    title={point.label || `${point.value}${unit}`}
                  ></div>
                  {point.label && (
                    <div className="absolute -bottom-6 transform -translate-x-1/2 text-xs text-gray-400">
                      {point.label}
                    </div>
                  )}
                </div>
              );
            })}

            {/* 滑块手柄 */}
            <div
              className={`absolute w-4 h-4 bg-white rounded-full shadow-md transform -translate-x-1/2 transition-all duration-150 ${
                hovering ? "scale-110" : ""
              }`}
              style={{ left: `${percentage}%` }}
            ></div>

            {/* 实际的输入元素（隐藏但可交互） */}
            <input
              type="range"
              min={min}
              max={max}
              value={value}
              onChange={(e) => onChange(parseInt(e.target.value))}
              className="absolute w-full h-7 opacity-0 cursor-pointer"
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default Slider;
