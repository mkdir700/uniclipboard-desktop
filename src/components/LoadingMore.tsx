import React from "react";

const LoadingMore: React.FC = () => {
  return (
    <div className="py-6 flex flex-col items-center justify-center text-gray-500">
      <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-violet-400 mb-3"></div>
      <p className="text-sm">加载更多历史记录...</p>
    </div>
  );
};

export default LoadingMore;
