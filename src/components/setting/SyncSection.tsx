import React, { useState } from "react";
import RadioCheckbox from "../ui/RadioCheckbox";
import Toggle from "../ui/Toggle";
import Select from "../ui/Select";
import Slider from "../ui/Slider";

const SyncSection: React.FC = () => {
  // 添加状态管理
  const [autoSync, setAutoSync] = useState(true);
  const [syncFrequency, setSyncFrequency] = useState("realtime");
  const [contentTypes, setContentTypes] = useState({
    text: true,
    image: true,
    link: true,
    file: true,
    codeSnippet: true,
    richText: true,
  });
  const [maxFileSize, setMaxFileSize] = useState(10);

  // 同步频率选项
  const syncFrequencyOptions = [
    { value: "realtime", label: "实时同步" },
    { value: "30s", label: "每30秒" },
    { value: "1m", label: "每分钟" },
    { value: "5m", label: "每5分钟" },
    { value: "15m", label: "每15分钟" },
  ];

  // 处理内容类型复选框变化
  const handleContentTypeChange = (type: keyof typeof contentTypes) => {
    setContentTypes((prev) => ({
      ...prev,
      [type]: !prev[type],
    }));
  };

  return (
    <div className="space-y-4">
      <div className="settings-item py-2 rounded-lg px-2">
        <Toggle
          checked={autoSync}
          onChange={() => setAutoSync(!autoSync)}
          label="自动同步"
          description="启用后，ClipSync将自动同步您复制的内容到所有设备"
        />
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={syncFrequencyOptions}
          value={syncFrequency}
          onChange={setSyncFrequency}
          label="同步频率"
          description="控制ClipSync检查新内容的频率"
          width="w-36"
        />
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <h4 className="text-sm font-medium text-white mb-2">同步内容类型</h4>
        <div className="grid grid-cols-2 gap-3">
          <RadioCheckbox
            checked={contentTypes.text}
            onChange={() => handleContentTypeChange("text")}
            label="文本"
          />
          <RadioCheckbox
            checked={contentTypes.image}
            onChange={() => handleContentTypeChange("image")}
            label="图片"
          />
          <RadioCheckbox
            checked={contentTypes.link}
            onChange={() => handleContentTypeChange("link")}
            label="链接"
          />
          <RadioCheckbox
            checked={contentTypes.file}
            onChange={() => handleContentTypeChange("file")}
            label="文件"
          />
          <RadioCheckbox
            checked={contentTypes.codeSnippet}
            onChange={() => handleContentTypeChange("codeSnippet")}
            label="代码片段"
          />
          <RadioCheckbox
            checked={contentTypes.richText}
            onChange={() => handleContentTypeChange("richText")}
            label="富文本"
          />
        </div>
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <Slider
          min={1}
          max={50}
          value={maxFileSize}
          onChange={setMaxFileSize}
          label="最大同步文件大小"
          description="限制单个文件的最大同步大小"
          unit="MB"
          keyPoints={[
            { value: 10, label: "10MB" },
            { value: 25, label: "25MB" },
            { value: 50, label: "50MB" },
          ]}
        />
      </div>
    </div>
  );
};

export default SyncSection;
