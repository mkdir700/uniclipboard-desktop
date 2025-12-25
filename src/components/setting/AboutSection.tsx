import React from "react";
import { useTranslation } from "react-i18next";

const AboutSection: React.FC = () => {
  const { t } = useTranslation();
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
            <h4 className="text-lg font-medium">{t("settings.sections.about.appName")}</h4>
            <p className="text-sm text-muted-foreground">{t("settings.sections.about.version")}</p>
          </div>
        </div>
        <button className="px-4 py-2 bg-secondary hover:bg-secondary/80 text-sm font-medium transition duration-200 rounded-lg">
          {t("settings.sections.about.checkUpdate")}
        </button>
      </div>

      <div className="space-y-4 pt-4 border-t border-border/50">
        <p className="text-sm text-muted-foreground">{t("settings.sections.about.copyright")}</p>
        <div className="flex space-x-6 text-sm">
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            {t("settings.sections.about.links.privacyPolicy")}
          </a>
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            {t("settings.sections.about.links.termsOfService")}
          </a>
          <a href="#" className="text-primary hover:text-primary/80 transition-colors">
            {t("settings.sections.about.links.helpCenter")}
          </a>
        </div>
      </div>
    </div>
  );
};

export default AboutSection;
