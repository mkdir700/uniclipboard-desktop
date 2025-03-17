import React, { useEffect, useState } from "react";
import { useSetting } from "../../contexts/SettingContext";
import Input from "../ui/Input";
import Select from "../ui/Select";
import Toggle from "../ui/Toggle";
import IPInput from "../ui/IPInput";

const NetworkSection: React.FC = () => {
  const { setting, error, updateNetworkSetting } = useSetting();

  // 本地状态
  const [syncMethod, setSyncMethod] = useState("lan_first");
  const [cloudServer, setCloudServer] = useState("api.clipsync.com");
  const [webserverPort, setWebserverPort] = useState(29217);
  const [portError, setPortError] = useState<string | null>(null);

  // 自定义同步节点状态
  const [customPeerDevice, setCustomPeerDevice] = useState(false);
  const [peerDeviceAddr, setPeerDeviceAddr] = useState("192.168.1.100");
  const [peerDevicePort, setPeerDevicePort] = useState(29217);
  const [peerIpError, setPeerIpError] = useState<string | null>(null);
  const [peerPortError, setPeerPortError] = useState<string | null>(null);

  // 同步方式选项
  const syncMethodOptions = [
    { value: "lan_first", label: "优先使用局域网同步 (推荐)" },
    { value: "cloud_only", label: "仅使用云端同步" },
    { value: "lan_only", label: "仅使用局域网同步" },
  ];

  // 当设置加载完成后，更新本地状态
  useEffect(() => {
    if (setting) {
      setSyncMethod(setting.network.sync_method);
      setCloudServer(setting.network.cloud_server);
      setWebserverPort(setting.network.webserver_port);

      // 加载自定义同步节点设置
      setCustomPeerDevice(setting.network.custom_peer_device || false);
      setPeerDeviceAddr(setting.network.peer_device_addr || "192.168.1.100");
      setPeerDevicePort(setting.network.peer_device_port || 29217);
    }
  }, [setting]);

  // 处理同步方式变化
  const handleSyncMethodChange = (value: string) => {
    setSyncMethod(value);
    updateNetworkSetting({ sync_method: value });
  };

  // 处理本机开放端口变化
  const handleWebserverPortChange = (value: string) => {
    // 如果输入为空，不做任何处理，允许用户继续输入
    if (!value.trim()) {
      setPortError(null);
      setWebserverPort(0); // 临时设置为0，但不更新到设置中
      return;
    }

    // 检查是否为数字
    if (!/^\d+$/.test(value)) {
      setPortError("请输入有效的端口号");
      setWebserverPort(parseInt(value) || 0); // 即使有错误也更新显示值
      return;
    }

    const port = parseInt(value);
    setWebserverPort(port); // 无论如何都更新显示值

    // 验证端口范围
    if (port < 1024 || port > 65535) {
      setPortError("端口号必须在 1024-65535 之间");
      return;
    }

    // 验证通过
    setPortError(null);
    updateNetworkSetting({ webserver_port: port });
  };

  // 处理自定义同步节点开关变化
  const handleCustomPeerDeviceChange = () => {
    const newValue = !customPeerDevice;
    setCustomPeerDevice(newValue);
    updateNetworkSetting({ custom_peer_device: newValue });

    // 如果关闭，清除错误状态
    if (!newValue) {
      setPeerIpError(null);
      setPeerPortError(null);
    }
  };

  // 验证 IPv4 地址
  const validateIPv4 = (ip: string): boolean => {
    const ipv4Regex =
      /^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
    return ipv4Regex.test(ip);
  };

  // 处理节点 IP 变化
  const handlePeerDeviceAddrChange = (value: string) => {
    setPeerDeviceAddr(value);

    // 验证 IP 地址
    if (!value.trim()) {
      setPeerIpError("IP 地址不能为空");
      return;
    }

    if (!validateIPv4(value)) {
      setPeerIpError("请输入有效的 IPv4 地址");
      return;
    }

    // 验证通过
    setPeerIpError(null);
    updateNetworkSetting({ peer_device_addr: value });
  };

  // 处理节点端口变化
  const handlePeerDevicePortChange = (value: string) => {
    // 如果输入为空，不做任何处理，允许用户继续输入
    if (!value.trim()) {
      setPeerPortError("端口号不能为空");
      setPeerDevicePort(0); // 临时设置为0，但不更新到设置中
      return;
    }

    // 检查是否为数字
    if (!/^\d+$/.test(value)) {
      setPeerPortError("请输入有效的端口号");
      setPeerDevicePort(parseInt(value) || 0); // 即使有错误也更新显示值
      return;
    }

    const port = parseInt(value);
    setPeerDevicePort(port); // 无论如何都更新显示值

    // 验证端口范围
    if (port < 1024 || port > 65535) {
      setPeerPortError("端口号必须在 1024-65535 之间");
      return;
    }

    // 验证通过
    setPeerPortError(null);
    updateNetworkSetting({ peer_device_port: port });
  };

  // 如果正在加载，显示加载状态
  //   if (loading) {
  //     return <div className="text-center py-4">正在加载设置...</div>;
  //   }

  // 如果有错误，显示错误信息
  if (error) {
    return <div className="text-red-500 py-4">加载设置失败: {error}</div>;
  }

  return (
    <div className="space-y-4">
      {/* 同步方式 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Select
          options={syncMethodOptions}
          value={syncMethod}
          onChange={handleSyncMethodChange}
          label="同步方式"
          description="选择同步方式"
          width="w-64"
        />
      </div>

      {/* 本机开放端口 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Input
          label="本机开放端口"
          description="设置本机开放端口 (1024-65535)"
          value={webserverPort.toString()}
          onChange={handleWebserverPortChange}
          type="text"
          errorMessage={portError}
        />
      </div>

      {/* 是否自定义同步节点 */}
      <div className="settings-item py-2 rounded-lg px-2">
        <Toggle
          label="自定义同步节点"
          description="手动指定要同步的设备地址和端口"
          checked={customPeerDevice}
          onChange={handleCustomPeerDeviceChange}
        />
      </div>

      {/* 节点 IP 和端口（仅在启用自定义同步节点时显示） */}
      {customPeerDevice && (
        <>
          {/* 节点 IP */}
          <div className="settings-item py-2 rounded-lg px-2 ml-4 border-l-2 border-gray-700">
            <IPInput
              label="节点 IP 地址"
              description="输入要同步的设备 IPv4 地址"
              value={peerDeviceAddr}
              onChange={handlePeerDeviceAddrChange}
              errorMessage={peerIpError}
            />
          </div>

          {/* 节点端口 */}
          <div className="settings-item py-2 rounded-lg px-2 ml-4 border-l-2 border-gray-700">
            <Input
              label="节点端口"
              description="输入要同步的设备端口 (1024-65535)"
              value={peerDevicePort.toString()}
              onChange={handlePeerDevicePortChange}
              type="text"
              errorMessage={peerPortError}
            />
          </div>
        </>
      )}

      {/* 云服务器配置 */}
      <div className="settings-item py-2 rounded-lg px-2 opacity-60 cursor-not-allowed">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center">
            <h4 className="text-sm font-medium text-white">云服务器配置</h4>
            <span className="ml-2 px-1.5 py-0.5 bg-gray-700 text-xs text-gray-400 rounded">
              即将推出
            </span>
          </div>
          <button
            className="px-2 py-1 bg-gray-700 text-xs text-gray-300 rounded pointer-events-none"
            disabled
          >
            高级选项
          </button>
        </div>
        <div className="flex">
          <div className="px-2 py-1 bg-gray-700 rounded-lg text-sm text-gray-300 flex-1">
            使用默认云服务器 ({cloudServer})
          </div>
        </div>
      </div>
    </div>
  );
};

export default NetworkSection;
