import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSetting } from "@/contexts/SettingContext";
import { Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Switch } from "@/components/ui";

const NetworkSection: React.FC = () => {
  const { t } = useTranslation();
  const { setting, error, updateNetworkSetting } = useSetting();

  // Local state
  const [syncMethod, setSyncMethod] = useState("lan_first");
  const [cloudServer, setCloudServer] = useState("api.clipsync.com");
  const [webserverPort, setWebserverPort] = useState(29217);
  const [portError, setPortError] = useState<string | null>(null);

  // Custom peer device state
  const [customPeerDevice, setCustomPeerDevice] = useState(false);
  const [peerDeviceAddr, setPeerDeviceAddr] = useState("192.168.1.100");
  const [peerDevicePort, setPeerDevicePort] = useState(29217);
  const [peerIpError, setPeerIpError] = useState<string | null>(null);
  const [peerPortError, setPeerPortError] = useState<string | null>(null);

  // Sync method options
  const syncMethodOptions = [
    { value: "lan_first", label: t("settings.sections.network.syncMethod.lanFirst") },
    { value: "cloud_only", label: t("settings.sections.network.syncMethod.cloudOnly") },
    { value: "lan_only", label: t("settings.sections.network.syncMethod.lanOnly") },
  ];

  // Update local state when settings are loaded
  useEffect(() => {
    if (setting) {
      setSyncMethod(setting.network.sync_method);
      setCloudServer(setting.network.cloud_server);
      setWebserverPort(setting.network.webserver_port);

      // Load custom peer device settings
      setCustomPeerDevice(setting.network.custom_peer_device || false);
      setPeerDeviceAddr(setting.network.peer_device_addr || "192.168.1.100");
      setPeerDevicePort(setting.network.peer_device_port || 29217);
    }
  }, [setting]);

  // Handle sync method change
  const handleSyncMethodChange = (value: string) => {
    setSyncMethod(value);
    updateNetworkSetting({ sync_method: value });
  };

  // Handle webserver port change
  const handleWebserverPortChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    // If input is empty, allow user to continue typing
    if (!value.trim()) {
      setPortError(null);
      setWebserverPort(0); // Temporarily set to 0, but don't update to settings
      return;
    }

    // Check if it's a number
    if (!/^\d+$/.test(value)) {
      setPortError(t("settings.sections.network.webserverPort.errors.invalid"));
      setWebserverPort(parseInt(value) || 0); // Update display value even with error
      return;
    }

    const port = parseInt(value);
    setWebserverPort(port); // Always update display value

    // Validate port range
    if (port < 1024 || port > 65535) {
      setPortError(t("settings.sections.network.webserverPort.errors.range"));
      return;
    }

    // Validation passed
    setPortError(null);
    updateNetworkSetting({ webserver_port: port });
  };

  // Handle custom peer device toggle change
  const handleCustomPeerDeviceChange = (checked: boolean) => {
    const newValue = checked;
    setCustomPeerDevice(newValue);
    updateNetworkSetting({ custom_peer_device: newValue });

    // If turned off, clear error states
    if (!newValue) {
      setPeerIpError(null);
      setPeerPortError(null);
    }
  };

  // Validate IPv4 address
  const validateIPv4 = (ip: string): boolean => {
    const ipv4Regex =
      /^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
    return ipv4Regex.test(ip);
  };

  // Handle peer device IP change
  const handlePeerDeviceAddrChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setPeerDeviceAddr(value);

    // Validate IP address
    if (!value.trim()) {
      setPeerIpError(t("settings.sections.network.customPeerDevice.peerIp.errors.empty"));
      return;
    }

    if (!validateIPv4(value)) {
      setPeerIpError(t("settings.sections.network.customPeerDevice.peerIp.errors.invalid"));
      return;
    }

    // Validation passed
    setPeerIpError(null);
    updateNetworkSetting({ peer_device_addr: value });
  };

  // Handle peer device port change
  const handlePeerDevicePortChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;

    // If input is empty, allow user to continue typing
    if (!value.trim()) {
      setPeerPortError(t("settings.sections.network.customPeerDevice.peerPort.errors.empty"));
      setPeerDevicePort(0); // Temporarily set to 0, but don't update to settings
      return;
    }

    // Check if it's a number
    if (!/^\d+$/.test(value)) {
      setPeerPortError(t("settings.sections.network.customPeerDevice.peerPort.errors.invalid"));
      setPeerDevicePort(parseInt(value) || 0); // Update display value even with error
      return;
    }

    const port = parseInt(value);
    setPeerDevicePort(port); // Always update display value

    // Validate port range
    if (port < 1024 || port > 65535) {
      setPeerPortError(t("settings.sections.network.customPeerDevice.peerPort.errors.range"));
      return;
    }

    // Validation passed
    setPeerPortError(null);
    updateNetworkSetting({ peer_device_port: port });
  };

  // If there is an error, display error message
  if (error) {
    return <div className="text-red-500 py-4">{t("settings.sections.network.loadError")}: {error}</div>;
  }

  return (
    <>
      {/* Sync Method */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.network.syncMethod.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.network.syncMethod.description")}
            </p>
          </div>
          <Select
            value={syncMethod}
            onValueChange={handleSyncMethodChange}
          >
            <SelectTrigger className="w-64">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {syncMethodOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Webserver Port */}
      <div className="py-2 rounded-lg px-2">
        <div className="space-y-2">
          <h4 className="text-base font-medium">{t("settings.sections.network.webserverPort.label")}</h4>
          <p className="text-sm text-muted-foreground">
            {t("settings.sections.network.webserverPort.description")}
          </p>
          <Input
            type="text"
            value={webserverPort.toString()}
            onChange={handleWebserverPortChange}
            className={portError ? "border-red-500" : ""}
          />
          {portError && (
            <p className="text-xs text-red-500">{portError}</p>
          )}
        </div>
      </div>

      {/* Custom Peer Device Toggle */}
      <div className="py-2 rounded-lg px-2">
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <h4 className="text-base font-medium">{t("settings.sections.network.customPeerDevice.label")}</h4>
            <p className="text-sm text-muted-foreground">
              {t("settings.sections.network.customPeerDevice.description")}
            </p>
          </div>
          <Switch
            checked={customPeerDevice}
            onCheckedChange={handleCustomPeerDeviceChange}
          />
        </div>
      </div>

      {/* Peer IP and Port (only shown when custom peer device is enabled) */}
      {customPeerDevice && (
        <>
          {/* Peer IP */}
          <div className="py-2 rounded-lg px-2 ml-4 border-l-2 border-muted pl-4">
            <div className="space-y-2">
              <h4 className="text-base font-medium">{t("settings.sections.network.customPeerDevice.peerIp.label")}</h4>
              <p className="text-sm text-muted-foreground">
                {t("settings.sections.network.customPeerDevice.peerIp.description")}
              </p>
              <Input
                type="text"
                value={peerDeviceAddr}
                onChange={handlePeerDeviceAddrChange}
                className={peerIpError ? "border-red-500" : ""}
              />
              {peerIpError && (
                <p className="text-xs text-red-500">{peerIpError}</p>
              )}
            </div>
          </div>

          {/* Peer Port */}
          <div className="py-2 rounded-lg px-2 ml-4 border-l-2 border-muted pl-4">
            <div className="space-y-2">
              <h4 className="text-base font-medium">{t("settings.sections.network.customPeerDevice.peerPort.label")}</h4>
              <p className="text-sm text-muted-foreground">
                {t("settings.sections.network.customPeerDevice.peerPort.description")}
              </p>
              <Input
                type="text"
                value={peerDevicePort.toString()}
                onChange={handlePeerDevicePortChange}
                className={peerPortError ? "border-red-500" : ""}
              />
              {peerPortError && (
                <p className="text-xs text-red-500">{peerPortError}</p>
              )}
            </div>
          </div>
        </>
      )}

      {/* Cloud Server Configuration */}
      <div className="py-2 rounded-lg px-2 opacity-60 cursor-not-allowed">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center space-x-2">
            <h4 className="text-base font-medium">{t("settings.sections.network.cloudServer.label")}</h4>
            <span className="px-1.5 py-0.5 bg-muted text-xs text-muted-foreground rounded">
              {t("settings.sections.network.cloudServer.badge")}
            </span>
          </div>
          <button
            className="px-2 py-1 bg-muted text-xs text-muted-foreground rounded pointer-events-none"
            disabled
          >
            {t("settings.sections.network.cloudServer.advanced")}
          </button>
        </div>
        <div className="flex">
          <div className="px-2 py-1 bg-muted rounded-lg text-sm text-muted-foreground flex-1">
            {t("settings.sections.network.cloudServer.default")} ({cloudServer})
          </div>
        </div>
      </div>
    </>
  );
};

export default NetworkSection;
