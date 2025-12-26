import React, { useState } from "react";
import { Smartphone, Monitor, Tablet, Settings, Eye, Trash2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import DeviceSettingsPanel from "./DeviceSettingsPanel";

const OtherDevice: React.FC = () => {
  const [expandedDevices, setExpandedDevices] = useState<Record<string, boolean>>({});

  const toggleDevice = (id: string) => {
    setExpandedDevices(prev => ({
      ...prev,
      [id]: !prev[id]
    }));
  };

  const devices = [
    {
      id: "iphone",
      name: "iPhone 13",
      type: "mobile",
      icon: Smartphone,
      status: "online",
      lastActive: "10分钟前",
      label: "移动设备",
      color: "blue"
    },
    {
      id: "workstation",
      name: "工作站",
      type: "desktop",
      icon: Monitor,
      status: "idle",
      lastActive: "1小时前",
      label: "Windows",
      color: "purple"
    },
    {
      id: "ipad",
      name: "iPad Pro",
      type: "tablet",
      icon: Tablet,
      status: "offline",
      lastActive: "2天前",
      label: "平板设备",
      color: "green"
    }
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case "online": return "text-green-500 bg-green-500/10 border-green-500/20";
      case "idle": return "text-yellow-500 bg-yellow-500/10 border-yellow-500/20";
      case "offline": return "text-muted-foreground bg-muted border-border";
      default: return "text-muted-foreground";
    }
  };

  const getIconColor = (color: string) => {
    switch (color) {
      case "blue": return "text-blue-500 bg-blue-500/10 border-blue-500/20";
      case "purple": return "text-purple-500 bg-purple-500/10 border-purple-500/20";
      case "green": return "text-green-500 bg-green-500/10 border-green-500/20";
      default: return "text-primary bg-primary/10 border-primary/20";
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-4 mb-4 mt-8">
        <h3 className="text-sm font-medium text-muted-foreground whitespace-nowrap">其他已连接设备</h3>
        <div className="h-px flex-1 bg-border/50"></div>
      </div>

      {devices.map((device) => {
        const Icon = device.icon;
        const isExpanded = expandedDevices[device.id] || false;
        
        return (
          <div key={device.id} className="group relative overflow-hidden bg-card/50 hover:bg-card/80 border border-border/50 hover:border-primary/20 rounded-lg transition-all duration-300 shadow-sm hover:shadow-md">
             <div className="relative z-10 p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-5">
                  {/* Icon Box */}
                  <div className={`h-14 w-14 rounded-md flex items-center justify-center ring-1 shadow-inner ${getIconColor(device.color)}`}>
                    <Icon className="h-7 w-7" />
                  </div>

                  {/* Info */}
                  <div>
                    <div className="flex items-center gap-3">
                      <h4 className="text-lg font-semibold text-foreground tracking-tight">{device.name}</h4>
                      <span className={`px-2.5 py-0.5 text-xs font-medium rounded-full border ${getIconColor(device.color)}`}>
                        {device.label}
                      </span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-1">最后活动时间: {device.lastActive}</p>
                  </div>
                </div>

                {/* Actions & Status */}
                <div className="flex items-center gap-6">
                  {/* Status Indicator */}
                  <div className={`flex items-center gap-2 px-3 py-1.5 rounded-full border ${getStatusColor(device.status)}`}>
                    <span className="relative flex h-2 w-2">
                      {device.status === 'online' && (
                         <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-500 opacity-75"></span>
                      )}
                      <span className={`relative inline-flex rounded-full h-2 w-2 ${
                          device.status === 'online' ? 'bg-green-500' : 
                          device.status === 'idle' ? 'bg-yellow-500' : 'bg-gray-400'
                      }`}></span>
                    </span>
                    <span className="text-xs font-medium">
                      {device.status === 'online' ? '在线' : device.status === 'idle' ? '空闲' : '离线'}
                    </span>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex items-center gap-2">
                       <button 
                        onClick={() => toggleDevice(device.id)}
                        className={`p-2 rounded-xl transition-all duration-300 ${isExpanded ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25" : "text-muted-foreground hover:text-foreground hover:bg-muted"}`}
                        title="设置"
                      >
                      <Settings className={`h-5 w-5 transition-transform duration-500 ${isExpanded ? "rotate-90" : ""}`} />
                      </button>
                      <button className="p-2 text-muted-foreground hover:text-foreground hover:bg-muted rounded-xl transition-colors" title="查看">
                      <Eye className="h-5 w-5" />
                      </button>
                      <button className="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-xl transition-colors" title="删除">
                      <Trash2 className="h-5 w-5" />
                      </button>
                  </div>
                </div>
              </div>

               {/* Expandable Settings Panel */}
               <AnimatePresence>
                  {isExpanded && (
                      <motion.div
                          initial={{ height: 0, opacity: 0 }}
                          animate={{ height: "auto", opacity: 1 }}
                          exit={{ height: 0, opacity: 0 }}
                          transition={{ duration: 0.3, ease: "easeInOut" }}
                          className="overflow-hidden"
                      >
                           <div className="pt-6 border-t border-border/50 mt-6">
                              <DeviceSettingsPanel deviceId={device.id} deviceName={device.name} />
                           </div>
                      </motion.div>
                  )}
              </AnimatePresence>
            </div>
          </div>
        );
      })}
    </div>
  );
};

export default OtherDevice;
