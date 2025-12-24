import React from "react";

const AboutSection: React.FC = () => {
  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center">
          <div className="h-12 w-12 rounded-xl bg-gradient-to-br from-primary to-primary/60 flex items-center justify-center shadow-lg shadow-primary/20">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-7 w-7 text-primary-foreground"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
              />
            </svg>
          </div>
          <div className="ml-4 space-y-0.5">
            <h4 className="text-lg font-medium">UniClipboard</h4>
            <p className="text-sm text-muted-foreground">版本 0.1.0</p>
          </div>
        </div>
        <button className="px-4 py-2 bg-secondary hover:bg-secondary/80 text-sm font-medium transition duration-200 rounded-lg">
          检查更新
        </button>
      </div>

      <div className="space-y-4 pt-4 border-t border-border/50">
        <p className="text-sm text-muted-foreground">© 2025 UniClipboard Team. All rights reserved.</p>
        <div className="flex space-x-6 text-sm">
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            隐私政策
          </a>
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            使用条款
          </a>
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            帮助中心
          </a>
        </div>
      </div>
    </div>
  );
};

export default AboutSection;
