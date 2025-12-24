import React, { useState } from "react";
import { DeviceList, DeviceHeader, DeviceFooter } from "@/components";
import { Rules, Permissions, PairingModal } from "@/components/device";
import { MainLayout } from "@/layouts";

const DevicesPage: React.FC = () => {
  const [showPairingModal, setShowPairingModal] = useState(false);

  const handleAddDevice = () => {
    setShowPairingModal(true);
  };

  const handleClosePairingModal = () => {
    setShowPairingModal(false);
  };

  return (
    <MainLayout>
      {/* 顶部标题栏 */}
      <DeviceHeader addDevice={handleAddDevice} />

      {/* 设备列表 */}
      <div className="flex-1 overflow-hidden mt-4">
        <div className="h-full px-8 pb-3 overflow-auto scrollbar-thin">
          {/* 设备列表 */}
          <DeviceList />

          {/* 设备规则 */}
          <Rules />

          {/* 权限管理 */}
          <Permissions />
        </div>
      </div>

      {/* 设备管理底部 */}
      <DeviceFooter />

      {/* 设备配对模态框 */}
      <PairingModal open={showPairingModal} onClose={handleClosePairingModal} />
    </MainLayout>
  );
};

export default DevicesPage;
