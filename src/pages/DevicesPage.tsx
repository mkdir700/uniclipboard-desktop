import React, { useState } from "react";
import {
  DeviceList,
  DeviceRules,
  DevicePermissions,
  DevicePairingModal,
  DeviceHeader,
  DeviceFooter,
} from "../components";
import { MainLayout } from "../layouts";

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
        <div className="h-full px-4 pb-3 overflow-auto hide-scrollbar">
          {/* 设备列表 */}
          <DeviceList />

          {/* 设备规则 */}
          <DeviceRules />

          {/* 权限管理 */}
          <DevicePermissions />
        </div>
      </div>

      {/* 设备管理底部 */}
      <DeviceFooter />

      {/* 设备配对模态框 */}
      {showPairingModal && (
        <DevicePairingModal onClose={handleClosePairingModal} />
      )}
    </MainLayout>
  );
};

export default DevicesPage;
