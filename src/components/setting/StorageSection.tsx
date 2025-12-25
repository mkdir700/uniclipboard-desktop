import React, { useEffect, useState } from "react";
import { Slider, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { useTranslation } from "react-i18next";

const StorageSection: React.FC = () => {
  const { setting, error, updateStorageSetting } = useSetting();
  const { t } = useTranslation();

  // Local state
  const [historyRetentionDays, setHistoryRetentionDays] = useState(30);
  const [maxHistoryItems, setMaxHistoryItems] = useState(1000);
  const [autoClearHistory, setAutoClearHistory] = useState("never");

  // Auto clear options
  const autoClearOptions = [
    { value: "never", label: t("settings.sections.storage.autoClearHistory.never") },
    { value: "daily", label: t("settings.sections.storage.autoClearHistory.daily") },
    { value: "weekly", label: t("settings.sections.storage.autoClearHistory.weekly") },
    { value: "monthly", label: t("settings.sections.storage.autoClearHistory.monthly") },
    { value: "on_exit", label: t("settings.sections.storage.autoClearHistory.onExit") },
  ];

  // Max history items options
  const maxHistoryOptions = [
    { value: "100", label: t("settings.sections.storage.maxHistoryItems.items", { count: 100 }) },
    { value: "500", label: t("settings.sections.storage.maxHistoryItems.items", { count: 500 }) },
    { value: "1000", label: t("settings.sections.storage.maxHistoryItems.items", { count: 1000 }) },
    { value: "5000", label: t("settings.sections.storage.maxHistoryItems.items", { count: 5000 }) },
    { value: "0", label: t("settings.sections.storage.maxHistoryItems.unlimited") },
  ];

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setAutoClearHistory(setting.storage.auto_clear_history);
      setHistoryRetentionDays(setting.storage.history_retention_days);
      setMaxHistoryItems(setting.storage.max_history_items);
    }
  }, [setting]);

  // Handle history retention days change
  const handleHistoryRetentionChange = (value: number[]) => {
    const newValue = value[0];
    setHistoryRetentionDays(newValue);
    updateStorageSetting({ history_retention_days: newValue });
  };

  // Handle max history items change
  const handleMaxHistoryItemsChange = (value: string) => {
    const numValue = parseInt(value, 10);
    setMaxHistoryItems(numValue);
    updateStorageSetting({ max_history_items: numValue });
  };

  // Handle auto clear history option change
  const handleAutoClearHistoryChange = (value: string) => {
    setAutoClearHistory(value);
    updateStorageSetting({ auto_clear_history: value });
  };

  // Handle clear history
  const handleClearHistory = () => {
    // Logic to clear history can be added here
    // For example, call backend API to clear history
    alert(t("settings.sections.storage.clearHistory.cleared"));
  };

  // Show error message if there is an error
  if (error) {
    return <div className="text-red-500 py-4">{t("settings.sections.storage.loadError")}: {error}</div>;
  }

  return (
    <>
      {/* Auto clear rule */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.storage.autoClearHistory.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.autoClearHistory.description")}
            </p>
          </div>
          <Select
            value={autoClearHistory}
            onValueChange={handleAutoClearHistoryChange}
          >
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {autoClearOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Storage usage */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-base font-medium">{t("settings.sections.storage.storageUsage.label")}</h4>
          <span className="text-sm text-muted-foreground">128MB / 1GB</span>
        </div>
        <div className="w-full bg-secondary rounded-full h-2.5">
          <div
            className="bg-primary h-2.5 rounded-full"
            style={{ width: "12.8%" }}
          ></div>
        </div>
        <div className="flex justify-between mt-1 text-xs text-muted-foreground">
          <span>{t("settings.sections.storage.storageUsage.usage", { percentage: "12.8" })}</span>
        </div>
      </div>

      {/* Storage limit */}
      <div className="py-2 rounded-lg px-2">
        <div className="w-full space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-base font-medium">{t("settings.sections.storage.historyRetention.label")}</h4>
              <p className="text-sm text-muted-foreground">
                {t("settings.sections.storage.historyRetention.description")}
              </p>
            </div>
            <span className="text-sm text-muted-foreground">{t("settings.sections.storage.historyRetention.days", { days: historyRetentionDays })}</span>
          </div>
          <Slider
            min={1}
            max={90}
            step={1}
            value={[historyRetentionDays]}
            onValueChange={handleHistoryRetentionChange}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>{t("settings.sections.storage.historyRetention.days", { days: 7 })}</span>
            <span>{t("settings.sections.storage.historyRetention.days", { days: 30 })}</span>
            <span>{t("settings.sections.storage.historyRetention.days", { days: 60 })}</span>
            <span>{t("settings.sections.storage.historyRetention.days", { days: 90 })}</span>
          </div>
        </div>
      </div>

      {/* Max history items */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.storage.maxHistoryItems.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.maxHistoryItems.description")}
            </p>
          </div>
          <Select
            value={maxHistoryItems.toString()}
            onValueChange={handleMaxHistoryItemsChange}
          >
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {maxHistoryOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Clear history */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.storage.clearHistory.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.clearHistory.description")}
            </p>
          </div>
          <button
            className="px-3 py-1.5 bg-destructive/10 hover:bg-destructive/20 text-sm text-destructive rounded-lg transition duration-150"
            onClick={handleClearHistory}
          >
            {t("settings.sections.storage.clearHistory.button")}
          </button>
        </div>
      </div>
    </>
  );
};

export default StorageSection;
