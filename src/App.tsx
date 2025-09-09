import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ErrorBoundary } from "@/components/common/error-boundary";
import { AppLayout } from "@/components/layout/app-layout";
import {
  DashboardPage,
  ExplorePage,
  CollectionDetailPage,
  CollectionsPage,
  SettingsPage,
  AnimeDetailPage,
} from "@/pages";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      retry: 2,
      refetchOnWindowFocus: false,
    },
  },
});

export default function App() {
  return (
    <ErrorBoundary>
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
