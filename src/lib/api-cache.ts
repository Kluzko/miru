import { QueryClient } from "@tanstack/react-query";

// React Query configuration optimized for Tauri desktop apps
export function createCachedQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // Desktop apps benefit from longer stale times
        // Data is cached for 15 minutes before considered stale
        staleTime: 15 * 60 * 1000, // 15 minutes

        // Garbage collection time - keep unused data in cache for 30 minutes
        gcTime: 30 * 60 * 1000, // 30 minutes (formerly cacheTime)

        // Retry logic - don't retry on client errors (4xx)
        retry: (failureCount, error: any) => {
          if (error?.status >= 400 && error?.status < 500) {
            return false;
          }
          return failureCount < 2;
        },

        // Exponential backoff for retries
        retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),

        // Desktop app optimizations - disable unnecessary refetching
        refetchOnWindowFocus: false, // No need in desktop apps
        refetchOnReconnect: false, // Desktop apps don't disconnect frequently
        refetchOnMount: true, // Only refetch when component mounts
      },
      mutations: {
        retry: false, // Never retry mutations
      },
    },
  });
}

export default createCachedQueryClient;
