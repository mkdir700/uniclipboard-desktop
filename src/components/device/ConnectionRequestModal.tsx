/**
 * 连接请求确认弹窗
 *
 * 当收到其他设备的连接请求时，显示此弹窗让用户确认
 */

import React, { useState, useEffect } from "react";
import {
  Smartphone,
  Network,
  Loader2,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  respondToConnectionRequest,
  type ConnectionRequestInfo,
  type ConnectionRequestDecision,
} from "@/api/deviceConnection";

interface ConnectionRequestModalProps {
  open: boolean;
  onClose: () => void;
  request: ConnectionRequestInfo | null;
}

type RequestStatus = "idle" | "processing" | "accepted" | "rejected" | "error";

const ConnectionRequestModal: React.FC<ConnectionRequestModalProps> = ({
  open,
  onClose,
  request,
}) => {
  const [status, setStatus] = useState<RequestStatus>("idle");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [timeLeft, setTimeLeft] = useState<number>(30);

  // 倒计时自动拒绝
  useEffect(() => {
    if (!open || !request || status !== "idle") return;

    setTimeLeft(30);
    const timer = setInterval(() => {
      setTimeLeft((prev) => {
        if (prev <= 1) {
          clearInterval(timer);
          // 自动拒绝
          handleResponse(false);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [open, request]);

  // 重置状态
  useEffect(() => {
    if (open && request) {
      setStatus("idle");
      setErrorMessage("");
      setTimeLeft(30);
    }
  }, [open, request]);

  const handleResponse = async (accept: boolean) => {
    if (!request) return;

    setStatus("processing");

    try {
      const decision: ConnectionRequestDecision = {
        accept,
        requester_device_id: request.requester_device_id,
      };

      const response = await respondToConnectionRequest(decision);

      if (response.success) {
        setStatus(accept ? "accepted" : "rejected");

        // 延迟后关闭弹窗
        setTimeout(() => {
          onClose();
        }, 1500);
      } else {
        setStatus("error");
        setErrorMessage(response.message);
      }
    } catch (error: any) {
      setStatus("error");
      setErrorMessage(error.toString() || "处理失败，请重试");
    }
  };

  if (!request) return null;

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>设备连接请求</DialogTitle>
          <DialogDescription asChild>
            <div>
              {status === "idle" && (
                <p className="text-muted-foreground">
                  设备 {request.requester_alias || request.requester_device_id} 请求连接到此设备
                </p>
              )}
              {status === "processing" && (
                <p className="text-muted-foreground">正在处理...</p>
              )}
              {status === "accepted" && (
                <p className="text-muted-foreground text-green-600">
                  已接受连接请求
                </p>
              )}
              {status === "rejected" && (
                <p className="text-muted-foreground text-red-600">
                  已拒绝连接请求
                </p>
              )}
              {status === "error" && (
                <p className="text-muted-foreground text-red-600">
                  {errorMessage}
                </p>
              )}
            </div>
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          {status === "idle" && (
            <div className="space-y-4">
              {/* 设备信息卡片 */}
              <div className="bg-muted rounded-2xl p-5 border border-border/50">
                <div className="flex items-center gap-4">
                  <div className="p-3 bg-primary/10 rounded-xl">
                    <Smartphone className="w-6 h-6 text-primary" />
                  </div>
                  <div className="flex-1">
                    <div className="font-semibold text-lg">
                      {request.requester_alias || `设备 ${request.requester_device_id}`}
                    </div>
                    <div className="text-sm text-muted-foreground">
                      ID: {request.requester_device_id}
                    </div>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-border/50">
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Network className="w-4 h-4" />
                    <span>IP: {request.requester_ip}</span>
                  </div>
                  {request.requester_platform && (
                    <div className="text-xs text-muted-foreground mt-1">
                      平台: {request.requester_platform}
                    </div>
                  )}
                </div>
              </div>

              {/* 倒计时提示 */}
              <div className="text-center text-sm text-muted-foreground">
                <span className={timeLeft <= 10 ? "text-red-500" : ""}>
                  {timeLeft}
                </span>{" "}
                秒后自动拒绝
              </div>

              {/* 操作按钮 */}
              <div className="flex gap-3">
                <Button
                  onClick={() => handleResponse(false)}
                  variant="outline"
                  className="flex-1"
                  disabled={status !== "idle"}
                >
                  拒绝
                </Button>
                <Button
                  onClick={() => handleResponse(true)}
                  className="flex-1"
                  disabled={status !== "idle"}
                >
                  接受
                </Button>
              </div>
            </div>
          )}

          {status === "processing" && (
            <div className="flex flex-col items-center py-6">
              <Loader2 className="w-12 h-12 animate-spin text-primary mb-4" />
              <p className="text-sm text-muted-foreground">正在处理连接请求...</p>
            </div>
          )}

          {status === "accepted" && (
            <div className="flex flex-col items-center py-6">
              <CheckCircle2 className="w-12 h-12 text-green-500 mb-4" />
              <p className="text-sm font-medium text-green-600">连接已建立</p>
            </div>
          )}

          {status === "rejected" && (
            <div className="flex flex-col items-center py-6">
              <XCircle className="w-12 h-12 text-red-500 mb-4" />
              <p className="text-sm font-medium text-red-600">已拒绝连接</p>
            </div>
          )}

          {status === "error" && (
            <div className="space-y-4">
              <div className="flex items-start gap-3 p-3 bg-destructive/10 rounded-lg">
                <XCircle className="w-5 h-5 text-destructive flex-shrink-0 mt-0.5" />
                <p className="text-sm text-destructive">{errorMessage}</p>
              </div>
              <div className="flex gap-3">
                <Button
                  onClick={onClose}
                  variant="outline"
                  className="flex-1"
                >
                  关闭
                </Button>
                <Button
                  onClick={() => handleResponse(true)}
                  className="flex-1"
                >
                  重试
                </Button>
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ConnectionRequestModal;
