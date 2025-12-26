/**
 * 设备配对弹窗
 *
 * 支持两种连接方式：
 * 1. 手动连接 - 输入 IP 和端口连接
 * 2. 自动发现 - 自动发现局域网内的设备（开发中）
 */

import React, { useState, useEffect, useRef } from "react";
import {
  Wifi,
  Loader2,
  AlertCircle,
  CheckCircle2,
  Network,
  ChevronDown,
  ChevronRight,
  X,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  connectToDeviceManual,
  getLocalNetworkInterfaces,
  type ManualConnectionRequest,
  type ManualConnectionResponse,
  type ConnectionState,
  type NetworkInterface,
} from "@/api/deviceConnection";

interface PairingModalProps {
  open: boolean;
  onClose: () => void;
  onConnectSuccess?: () => void;
}

type ConnectionTab = "manual" | "auto";

const PairingModal: React.FC<PairingModalProps> = ({
  open,
  onClose,
  onConnectSuccess,
}) => {
  const [activeTab, setActiveTab] = useState<ConnectionTab>("manual");
  // 选中的网卡索引
  const [selectedInterfaceIndex, setSelectedInterfaceIndex] = useState<number>(
    -1
  );
  // IP 地址四段输入
  const [ipSegments, setIpSegments] = useState<string[]>(["", "", "", ""]);
  const [targetPort, setTargetPort] = useState<string>("29217");
  const [connectionState, setConnectionState] = useState<ConnectionState>({
    status: "idle",
  });
  // 本机网卡信息
  const [localInterfaces, setLocalInterfaces] = useState<NetworkInterface[]>([]);
  // 端口设置展开状态
  const [showPortSettings, setShowPortSettings] = useState(false);
  // 异常消息可见状态（用于动画）
  const [showErrorMessage, setShowErrorMessage] = useState(false);

  const ipInputRefs = [
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
    useRef<HTMLInputElement>(null),
  ];

  // 重置状态
  useEffect(() => {
    if (open) {
      setActiveTab("manual");
      setSelectedInterfaceIndex(-1);
      setIpSegments(["", "", "", ""]);
      setTargetPort("29217");
      setConnectionState({ status: "idle" });
      setShowErrorMessage(false);
      // 加载本机网卡信息
      loadLocalInterfaces();
    }
  }, [open]);

  // 监听连接状态变化，控制异常消息显示
  useEffect(() => {
    if (connectionState.status === "failed") {
      setShowErrorMessage(true);
    } else if (connectionState.status === "connecting") {
      setShowErrorMessage(false);
    }
  }, [connectionState.status]);

  // 加载本机网卡信息
  const loadLocalInterfaces = async () => {
    try {
      const interfaces = await getLocalNetworkInterfaces();
      setLocalInterfaces(interfaces);
      // 默认选中第一个网卡
      if (interfaces.length > 0) {
        setSelectedInterfaceIndex(0);
        fillNetworkPrefix(interfaces[0].ip);
      }
    } catch (error) {
      console.error("Failed to load local network interfaces:", error);
    }
  };

  // 获取 IP 的网段前缀（前三个数字）
  const getNetworkPrefix = (ip: string): string[] => {
    const parts = ip.split(".");
    if (parts.length >= 3) {
      return [parts[0], parts[1], parts[2]];
    }
    return ["", "", ""];
  };

  // 填充网段前缀到 IP 输入框
  const fillNetworkPrefix = (ip: string) => {
    const prefix = getNetworkPrefix(ip);
    setIpSegments([...prefix, ""]);
    // 聚焦到最后一个输入框
    setTimeout(() => {
      ipInputRefs[3].current?.focus();
    }, 0);
  };

  // 处理网卡选择
  const handleInterfaceSelect = (index: number) => {
    setSelectedInterfaceIndex(index);
    const iface = localInterfaces[index];
    fillNetworkPrefix(iface.ip);
  };

  // 构建 IP 地址字符串
  const buildIpString = (): string => {
    return ipSegments.join(".");
  };

  // 处理 IP 段输入变化
  const handleIpSegmentChange = (index: number, value: string) => {
    // 只允许数字
    const numericValue = value.replace(/\D/g, "");

    const newSegments = [...ipSegments];
    newSegments[index] = numericValue;
    setIpSegments(newSegments);

    // 自动跳到下一个输入框
    if (numericValue.length >= 3 && index < 3) {
      ipInputRefs[index + 1].current?.focus();
    }
  };

  // 处理 IP 段键盘事件
  const handleIpSegmentKeyDown = (
    index: number,
    e: React.KeyboardEvent<HTMLInputElement>
  ) => {
    // 按小数点或空格跳到下一个输入框
    if (e.key === "." || e.key === " ") {
      e.preventDefault();
      if (index < 3) {
        ipInputRefs[index + 1].current?.focus();
      }
    }
    // 按 Backspace 且当前输入框为空时，跳到上一个输入框
    if (e.key === "Backspace" && ipSegments[index] === "" && index > 0) {
      e.preventDefault();
      ipInputRefs[index - 1].current?.focus();
    }
    // 按 Enter 触发连接
    if (e.key === "Enter") {
      handleConnect();
    }
  };

  // 处理 IP 段粘贴事件
  const handleIpSegmentPaste = (e: React.ClipboardEvent) => {
    e.preventDefault();
    const pastedText = e.clipboardData.getData("text");
    // 提取所有数字
    const numbers = pastedText.match(/\d+/g);
    if (numbers) {
      const allNumbers = numbers.join("").split("");
      const newSegments = ["", "", "", ""];
      for (let i = 0; i < 4; i++) {
        newSegments[i] = allNumbers.slice(i * 3, (i + 1) * 3).join("") || "";
      }
      setIpSegments(newSegments);
    }
  };

  const validateIpSegments = (): boolean => {
    return ipSegments.every((segment) => {
      if (segment === "") return false;
      const num = parseInt(segment, 10);
      return num >= 0 && num <= 255;
    });
  };

  const validatePort = (port: string): boolean => {
    const portNum = parseInt(port, 10);
    return !isNaN(portNum) && portNum >= 1024 && portNum <= 65535;
  };

  const handleConnect = async () => {
    // 验证输入
    if (!validateIpSegments()) {
      setConnectionState({
        status: "failed",
        message: "请输入有效的 IP 地址",
        canRetry: true,
      });
      return;
    }

    if (!validatePort(targetPort)) {
      setConnectionState({
        status: "failed",
        message: "请输入有效的端口号 (1024-65535)",
        canRetry: true,
      });
      return;
    }

    // 开始连接
    setConnectionState({
      status: "connecting",
      message: "正在连接设备...",
    });

    try {
      const request: ManualConnectionRequest = {
        ip: buildIpString(),
        port: parseInt(targetPort, 10),
      };

      const response: ManualConnectionResponse = await connectToDeviceManual(
        request
      );

      if (response.success) {
        setConnectionState({
          status: "connected",
          message: "连接成功！",
          device_id: response.device_id,
        });

        // 延迟后关闭弹窗
        setTimeout(() => {
          onConnectSuccess?.();
          onClose();
        }, 1500);
      } else {
        setConnectionState({
          status: "failed",
          message: response.message,
          canRetry: true,
        });
      }
    } catch (error: any) {
      setConnectionState({
        status: "failed",
        message: error.toString() || "连接失败，请重试",
        canRetry: true,
      });
    }
  };

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>添加新设备</DialogTitle>
          <DialogDescription>
            选择一种方式来连接新设备
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          <Tabs
            value={activeTab}
            onValueChange={(value) => setActiveTab(value as ConnectionTab)}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="manual">手动连接</TabsTrigger>
              <TabsTrigger value="auto" disabled className="relative">
                自动发现
                <span className="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-muted text-[10px] text-muted-foreground">
                  即将
                </span>
              </TabsTrigger>
            </TabsList>

            <TabsContent value="manual" className="mt-4">
              {/* 异常消息横幅 - 带动画 */}
              {showErrorMessage && connectionState.status === "failed" && (
                <div
                  className={`mb-4 overflow-hidden transition-all duration-300 ease-in-out ${
                    showErrorMessage
                      ? "max-h-24 opacity-100"
                      : "max-h-0 opacity-0"
                  }`}
                >
                  <div className="flex items-start gap-3 p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
                    <AlertCircle className="w-5 h-5 text-destructive flex-shrink-0 mt-0.5" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-destructive font-medium">
                        {connectionState.message || "连接失败"}
                      </p>
                    </div>
                    <button
                      onClick={() => setShowErrorMessage(false)}
                      className="flex-shrink-0 text-destructive/70 hover:text-destructive transition-colors"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              )}

              {connectionState.status === "connected" && (
                <div className="flex flex-col items-center py-6">
                  <CheckCircle2 className="w-12 h-12 text-green-500 mb-4" />
                  <p className="text-sm font-medium text-green-600">
                    {connectionState.message || "连接成功"}
                  </p>
                </div>
              )}

              {connectionState.status !== "connected" && (
                <div className="space-y-4">
                  {/* 本机网卡选择 */}
                  {localInterfaces.length > 0 && (
                    <div>
                      <label className="block text-sm font-medium mb-2 flex items-center gap-1.5">
                        <Network className="w-4 h-4" />
                        本机网络
                      </label>
                      <Select
                        value={
                          selectedInterfaceIndex >= 0
                            ? String(selectedInterfaceIndex)
                            : undefined
                        }
                        onValueChange={(value) =>
                          handleInterfaceSelect(parseInt(value, 10))
                        }
                      >
                        <SelectTrigger>
                          <SelectValue placeholder="选择网络" />
                        </SelectTrigger>
                        <SelectContent>
                          {localInterfaces.map((iface, idx) => (
                            <SelectItem key={idx} value={String(idx)}>
                              <span className="text-muted-foreground">
                                {iface.name}
                              </span>
                              <span className="ml-2 font-mono">{iface.ip}</span>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  )}

                  {/* IP 输入 */}
                  <div>
                    <label className="block text-sm font-medium mb-1.5">
                      对向设备 IP 地址
                    </label>
                    <div className="flex items-center gap-2">
                      {ipSegments.map((segment, index) => (
                        <React.Fragment key={index}>
                          <input
                            ref={ipInputRefs[index]}
                            type="text"
                            inputMode="numeric"
                            value={segment}
                            onChange={(e) =>
                              handleIpSegmentChange(index, e.target.value)
                            }
                            onKeyDown={(e) => handleIpSegmentKeyDown(index, e)}
                            onPaste={handleIpSegmentPaste}
                            placeholder="000"
                            maxLength={3}
                            readOnly={index < 3 && selectedInterfaceIndex >= 0}
                            disabled={connectionState.status === "connecting"}
                            className={`w-full px-3 py-2 bg-background border rounded-lg text-center font-mono focus:outline-none focus:ring-2 ${
                              index < 3 && selectedInterfaceIndex >= 0
                                ? "border-border text-foreground bg-muted/50 cursor-default"
                                : "border-input text-foreground focus:ring-ring"
                            } ${
                              connectionState.status === "connecting"
                                ? "opacity-50 cursor-not-allowed"
                                : ""
                            }`}
                          />
                          {index < 3 && (
                            <span className="text-muted-foreground text-lg">
                              .
                            </span>
                          )}
                        </React.Fragment>
                      ))}
                    </div>
                    {selectedInterfaceIndex >= 0 && (
                      <p className="text-xs text-muted-foreground mt-1.5">
                        网段已根据所选网络自动填充，只需输入最后一部分
                      </p>
                    )}
                  </div>

                  {/* 端口输入（可折叠） */}
                  <div>
                    <button
                      onClick={() =>
                        connectionState.status !== "connecting" &&
                        setShowPortSettings(!showPortSettings)
                      }
                      className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors mb-2 disabled:opacity-50 disabled:cursor-not-allowed"
                      disabled={connectionState.status === "connecting"}
                    >
                      {showPortSettings ? (
                        <ChevronDown className="w-4 h-4" />
                      ) : (
                        <ChevronRight className="w-4 h-4" />
                      )}
                      <span>高级设置（端口号）</span>
                    </button>
                    {showPortSettings && (
                      <div>
                        <input
                          type="number"
                          value={targetPort}
                          onChange={(e) => setTargetPort(e.target.value)}
                          placeholder="29217"
                          disabled={connectionState.status === "connecting"}
                          className="w-full px-3 py-2 bg-background border border-input rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50 disabled:cursor-not-allowed"
                        />
                        <p className="text-xs text-muted-foreground mt-1.5">
                          默认端口为 29217，通常无需修改
                        </p>
                      </div>
                    )}
                  </div>

                  {/* 操作按钮 */}
                  <div className="flex gap-3 pt-2">
                    <Button
                      onClick={onClose}
                      variant="outline"
                      className="flex-1"
                      disabled={connectionState.status === "connecting"}
                    >
                      取消
                    </Button>
                    <Button
                      onClick={handleConnect}
                      className="flex-1"
                      disabled={
                        !ipSegments.every((s) => s !== "") ||
                        !targetPort ||
                        connectionState.status === "connecting"
                      }
                    >
                      {connectionState.status === "connecting" ? (
                        <>
                          <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                          连接中
                        </>
                      ) : (
                        "连接"
                      )}
                    </Button>
                  </div>
                </div>
              )}
            </TabsContent>

            <TabsContent value="auto" className="mt-4">
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <div className="w-16 h-16 bg-muted rounded-2xl flex items-center justify-center mb-4">
                  <Wifi className="w-8 h-8 text-muted-foreground" />
                </div>
                <h3 className="text-lg font-semibold mb-2">自动发现设备</h3>
                <p className="text-sm text-muted-foreground max-w-xs">
                  此功能正在开发中，将支持自动发现局域网内的设备并快速连接
                </p>
                <div className="mt-4 px-3 py-1.5 bg-muted rounded-full">
                  <span className="text-xs text-muted-foreground">即将推出</span>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default PairingModal;
