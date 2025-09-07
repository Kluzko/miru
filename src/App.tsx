import "./index.css";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ErrorBoundary } from "@/components/common/error-boundary";
import { AppLayout } from "@/components/layout/app-layout";
import { DashboardPage } from "@/pages/dashboard";
import { SearchPage } from "@/pages/search";
import { CollectionDetailPage } from "@/pages/collection/[id]";

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
              <Route path="search" element={<SearchPage />} />
              <Route path="collection/:id" element={<CollectionDetailPage />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
