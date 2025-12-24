import React, { useState } from "react";
import { cn } from "@/lib/utils";
import { motion, AnimatePresence } from "framer-motion";

interface DeviceSettingsPanelProps {
  deviceId: string;
  deviceName: string;
}

type SettingsTab = "rules" | "permissions";

const DeviceSettingsPanel: React.FC<DeviceSettingsPanelProps> = ({ deviceName }) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>("rules");

  return (
    <div className="pt-2 pb-6 px-1">
        <div className="bg-card/30 rounded-xl border border-border/50 overflow-hidden">
            {/* Tab Navigation */}
            <div className="flex border-b border-border/50">
                <button
                    onClick={() => setActiveTab("rules")}
                    className={cn(
                        "flex-1 py-3 text-sm font-medium transition-colors relative",
                        activeTab === "rules" ? "text-primary" : "text-muted-foreground hover:text-foreground hover:bg-muted/30"
                    )}
                >
                    同步规则
                    {activeTab === "rules" && (
                        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
                    )}
                </button>
                <button
                    onClick={() => setActiveTab("permissions")}
                    className={cn(
                        "flex-1 py-3 text-sm font-medium transition-colors relative",
                        activeTab === "permissions" ? "text-primary" : "text-muted-foreground hover:text-foreground hover:bg-muted/30"
                    )}
                >
                    访问权限
                     {activeTab === "permissions" && (
                        <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
                    )}
                </button>
            </div>

            {/* Content Area */}
            <div className="p-6">
                 <AnimatePresence mode="wait">
                    {activeTab === "rules" ? (
                        <motion.div
                            key="rules"
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -10 }}
                            transition={{ duration: 0.2 }}
                            className="space-y-4"
                        >
                            <div className="flex items-center justify-between mb-2">
                                <h4 className="text-sm font-medium text-foreground">{deviceName} 同步设置</h4>
                                <button className="text-xs px-3 py-1 border border-border/50 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
                                    恢复默认
                                </button>
                            </div>

                             {/* Sync Rule Items */}
                             {[
                                { title: "自动同步", desc: "在设备解锁状态下自动同步剪贴板内容", defaultChecked: true },
                                { title: "同步文本", desc: "允许同步文本内容", defaultChecked: true },
                                { title: "同步图片", desc: "允许同步图片内容 (可能会消耗更多流量)", defaultChecked: true },
                                { title: "同步文件", desc: "允许同步文件内容 (最大10MB)", defaultChecked: false },
                             ].map((rule, idx) => (
                                 <div key={idx} className="flex items-center justify-between bg-background/50 rounded-lg p-3 border border-border/30">
                                     <div>
                                         <h5 className="text-sm font-medium text-foreground">{rule.title}</h5>
                                         <p className="text-xs text-muted-foreground mt-0.5">{rule.desc}</p>
                                     </div>
                                     <label className="flex items-center cursor-pointer">
                                         <div className="relative">
                                             <input type="checkbox" className="sr-only peer" defaultChecked={rule.defaultChecked} />
                                             <div className="block bg-muted w-10 h-5 rounded-full peer-checked:bg-primary transition-colors"></div>
                                             <div className="absolute left-1 top-1 bg-white w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-5"></div>
                                         </div>
                                     </label>
                                 </div>
                             ))}
                        </motion.div>
                    ) : (
                        <motion.div
                             key="permissions"
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: -10 }}
                            transition={{ duration: 0.2 }}
                            className="space-y-4"
                        >
                            <div className="mb-4">
                                <h4 className="font-medium text-foreground mb-1">{deviceName} 访问权限</h4>
                                <p className="text-sm text-muted-foreground">
                                    控制该设备可以访问的内容类型。
                                </p>
                            </div>

                            <div className="space-y-3">
                                {[
                                    { label: "读取剪贴板", checked: true },
                                    { label: "写入剪贴板", checked: true },
                                    { label: "访问历史记录", checked: true },
                                    { label: "传输文件", checked: false },
                                ].map((perm, idx) => (
                                    <div key={idx} className="flex items-center justify-between py-2 border-b border-border/30 last:border-0">
                                        <span className="text-sm text-foreground">{perm.label}</span>
                                        <label className="flex items-center cursor-pointer">
                                            <div className="relative">
                                                <input type="checkbox" className="sr-only peer" defaultChecked={perm.checked} />
                                                <div className="block bg-muted w-10 h-5 rounded-full peer-checked:bg-primary transition-colors"></div>
                                                <div className="absolute left-1 top-1 bg-white w-3 h-3 rounded-full transition-transform transform peer-checked:translate-x-5"></div>
                                            </div>
                                        </label>
                                    </div>
                                ))}
                            </div>
                        </motion.div>
                    )}
                 </AnimatePresence>
            </div>
        </div>
    </div>
  );
};

export default DeviceSettingsPanel;
