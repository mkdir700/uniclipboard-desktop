import React, { useEffect, useState } from "react";
import { Switch, Input, Button } from "@/components/ui";
import { useSetting } from "@/contexts/SettingContext";
import { setEncryptionPassword, getEncryptionPassword } from "@/api/security";

const SecuritySection: React.FC = () => {
  const { setting, error, updateSecuritySetting } = useSetting();

  // 本地状态
  const [endToEndEncryption, setEndToEndEncryption] = useState(true);
  const [encryptionPassword, setEncryptionPasswordInput] = useState("");
  const [showPasswordInput, setShowPasswordInput] = useState(false);

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setEndToEndEncryption(setting.security.end_to_end_encryption);
      getEncryptionPassword().then((password) => {
        setEncryptionPasswordInput(password || "");
      });
    }
  }, [setting]);

  // 处理端到端加密开关变化
  const handleEndToEndEncryptionChange = (checked: boolean) => {
    const newValue = checked;
    setEndToEndEncryption(newValue);
    updateSecuritySetting({ end_to_end_encryption: newValue });
  };

  // 处理加密密码保存
  const handleSavePassword = () => {
    setEncryptionPassword(encryptionPassword);
    setShowPasswordInput(false);
  };

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <>
      {/* 端到端加密 */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">端到端加密</h4>
            <p className="text-sm text-muted-foreground">
              启用后，所有同步内容将使用端到端加密传输
            </p>
          </div>
          <Switch
            checked={endToEndEncryption}
            onCheckedChange={handleEndToEndEncryptionChange}
          />
        </div>
      </div>

      {/* 加密口令 */}
      <div className="py-2 rounded-lg px-2">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <h4 className="text-base font-medium">加密口令</h4>
              <p className="text-sm text-muted-foreground">
                用于加解密数据
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowPasswordInput(!showPasswordInput)}
            >
              {showPasswordInput ? "取消" : "修改"}
            </Button>
          </div>

          {showPasswordInput && (
            <div className="flex gap-2 mt-2">
              <Input
                type="password"
                value={encryptionPassword}
                onChange={(e) => setEncryptionPasswordInput(e.target.value)}
                placeholder="输入新的加密口令"
                className="flex-1"
              />
              <Button
                size="sm"
                onClick={handleSavePassword}
              >
                保存
              </Button>
            </div>
          )}
        </div>
      </div>
    </>
  );
};

export default SecuritySection;
