import React from "react";
import CurrentDevice from "./CurrentDevice";
import OtherDevice from "./OtherDevice";

const DeviceList: React.FC = () => {
  return (
    <>
      {/* 当前设备 */}
      <CurrentDevice />

      {/* 其他已连接设备 */}
      <OtherDevice />
    </>
  );
};

export default DeviceList;
