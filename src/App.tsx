import "./index.css";

import React from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { AppLayout } from "@/components/layout/app-layout";
import {
  DashboardPage,
  ExplorePage,
  CollectionDetailPage,
  CollectionsPage,
  SettingsPage,
  AnimeDetailPage,
} from "@/pages";
import { createCachedQueryClient, CacheWarmer } from "@/lib/api-cache";
import { PerformanceErrorBoundary } from "@/lib/performance";

// Create optimized query client with enhanced caching
const queryClient = createCachedQueryClient();

// Error handler for global error boundary
function handleGlobalError(error: Error, errorInfo: any, errorId: string) {
  // In production, this would send to your error reporting service
  // Example: Sentry.captureException(error, { extra: errorInfo, tags: { errorId } });

  console.error(`Global Error [${errorId}]:`, error, errorInfo);

  // Could also store error locally for offline reporting
  try {
    const errorData = {
      error: { message: error.message, stack: error.stack },
      errorInfo,
      errorId,
      timestamp: Date.now(),
      url: window.location.href,
    };
    localStorage.setItem(`error_${errorId}`, JSON.stringify(errorData));
  } catch (e) {
    console.warn("Failed to store error locally:", e);
  }
}

export default function App() {
  // Warm cache with popular content on app startup
  React.useEffect(() => {
    const warmer = CacheWarmer.getInstance();
    warmer.warmPopularContent().catch((err) => {
      console.warn("Cache warming failed:", err);
    });
  }, []);

  // Performance monitoring in development
  React.useEffect(() => {
    if (process.env.NODE_ENV === "development") {
      // Monitor memory usage
      const logMemoryUsage = () => {
        if ("memory" in performance) {
          const memory = (performance as any).memory;
          console.log("🧠 Memory Usage:", {
            used: `${(memory.usedJSHeapSize / 1024 / 1024).toFixed(2)}MB`,
            total: `${(memory.totalJSHeapSize / 1024 / 1024).toFixed(2)}MB`,
            limit: `${(memory.jsHeapSizeLimit / 1024 / 1024).toFixed(2)}MB`,
          });
        }
      };

      const interval = setInterval(logMemoryUsage, 30000); // Every 30 seconds
      return () => clearInterval(interval);
    }
  }, []);

  return (
    <PerformanceErrorBoundary componentName="App">
      <ErrorBoundary onError={handleGlobalError} resetOnPropsChange={true}>
        <QueryClientProvider client={queryClient}>
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<AppLayout />}>
                <Route index element={<DashboardPage />} />
                <Route path="explore" element={<ExplorePage />} />
                <Route path="collections" element={<CollectionsPage />} />
                <Route
                  path="collection/:id"
                  element={<CollectionDetailPage />}
                />
                <Route path="anime/:id" element={<AnimeDetailPage />} />
                <Route path="settings" element={<SettingsPage />} />
              </Route>
            </Routes>
          </BrowserRouter>
        </QueryClientProvider>
      </ErrorBoundary>
    </PerformanceErrorBoundary>
  );
}
