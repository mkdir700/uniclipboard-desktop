import React from "react";
import CurrentDevice from "./CurrentDevice";
import OtherDevice from "./OtherDevice";

interface DeviceProps {
  type: "current" | "connected" | "other";
  name: string;
  deviceType: "mobile" | "laptop" | "tablet" | "desktop";
  lastActive?: string;
  status: "online" | "offline" | "syncing";
  batteryLevel?: number;
  osInfo: string;
}

const DeviceIcon: React.FC<{ deviceType: string }> = ({ deviceType }) => {
  let bgColor = "bg-blue-500/20";
  let iconContent = (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      className="h-6 w-6 text-blue-400"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"
      />
    </svg>
  );

  if (deviceType === "laptop") {
    bgColor = "bg-blue-500/20";
    iconContent = (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        className="h-6 w-6 text-blue-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          d="M12 18h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"
        />
      </svg>
    );
  } else if (deviceType === "desktop") {
    bgColor = "bg-purple-500/20";
    iconContent = (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        className="h-6 w-6 text-purple-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
        />
      </svg>
    );
  } else if (deviceType === "tablet") {
    bgColor = "bg-green-500/20";
    iconContent = (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        className="h-6 w-6 text-green-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          d="M12 18h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"
        />
      </svg>
    );
  }

  return (
    <div
      className={`flex-shrink-0 h-12 w-12 ${bgColor} rounded-lg flex items-center justify-center`}
    >
      {iconContent}
    </div>
  );
};

const StatusBadge: React.FC<{ status: string }> = ({ status }) => {
  let bgColor = "bg-green-500/20";
  let textColor = "text-green-400";
  let statusText = "在线";
  let statusIcon = (
    <div className="w-2 h-2 rounded-full bg-green-500 mr-1"></div>
  );

  if (status === "offline") {
    bgColor = "bg-gray-500/20";
    textColor = "text-gray-400";
    statusText = "离线";
    statusIcon = <div className="w-2 h-2 rounded-full bg-gray-500 mr-1"></div>;
  } else if (status === "syncing") {
    bgColor = "bg-yellow-500/20";
    textColor = "text-yellow-400";
    statusText = "同步中";
    statusIcon = (
      <svg
        xmlns="http://www.w3.org/2000/svg"
        className="h-3 w-3 mr-1 text-yellow-400 animate-spin"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
        />
      </svg>
    );
  }

  return (
    <span
      className={`text-xs px-2 py-1 ${bgColor} rounded-full ${textColor} flex items-center`}
    >
      {statusIcon}
      {statusText}
    </span>
  );
};

const BatteryIndicator: React.FC<{ level: number }> = ({ level }) => {
  let color = "text-green-400";
  let icon = (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      className={`h-4 w-4 ${color}`}
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M5 12H3v8a2 2 0 002 2h14a2 2 0 002-2v-8h-2m-2-4h-4v4h-4v-4H7V6a2 2 0 012-2h6a2 2 0 012 2v2z"
      />
    </svg>
  );

  if (level < 20) {
    color = "text-red-400";
  } else if (level < 50) {
    color = "text-yellow-400";
  }

  return (
    <div className="flex items-center text-xs text-gray-400">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        className={`h-4 w-4 mr-1 ${color}`}
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth="2"
          d="M13 10V3L4 14h7v7l9-11h-7z"
        />
      </svg>
      {level}%
    </div>
  );
};

const Device: React.FC<DeviceProps> = ({
  type,
  name,
  deviceType,
  lastActive,
  status,
  batteryLevel,
  osInfo,
}) => {
  return (
    <div className="bg-gray-900 rounded-lg border border-gray-800/50 p-4 flex items-center justify-between mb-3 hover:border-gray-700/80 transition duration-200">
      <div className="flex items-center">
        <DeviceIcon deviceType={deviceType} />
        <div className="ml-4">
          <h4 className="font-medium text-white">{name}</h4>
          <p className="text-sm text-gray-400">{osInfo}</p>
          {lastActive && type !== "current" && (
            <p className="text-xs text-gray-500 mt-1">上次活跃: {lastActive}</p>
          )}
        </div>
      </div>
      <div className="flex items-center space-x-3">
        <StatusBadge status={status} />
        {batteryLevel !== undefined && (
          <BatteryIndicator level={batteryLevel} />
        )}
        <button className="text-gray-400 hover:text-white">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
            />
          </svg>
        </button>
      </div>
    </div>
  );
};

const DeviceList: React.FC = () => {
  // 模拟设备数据
  const currentDevice: DeviceProps = {
    type: "current",
    name: "我的MacBook Pro",
    deviceType: "laptop",
    status: "online",
    batteryLevel: 78,
    osInfo: "macOS 12.0.1",
  };

  const connectedDevices: DeviceProps[] = [
    {
      type: "connected",
      name: "iPhone 13 Pro",
      deviceType: "mobile",
      lastActive: "刚刚",
      status: "online",
      batteryLevel: 42,
      osInfo: "iOS 15.1",
    },
    {
      type: "connected",
      name: "工作站",
      deviceType: "desktop",
      lastActive: "10分钟前",
      status: "syncing",
      osInfo: "Windows 11 Pro",
    },
    {
      type: "connected",
      name: "iPad Pro",
      deviceType: "tablet",
      lastActive: "1小时前",
      status: "offline",
      batteryLevel: 15,
      osInfo: "iPadOS 15.1",
    },
  ];

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
