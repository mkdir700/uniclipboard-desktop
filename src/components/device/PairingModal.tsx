import React, { useState } from "react";

interface DevicePairingModalProps {
  onClose: () => void;
}

const DevicePairingModal: React.FC<DevicePairingModalProps> = ({ onClose }) => {
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

  return (
    <div className="fixed inset-0 bg-gray-950/90 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-lg w-full max-w-md mx-4">
        <div className="p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xl font-semibold text-white">添加新设备</h3>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-white transition"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          {step === 1 && (
            <div>
              <p className="text-gray-300 mb-6">
                要添加新设备，请在新设备上安装 ClipSync
                应用并选择"添加现有账户"选项。然后点击下方按钮生成配对码。
              </p>

              <div className="flex items-center justify-center mb-6">
                <div className="h-32 w-32 bg-gray-700 rounded-lg flex items-center justify-center">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-16 w-16 text-violet-300"
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
                </div>
              </div>

              <button
                onClick={handleGenerateCode}
                className="w-full py-3 bg-violet-500 hover:bg-violet-400 text-white rounded-lg font-medium transition duration-150"
              >
                生成配对码
              </button>
            </div>
          )}

          {step === 2 && (
            <div>
              <p className="text-gray-300 mb-6">
                请在新设备上输入以下配对码。此代码将在10分钟后过期。
              </p>

              <div className="flex items-center justify-center mb-6">
                <div className="bg-gray-700 rounded-lg p-6">
                  <div className="flex space-x-3">
                    {code.split("").map((digit, index) => (
                      <div
                        key={index}
                        className="w-10 h-12 bg-gray-600 rounded-md flex items-center justify-center text-xl font-bold text-white"
                      >
                        {digit}
                      </div>
                    ))}
                  </div>
                </div>
              </div>

              <p className="text-gray-400 text-sm text-center mb-6">
                配对码有效期：10:00
              </p>

              <div className="flex space-x-3">
                <button
                  onClick={() => setStep(1)}
                  className="flex-1 py-3 bg-gray-600 hover:bg-gray-500 text-white rounded-lg font-medium transition duration-150"
                >
                  返回
                </button>
                <button
                  onClick={handleFinish}
                  className="flex-1 py-3 bg-violet-500 hover:bg-violet-400 text-white rounded-lg font-medium transition duration-150"
                >
                  完成
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default DevicePairingModal;
