import React, { useState } from "react";
import { Smartphone } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface DevicePairingModalProps {
  onClose: () => void;
  open?: boolean;
}

const DevicePairingModal: React.FC<DevicePairingModalProps> = ({
  onClose,
  open = true,
}) => {
  const [step, setStep] = useState<number>(1);
  const [code, setCode] = useState<string>("");

  const handleGenerateCode = () => {
    // 生成6位随机数字代码
    const randomCode = Math.floor(100000 + Math.random() * 900000).toString();
    setCode(randomCode);
    setStep(2);
  };

  const handleFinish = () => {
    onClose();
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
          <DialogDescription asChild>
            <div>
              {step === 1 && (
                <p className="text-muted-foreground">
                  要添加新设备，请在新设备上安装 uniClipboard
                  应用并选择"添加现有账户"选项。然后点击下方按钮生成配对码。
                </p>
              )}
              {step === 2 && (
                <p className="text-muted-foreground">
                  请在新设备上输入以下配对码。此代码将在10分钟后过期。
                </p>
              )}
            </div>
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          {step === 1 && (
            <div className="space-y-6">
              <div className="flex items-center justify-center">
                <div className="h-32 w-32 bg-muted rounded-2xl flex items-center justify-center">
                  <Smartphone className="h-16 w-16 text-primary" />
                </div>
              </div>

              <Button
                onClick={handleGenerateCode}
                className="w-full"
                size="lg"
              >
                生成配对码
              </Button>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-6">
              <div className="flex items-center justify-center">
                <div className="bg-muted rounded-2xl p-6">
                  <div className="flex space-x-3">
                    {code.split("").map((digit, index) => (
                      <div
                        key={index}
                        className="w-12 h-14 bg-card rounded-xl flex items-center justify-center text-xl font-bold border border-border shadow-sm"
                      >
                        {digit}
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              <p className="text-muted-foreground text-sm text-center">
                配对码有效期：10:00
              </p>

              <div className="flex gap-3">
                <Button
                  onClick={() => setStep(1)}
                  variant="outline"
                  className="flex-1"
                >
                  返回
                </Button>
                <Button
                  onClick={handleFinish}
                  className="flex-1"
                >
                  完成
                </Button>
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default DevicePairingModal;
