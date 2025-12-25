import React, { useState, useRef, useEffect } from "react";
import { DeviceList, DeviceHeader } from "@/components";
import { DeviceTab } from "@/components/device/Header";
import { PairingModal, ConnectionRequestModal } from "@/components/device";
import {
  onConnectionRequest,
  type ConnectionRequestInfo,
} from "@/api/deviceConnection";

const DevicesPage: React.FC = () => {
  const [showPairingModal, setShowPairingModal] = useState(false);
  const [activeTab, setActiveTab] = useState<DeviceTab>("connected");

  // 连接请求相关状态
  const [pendingRequest, setPendingRequest] = useState<ConnectionRequestInfo | null>(null);
  const unlistenRefRef = useRef<(() => void) | null>(null);

  // Refs for scrolling
  const connectedRef = useRef<HTMLDivElement>(null);
  const requestsRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 设置连接请求监听
    setupConnectionRequestListener();

    return () => {
      // 清理事件监听
      if (unlistenRefRef.current) {
        unlistenRefRef.current();
      }
    };
  }, []);

  const setupConnectionRequestListener = async () => {
    try {
      const unlisten = await onConnectionRequest((request) => {
        console.log("Received connection request:", request);
        setPendingRequest(request);
      });
      unlistenRefRef.current = unlisten;
    } catch (error) {
      console.error("Failed to setup connection request listener:", error);
    }
  };

  const handleAddDevice = () => {
    setShowPairingModal(true);
  };

  const handleClosePairingModal = () => {
    setShowPairingModal(false);
  };

  const handleConnectSuccess = () => {
    // 连接成功后刷新设备列表
    // TODO: 可以添加刷新设备列表的逻辑
  };

  const handleCloseConnectionRequest = () => {
    setPendingRequest(null);
  };

  const handleTabChange = (tab: DeviceTab) => {
    setActiveTab(tab);
    let targetRef;
    switch (tab) {
      case "connected":
        targetRef = connectedRef;
        break;
      case "requests":
        targetRef = requestsRef;
        break;
    }

    if (targetRef?.current) {
      targetRef.current.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  };

  // Optional: Update active tab on scroll
  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) return;

    const handleScroll = () => {
      const positions = [
        { id: "connected" as DeviceTab, ref: connectedRef },
        { id: "requests" as DeviceTab, ref: requestsRef },
      ];

      // Simple proximity check
      for (const { id, ref } of positions) {
        if (ref.current) {
          const rect = ref.current.getBoundingClientRect();
          // If the element is near the top of the viewport (with some offset for header)
          if (rect.top >= 0 && rect.top < 300) {
             setActiveTab(id);
             break;
          }
        }
      }
    };

    container.addEventListener("scroll", handleScroll);
    return () => container.removeEventListener("scroll", handleScroll);
  }, []);


  return (

      <div className="flex flex-col h-full relative">
        {/* 顶部标题栏 */}
        <DeviceHeader
          addDevice={handleAddDevice}
          activeTab={activeTab}
          onTabChange={handleTabChange}
        />

        {/* 内容区域 */}
        <div className="flex-1 overflow-hidden relative">
          <div
            ref={scrollContainerRef}
            className="h-full overflow-y-auto scrollbar-thin px-8 pb-32 pt-2 scroll-smooth"
          >
            {/* Pairing Requests Section */}
            <div id="requests" ref={requestsRef} className="scroll-mt-24 mb-12">
               <div className="flex items-center gap-4 mb-4 mt-8">
                <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">配对请求</h3>
                <div className="h-[1px] flex-1 bg-border/50"></div>
              </div>
              <div className="flex flex-col items-center justify-center p-8 border border-dashed border-border/50 rounded-2xl bg-muted/5 text-muted-foreground">
                <p>暂无配对请求</p>
              </div>
            </div>

            {/* Connected Devices Section */}
            <div id="connected" ref={connectedRef} className="scroll-mt-24 mb-12">
              <DeviceList />
            </div>
          </div>
        </div>

        {/* 设备配对模态框 */}
        <PairingModal
          open={showPairingModal}
          onClose={handleClosePairingModal}
          onConnectSuccess={handleConnectSuccess}
        />

        {/* 连接请求确认模态框 */}
        <ConnectionRequestModal
          open={pendingRequest !== null}
          onClose={handleCloseConnectionRequest}
          request={pendingRequest}
        />
      </div>

    );
};

export default DevicesPage;
