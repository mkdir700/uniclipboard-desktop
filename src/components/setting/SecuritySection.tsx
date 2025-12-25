import React, { useEffect, useState } from "react";
import { Switch, Input, Button } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { setEncryptionPassword, getEncryptionPassword } from "@/api/security";
import { useTranslation } from "react-i18next";

const SecuritySection: React.FC = () => {
  const { t } = useTranslation();
  const { setting, error, updateSecuritySetting } = useSetting();

  // Local state
  const [endToEndEncryption, setEndToEndEncryption] = useState(true);
  const [encryptionPassword, setEncryptionPasswordInput] = useState("");
  const [showPasswordInput, setShowPasswordInput] = useState(false);

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setEndToEndEncryption(setting.security.end_to_end_encryption);
      getEncryptionPassword().then((password) => {
        setEncryptionPasswordInput(password || "");
      });
    }
  }, [setting]);

  // Handle end-to-end encryption toggle change
  const handleEndToEndEncryptionChange = (checked: boolean) => {
    const newValue = checked;
    setEndToEndEncryption(newValue);
    updateSecuritySetting({ end_to_end_encryption: newValue });
  };

  // Handle encryption password save
  const handleSavePassword = () => {
    setEncryptionPassword(encryptionPassword);
    setShowPasswordInput(false);
  };

  // Display error message if there is an error
  if (error) {
    return <div className="text-red-500 py-4">{t("settings.sections.security.loadError")}: {error}</div>;
  }

  return (
    <>
      {/* End-to-end encryption */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.security.endToEndEncryption.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.security.endToEndEncryption.description")}
            </p>
          </div>
          <Switch
            checked={endToEndEncryption}
            onCheckedChange={handleEndToEndEncryptionChange}
          />
        </div>
      </div>

      {/* Encryption password */}
      <div className="py-2 rounded-lg px-2">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-base font-medium">{t("settings.sections.security.encryptionPassword.label")}</h4>
              <p className="text-sm text-muted-foreground">
                {t("settings.sections.security.encryptionPassword.description")}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowPasswordInput(!showPasswordInput)}
            >
              {showPasswordInput ? t("settings.sections.security.encryptionPassword.cancel") : t("settings.sections.security.encryptionPassword.modify")}
            </Button>
          </div>

          {showPasswordInput && (
            <div className="flex gap-2 mt-2">
              <Input
                type="password"
                value={encryptionPassword}
                onChange={(e) => setEncryptionPasswordInput(e.target.value)}
                placeholder={t("settings.sections.security.encryptionPassword.placeholder")}
                className="flex-1"
              />
              <Button
                size="sm"
                onClick={handleSavePassword}
              >
                {t("settings.sections.security.encryptionPassword.save")}
              </Button>
            </div>
          )}
        </div>
      </div>
    </>
  );
};

export default SecuritySection;
