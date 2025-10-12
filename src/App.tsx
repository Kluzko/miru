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
import { createCachedQueryClient } from "@/lib/api-cache";
import { errorLogger } from "@/lib/logger";

// Create optimized query client for Tauri desktop app
const queryClient = createCachedQueryClient();

// Error handler for global error boundary
function handleGlobalError(error: Error, errorInfo: any, errorId: string) {
  errorLogger.error("Global error caught", error, {
    errorId,
    componentStack: errorInfo?.componentStack,
    url: window.location.href,
  });

  // Store error locally for debugging
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
    errorLogger.warn("Failed to store error locally", { error: String(e) });
  }
}

export default function App() {
  return (
    <ErrorBoundary onError={handleGlobalError} resetOnPropsChange={true}>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <Routes>
            <Route path="/" element={<AppLayout />}>
              <Route index element={<DashboardPage />} />
              <Route path="explore" element={<ExplorePage />} />
              <Route path="collections" element={<CollectionsPage />} />
              <Route path="collection/:id" element={<CollectionDetailPage />} />
              <Route path="anime/:id" element={<AnimeDetailPage />} />
              <Route path="settings" element={<SettingsPage />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
