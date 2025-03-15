import React, { useEffect, useState } from "react";
import RadioCheckbox from "../ui/RadioCheckbox";
import Toggle from "../ui/Toggle";
import Select from "../ui/Select";
import Slider from "../ui/Slider";
import { useSetting } from "../../contexts/SettingContext";

const SyncSection: React.FC = () => {
  // 使用设置上下文
  const { setting, loading, error, updateSyncSetting } = useSetting();

  // 本地状态，用于UI展示
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

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setAutoSync(setting.sync.auto_sync);
      setSyncFrequency(setting.sync.sync_frequency);
      setContentTypes({
        text: setting.sync.content_types.text,
        image: setting.sync.content_types.image,
        link: setting.sync.content_types.link,
        file: setting.sync.content_types.file,
        codeSnippet: setting.sync.content_types.code_snippet,
        richText: setting.sync.content_types.rich_text,
      });
      setMaxFileSize(setting.sync.max_file_size);
    }
  }, [setting]);

  // 处理自动同步开关变化
  const handleAutoSyncChange = () => {
    const newValue = !autoSync;
    setAutoSync(newValue);
    updateSyncSetting({ auto_sync: newValue });
  };

  // 处理同步频率变化
  const handleSyncFrequencyChange = (value: string) => {
    setSyncFrequency(value);
    updateSyncSetting({ sync_frequency: value });
  };

  // 处理内容类型复选框变化
  const handleContentTypeChange = (type: string) => {
    const newContentTypes = {
      ...contentTypes,
      [type]: !contentTypes[type as keyof typeof contentTypes],
    };
    setContentTypes(newContentTypes);

    // 将前端的contentTypes转换为后端的content_types格式
    updateSyncSetting({
      content_types: {
        text: newContentTypes.text,
        image: newContentTypes.image,
        link: newContentTypes.link,
        file: newContentTypes.file,
        code_snippet: newContentTypes.codeSnippet,
        rich_text: newContentTypes.richText,
      },
    });
  };

  // 处理最大文件大小变化
  const handleMaxFileSizeChange = (value: number) => {
    setMaxFileSize(value);
    updateSyncSetting({ max_file_size: value });
  };

  // 如果正在加载，显示加载状态
  // if (loading) {
  //   return <div className="text-center py-4">正在加载设置...</div>;
  // }

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <div className="space-y-4">
      <div className="settings-item py-2 rounded-lg px-2">
        <Toggle
          checked={autoSync}
          onChange={handleAutoSyncChange}
          label="自动同步"
          description="启用后，ClipSync将自动同步您复制的内容到所有设备"
        />
      </div>

      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={syncFrequencyOptions}
          value={syncFrequency}
          onChange={handleSyncFrequencyChange}
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
          onChange={handleMaxFileSizeChange}
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
