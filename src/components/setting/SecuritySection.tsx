import React, { useEffect, useState } from "react";
import Toggle from "@/components/ui/Toggle";
import { useSetting } from "@/contexts/SettingContext";
import PasswordInput from "@/components/ui/PasswordInput";
import { setEncryptionPassword, getEncryptionPassword } from "@/api/security";

const SecuritySection: React.FC = () => {
  const { setting, error, updateSecuritySetting } = useSetting();

  // 本地状态
  const [endToEndEncryption, setEndToEndEncryption] = useState(true);
  const [encryptionPassword, setEncryptionPasswordInput] = useState("");

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
  const handleEndToEndEncryptionChange = () => {
    const newValue = !endToEndEncryption;
    setEndToEndEncryption(newValue);
    updateSecuritySetting({ end_to_end_encryption: newValue });
  };

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <>
      {/* 端到端加密 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Toggle
          checked={endToEndEncryption}
          onChange={handleEndToEndEncryptionChange}
          label="端到端加密"
          description="启用后，所有同步内容将使用端到端加密传输"
        />
      </div>
      <div className="settings-item py-2 rounded-lg px-2">
        <PasswordInput
          label="加密口令"
          description="用于加解密数据"
          value={encryptionPassword}
          showConfirmButton={true}
          onChange={(value) => {
            setEncryptionPasswordInput(value);
          }}
          onConfirm={() => {
            setEncryptionPassword(encryptionPassword);
          }}
        />
      </div>
    </>
  );
};

export default SecuritySection;
