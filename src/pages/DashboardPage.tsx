import React, { useState, useEffect, useRef, useCallback } from "react";
import Header from "@/components/layout/Header";
import ClipboardContent from "@/components/clipboard/ClipboardContent";
import { Filter, OrderBy } from "@/api/clipboardItems";
import { useAppDispatch } from "@/store/hooks";
import { fetchClipboardItems } from "@/store/slices/clipboardSlice";
import { useShortcutScope } from "@/hooks/useShortcutScope";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";

// Debounce delay in milliseconds
const DEBOUNCE_DELAY = 500;

// Global listener state management
interface ListenerState {
  isActive: boolean;
  unlisten?: () => void;
  lastEventTimestamp?: number;
}

const globalListenerState: ListenerState = {
  isActive: false,
};

const DashboardPage: React.FC = () => {
  const { t } = useTranslation();
  const [currentFilter, setCurrentFilter] = useState<Filter>(Filter.All);
  const dispatch = useAppDispatch();

  // 设置当前页面作用域为 clipboard
  useShortcutScope("clipboard");

  // Use ref to store the latest filter value
  const currentFilterRef = useRef<Filter>(currentFilter);
  // Debounce ref
  const debouncedLoadRef = useRef<number | null>(null);

  const handleFilterChange = (filterId: Filter) => {
    setCurrentFilter(filterId);
  };

  // Load clipboard records and statistics
  const loadData = useCallback(
    async (specificFilter?: Filter) => {
      const filterToUse = specificFilter || currentFilterRef.current;
      console.log(t("dashboard.logs.loadingClipboard"), filterToUse);

      dispatch(
        fetchClipboardItems({
          orderBy: OrderBy.ActiveTimeDesc,
          filter: filterToUse,
        })
      );
    },
    [dispatch, t]
  );



  // Debounced data loading
  const debouncedLoadData = useCallback(
    (specificFilter?: Filter) => {
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }

      debouncedLoadRef.current = setTimeout(() => {
        loadData(specificFilter);
        debouncedLoadRef.current = null;
      }, DEBOUNCE_DELAY);
    },
    [loadData]
  );

  // Update ref to track the latest filter
  useEffect(() => {
    console.log(t("dashboard.logs.filterChanged"), currentFilter);
    currentFilterRef.current = currentFilter;
    loadData(currentFilter); // Load directly without debounce
  }, [currentFilter, loadData, t]);

  // Setup clipboard content listener
  useEffect(() => {
    // Function to setup listener
    const setupListener = async () => {
      // Only setup if there's no active listener yet
      if (!globalListenerState.isActive) {
        console.log(t("dashboard.logs.settingGlobalListener"));
        globalListenerState.isActive = true;

        try {
          console.log(t("dashboard.logs.startingBackendListener"));
          await invoke("listen_clipboard_new_content");
          console.log(t("dashboard.logs.backendListenerStarted"));

          console.log(t("dashboard.logs.listeningToClipboardEvents"));
          // Clear previously existing listener
          if (globalListenerState.unlisten) {
            console.log(t("dashboard.logs.clearingPreviousListener"));
            globalListenerState.unlisten();
            globalListenerState.unlisten = undefined;
          }

          // Use listen function to listen for global events
          const unlisten = await listen<{
            record_id: string;
            timestamp: number;
          }>("clipboard-new-content", (event) => {
            console.log(t("dashboard.logs.newClipboardEvent"), event);

            // Check event timestamp to avoid processing duplicate events within short time
            const currentTime = Date.now();
            if (
              globalListenerState.lastEventTimestamp &&
              currentTime - globalListenerState.lastEventTimestamp <
                DEBOUNCE_DELAY
            ) {
              console.log(t("dashboard.logs.ignoringDuplicateEvent"));
              return;
            }

            // Update last event timestamp
            globalListenerState.lastEventTimestamp = currentTime;

            // Use debounced function to load data
            debouncedLoadData(currentFilterRef.current);
          });

          // Save unlisten function to global state
          globalListenerState.unlisten = unlisten;
        } catch (err) {
          console.error(t("dashboard.logs.setupListenerFailed"), err);
          globalListenerState.isActive = false;
        }
      } else {
        console.log(t("dashboard.logs.listenerAlreadyActive"));
      }
    };

    // Setup listener if not already set
    if (!globalListenerState.isActive) {
      setupListener();
    } else {
      console.log(t("dashboard.logs.globalListenerExists"));
    }

    // Cleanup function when component unmounts
    return () => {
      // Clear debounce timer
      if (debouncedLoadRef.current) {
        clearTimeout(debouncedLoadRef.current);
      }
      // Don't clean up global listener, keep it active
      console.log(t("dashboard.logs.componentUnmounting"));
    };
  }, [debouncedLoadData, t]);

  return (
    <div className="flex flex-col h-full relative">
      {/* Top search bar */}
      <Header onFilterChange={handleFilterChange} />

      {/* Clipboard content area - use flex-1 to make it take remaining space */}
      <div className="flex-1 overflow-hidden relative">
        <ClipboardContent filter={currentFilter} />
      </div>
    </div>
  );
};

export default DashboardPage;
