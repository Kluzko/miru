import {
  useQuery,
  useInfiniteQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { animeApi } from "./api";
import { animeLogger } from "@/lib/logger";

// Enhanced query keys factory with better categorization
export const animeKeys = {
  all: ["anime"] as const,
  search: (q: string) => [...animeKeys.all, "search", q] as const,
  searches: () => [...animeKeys.all, "search"] as const,
  detail: (id: string) => [...animeKeys.all, "detail", id] as const,
  details: () => [...animeKeys.all, "detail"] as const,
  top: (page: number) => [...animeKeys.all, "top", page] as const,
  topList: () => [...animeKeys.all, "top"] as const,
  seasonal: (year: number, season: string) =>
    [...animeKeys.all, "seasonal", year, season] as const,
  seasonalList: () => [...animeKeys.all, "seasonal"] as const,
  relations: (id: string) => [...animeKeys.all, "relations", id] as const,
  relationsList: () => [...animeKeys.all, "relations"] as const,
};

export function useAnimeSearch(
  query: string,
  options?: {
    enabled?: boolean;
    keepPreviousData?: boolean;
  },
) {
  const isEnabled = (options?.enabled ?? true) && query.length > 2;

  return useQuery({
    queryKey: animeKeys.search(query),
    queryFn: async () => {
      animeLogger.debug("Searching anime", { query });
      const timer = animeLogger.startTimed("Anime search");
      try {
        const result = await animeApi.search(query);
        timer.success({ query, results: result?.length || 0 });
        return result;
      } catch (error) {
        timer.error(error, { query });
        throw error;
      }
    },
    enabled: isEnabled,
    // Use global staleTime/gcTime from api-cache.ts (15min/30min)
    placeholderData: options?.keepPreviousData
      ? (previousData) => previousData
      : undefined,
    meta: {
      description: `Search anime with query: ${query}`,
    },
  });
}

export function useAnimeDetail(
  id: string,
  options?: {
    backgroundRefetch?: boolean;
  },
) {
  return useQuery({
    queryKey: animeKeys.detail(id),
    queryFn: async () => {
      animeLogger.debug("Fetching anime detail", { id });
      const timer = animeLogger.startTimed("Fetch anime detail");
      try {
        const result = await animeApi.getById(id);
        timer.success({ id, title: result?.title?.main || "Unknown" });
        return result;
      } catch (error) {
        timer.error(error, { id });
        throw error;
      }
    },
    enabled: !!id,
    // Use global staleTime/gcTime from api-cache.ts (15min/30min)
    refetchOnWindowFocus: options?.backgroundRefetch ?? false,
    meta: {
      description: `Anime detail for ID: ${id}`,
    },
  });
}

export function useTopAnime(page = 1) {
  return useQuery({
    queryKey: animeKeys.top(page),
    queryFn: async () => {
      animeLogger.debug("Fetching top anime", { page });
      const timer = animeLogger.startTimed("Fetch top anime");
      try {
        const result = await animeApi.getTop(page);
        timer.success({ page, count: result?.length || 0 });
        return result;
      } catch (error) {
        timer.error(error, { page });
        throw error;
      }
    },
    // Use global staleTime/gcTime from api-cache.ts (15min/30min)
    placeholderData: (previousData) => previousData,
    meta: {
      description: `Top anime page ${page}`,
    },
  });
}

// Infinite query for better UX with large lists
export function useInfiniteTopAnime() {
  return useInfiniteQuery({
    queryKey: [...animeKeys.topList(), "infinite"],
    queryFn: ({ pageParam = 1 }) => animeApi.getTop(pageParam),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      // Assume API returns hasNextPage or similar
      return lastPage.length === 25 ? allPages.length + 1 : undefined;
    },
    // Use global staleTime/gcTime from api-cache.ts (15min/30min)
    meta: {
      description: "Infinite scroll top anime list",
    },
  });
}

export function useSeasonalAnime(year: number, season: string) {
  return useQuery({
    queryKey: animeKeys.seasonal(year, season),
    queryFn: async () => {
      animeLogger.debug("Fetching seasonal anime", { year, season });
      const timer = animeLogger.startTimed("Fetch seasonal anime");
      try {
        const result = await animeApi.getSeasonal(year, season);
        timer.success({ year, season, count: result?.length || 0 });
        return result;
      } catch (error) {
        timer.error(error, { year, season });
        throw error;
      }
    },
    enabled: !!year && !!season,
    // Override: Seasonal data is very stable, cache longer
    staleTime: 60 * 60 * 1000, // 1 hour
    gcTime: 24 * 60 * 60 * 1000, // 24 hours
    meta: {
      description: `Seasonal anime for ${season} ${year}`,
    },
  });
}

// Note: useAnimeRelations has been removed
// Use useAnimeWithRelations from @/features/anime/components/anime-detailed/hooks instead
// The new hook uses a single optimized backend call with auto-discovery

// Background prefetching utilities
export function useAnimePrefetch() {
  const queryClient = useQueryClient();

  const prefetchAnimeDetail = (id: string) => {
    queryClient.prefetchQuery({
      queryKey: animeKeys.detail(id),
      queryFn: () => animeApi.getById(id),
      // Uses global staleTime from api-cache.ts
    });
  };

  const prefetchTopAnimePage = (page: number) => {
    queryClient.prefetchQuery({
      queryKey: animeKeys.top(page),
      queryFn: () => animeApi.getTop(page),
      // Uses global staleTime from api-cache.ts
    });
  };

  const prefetchSearchResults = (query: string) => {
    if (query.length > 2) {
      queryClient.prefetchQuery({
        queryKey: animeKeys.search(query),
        queryFn: () => animeApi.search(query),
        // Uses global staleTime from api-cache.ts
      });
    }
  };

  return {
    prefetchAnimeDetail,
    prefetchTopAnimePage,
    prefetchSearchResults,
  };
}

// Note: useAnimeRelationsCacheUtils has been removed
// Relations cache utilities are now part of useAnimeWithRelationsCache hook

// Cache management utilities
export function useAnimeCacheUtils() {
  const queryClient = useQueryClient();

  const invalidateSearches = () => {
    animeLogger.info("Invalidating all search caches");
    queryClient.invalidateQueries({
      queryKey: animeKeys.searches(),
    });
  };

  const invalidateAnimeDetail = (id: string) => {
    animeLogger.info("Invalidating anime detail cache", { id });
    queryClient.invalidateQueries({
      queryKey: animeKeys.detail(id),
    });
  };

  const clearAnimeCache = () => {
    animeLogger.warn("Clearing ALL anime cache");
    queryClient.removeQueries({
      queryKey: animeKeys.all,
    });
  };

  const updateAnimeInCache = (id: string, updater: (old: any) => any) => {
    animeLogger.debug("Updating anime in cache", { id });
    queryClient.setQueryData(animeKeys.detail(id), updater);
  };

  return {
    invalidateSearches,
    invalidateAnimeDetail,
    clearAnimeCache,
    updateAnimeInCache,
  };
}
