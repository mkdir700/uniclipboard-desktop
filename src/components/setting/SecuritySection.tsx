import React, { useEffect, useState } from "react";
import { Switch, Input } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { setEncryptionPassword, getEncryptionPassword } from "@/api/security";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff } from "lucide-react";

const SecuritySection: React.FC = () => {
  const { t } = useTranslation();
  const { setting, error, updateSecuritySetting } = useSetting();

  // Local state
  const [endToEndEncryption, setEndToEndEncryption] = useState(true);
  const [encryptionPassword, setEncryptionPasswordInput] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  // Debounce save password
  useEffect(() => {
    const timer = setTimeout(() => {
      setEncryptionPassword(encryptionPassword);
    }, 500);
    return () => clearTimeout(timer);
  }, [encryptionPassword]);

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
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.security.encryptionPassword.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.security.encryptionPassword.description")}
            </p>
          </div>
          <div className="relative flex items-center">
            <Input
              type={showPassword ? "text" : "password"}
              value={encryptionPassword}
              onChange={(e) => setEncryptionPasswordInput(e.target.value)}
              placeholder={t("settings.sections.security.encryptionPassword.placeholder")}
              className="w-64 pr-10"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2 text-muted-foreground hover:text-foreground transition-colors"
            >
              {showPassword ? <EyeOff size={18} /> : <Eye size={18} />}
            </button>
          </div>
        </div>
      </div>
    </>
  );
};

export default SecuritySection;
