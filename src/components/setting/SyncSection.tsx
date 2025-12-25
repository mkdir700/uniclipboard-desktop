import React, { useEffect, useState } from "react";
import {
  Switch,
  Label,
  Slider,

  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui";
import { useSetting } from "../../contexts/SettingContext";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";

const SyncSection: React.FC = () => {
  const { t } = useTranslation();
  // Use setting context
  const { setting, error, updateSyncSetting } = useSetting();

  // Local state for UI display
  const [autoSync, setAutoSync] = useState(true);
  const [syncFrequency, setSyncFrequency] = useState("realtime");

  const [maxFileSize, setMaxFileSize] = useState([10]);

  // Sync frequency options
  const syncFrequencyOptions = [
    { value: "realtime", label: t("settings.sections.sync.syncFrequency.realtime") },
    { value: "30s", label: t("settings.sections.sync.syncFrequency.30s") },
    { value: "1m", label: t("settings.sections.sync.syncFrequency.1m") },
    { value: "5m", label: t("settings.sections.sync.syncFrequency.5m") },
    { value: "15m", label: t("settings.sections.sync.syncFrequency.15m") },
  ];

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setAutoSync(setting.sync.auto_sync);
      setSyncFrequency(setting.sync.sync_frequency);

      setMaxFileSize([setting.sync.max_file_size]);
    }
  }, [setting]);

  // Handle auto sync switch change
  const handleAutoSyncChange = (checked: boolean) => {
    setAutoSync(checked);
    updateSyncSetting({ auto_sync: checked });
  };

  // Handle sync frequency change
  const handleSyncFrequencyChange = (value: string) => {
    setSyncFrequency(value);
    updateSyncSetting({ sync_frequency: value });
  };

  // Handle max file size change
  const handleMaxFileSizeChange = (value: number[]) => {
    setMaxFileSize(value);
    updateSyncSetting({ max_file_size: value[0] });
  };

  // Show error message if any
  if (error) {
    return <div className="text-destructive py-4">{t("settings.sections.sync.loadError")} {error}</div>;
  }

  return (
    <div className="space-y-6">
      {/* Auto sync switch */}
      <div className="flex items-center justify-between py-2 rounded-lg px-2">
        <div className="space-y-0.5">
          <Label htmlFor="auto-sync" className="text-base">
            {t("settings.sections.sync.autoSync.label")}
          </Label>
          <p className="text-sm text-muted-foreground">
            {t("settings.sections.sync.autoSync.description")}
          </p>
        </div>
        <Switch
          id="auto-sync"
          checked={autoSync}
          onCheckedChange={handleAutoSyncChange}
        />
      </div>

      {/* Sync frequency selection */}
      <div className="flex items-center justify-between py-2 rounded-lg px-2">
        <div className="space-y-0.5">
          <Label htmlFor="sync-frequency" className="text-base">
            {t("settings.sections.sync.syncFrequency.label")}
          </Label>
          <p className="text-sm text-muted-foreground">
            {t("settings.sections.sync.syncFrequency.description")}
          </p>
        </div>
        <Select value={syncFrequency} onValueChange={handleSyncFrequencyChange}>
          <SelectTrigger className="w-52">
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

      {/* Max file size slider */}
      <div className="py-2 rounded-lg px-2 space-y-4">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label className="text-base">{t("settings.sections.sync.maxFileSize.label")}</Label>
            <span className="text-sm text-muted-foreground">
              {maxFileSize[0]} {t("settings.sections.sync.maxFileSize.unit")}
            </span>
          </div>
          <p className="text-sm text-muted-foreground">
            {t("settings.sections.sync.maxFileSize.description")}
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
