import React, { useEffect, useState } from "react";
import {
  Switch,
  Label,
  Slider,
  Checkbox,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui";
import { useSetting } from "../../contexts/SettingContext";
import { cn } from "@/lib/utils";

const SyncSection: React.FC = () => {
  // 使用设置上下文
  const { setting, error, updateSyncSetting } = useSetting();

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
  const [maxFileSize, setMaxFileSize] = useState([10]);

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
      setMaxFileSize([setting.sync.max_file_size]);
    }
  }, [setting]);

  // 处理自动同步开关变化
  const handleAutoSyncChange = (checked: boolean) => {
    setAutoSync(checked);
    updateSyncSetting({ auto_sync: checked });
  };

  // 处理同步频率变化
  const handleSyncFrequencyChange = (value: string) => {
    setSyncFrequency(value);
    updateSyncSetting({ sync_frequency: value });
  };

  // 处理内容类型复选框变化
  const handleContentTypeChange = (type: string, checked: boolean) => {
    const newContentTypes = {
      ...contentTypes,
      [type]: checked,
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
  const handleMaxFileSizeChange = (value: number[]) => {
    setMaxFileSize(value);
    updateSyncSetting({ max_file_size: value[0] });
  };

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-destructive py-4">加载设置失败: {error}</div>;
  }

  return (
    <div className="space-y-6">
      {/* 自动同步开关 */}
      <div className="flex items-center justify-between py-2 rounded-lg px-2">
        <div className="space-y-0.5">
          <Label htmlFor="auto-sync" className="text-base">
            自动同步
          </Label>
          <p className="text-sm text-muted-foreground">
            启用后，uniClipboard将自动同步您复制的内容到所有设备
          </p>
        </div>
        <Switch
          id="auto-sync"
          checked={autoSync}
          onCheckedChange={handleAutoSyncChange}
        />
      </div>

      {/* 同步频率选择 */}
      <div className="py-2 rounded-lg px-2 space-y-2">
        <Label htmlFor="sync-frequency" className="text-base">
          同步频率
        </Label>
        <p className="text-sm text-muted-foreground mb-2">
          控制 uniClipboard 检查新内容的频率
        </p>
        <Select value={syncFrequency} onValueChange={handleSyncFrequencyChange}>
          <SelectTrigger className="w-[200px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {syncFrequencyOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* 同步内容类型 */}
      <div className="py-2 rounded-lg px-2 space-y-3">
        <Label className="text-base">同步内容类型</Label>
        <div className="grid grid-cols-2 gap-3">
          {Object.entries({
            text: "文本",
            image: "图片",
            link: "链接",
            file: "文件",
            codeSnippet: "代码片段",
            richText: "富文本",
          }).map(([key, label]) => (
            <div key={key} className="flex items-center space-x-2">
              <Checkbox
                id={`content-type-${key}`}
                checked={contentTypes[key as keyof typeof contentTypes]}
                onCheckedChange={(checked) =>
                  handleContentTypeChange(key, checked as boolean)
                }
              />
              <Label
                htmlFor={`content-type-${key}`}
                className="cursor-pointer"
              >
                {label}
              </Label>
            </div>
          ))}
        </div>
      </div>

      {/* 最大文件大小滑块 */}
      <div className="py-2 rounded-lg px-2 space-y-4">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-base">最大同步文件大小</Label>
            <span className="text-sm text-muted-foreground">
              {maxFileSize[0]} MB
            </span>
          </div>
          <p className="text-sm text-muted-foreground">
            限制单个文件的最大同步大小
          </p>
        </div>
        <Slider
          min={1}
          max={50}
          step={1}
          value={maxFileSize}
          onValueChange={handleMaxFileSizeChange}
          className="w-full"
        />
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>1MB</span>
          <span className={cn(maxFileSize[0] >= 10 && "text-foreground font-medium")}>
            10MB
          </span>
          <span className={cn(maxFileSize[0] >= 25 && "text-foreground font-medium")}>
            25MB
          </span>
          <span className={cn(maxFileSize[0] >= 50 && "text-foreground font-medium")}>
            50MB
          </span>
        </div>
      </div>
    </div>
  );
};

export default SyncSection;
