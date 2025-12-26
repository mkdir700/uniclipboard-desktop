import React, { useEffect, useState } from "react";
import { Slider, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";
import { Card, CardContent } from "@/components/ui/card";
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
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.storage.autoClearHistory.label")}
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="flex items-center justify-between gap-4 py-2">
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.autoClearHistory.description")}
            </p>
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
        </CardContent>
      </Card>

      {/* Storage usage */}
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.storage.storageUsage.label")}
          </h3>
          <span className="text-sm text-muted-foreground">128MB / 1GB</span>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="space-y-2 py-2">
            <div className="w-full bg-secondary rounded-full h-2.5">
              <div
                className="bg-primary h-2.5 rounded-full"
                style={{ width: "12.8%" }}
              ></div>
            </div>
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>{t("settings.sections.storage.storageUsage.usage", { percentage: "12.8" })}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* History retention */}
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.storage.historyRetention.label")}
          </h3>
          <span className="text-sm text-muted-foreground">{t("settings.sections.storage.historyRetention.days", { days: historyRetentionDays })}</span>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="space-y-4 py-2">
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.historyRetention.description")}
            </p>
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
        </CardContent>
      </Card>

      {/* Max history items */}
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.storage.maxHistoryItems.label")}
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="flex items-center justify-between gap-4 py-2">
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.maxHistoryItems.description")}
            </p>
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
        </CardContent>
      </Card>

      {/* Clear history */}
      <Card>
        <div className="flex items-center gap-4 mb-4 px-6 pt-6">
          <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">
            {t("settings.sections.storage.clearHistory.label")}
          </h3>
          <div className="h-px flex-1 bg-border/50"></div>
        </div>
        <CardContent className="pt-0">
          <div className="flex items-center justify-between gap-4 py-2">
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.storage.clearHistory.description")}
            </p>
            <button
              className="px-3 py-1.5 bg-destructive/10 hover:bg-destructive/20 text-sm text-destructive rounded-lg transition duration-150"
              onClick={handleClearHistory}
            >
              {t("settings.sections.storage.clearHistory.button")}
            </button>
          </div>
        </CardContent>
      </Card>
    </>
  );
};

export default StorageSection;
