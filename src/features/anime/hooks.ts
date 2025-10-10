import {
  useQuery,
  useInfiniteQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { animeApi } from "./api";

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
  return useQuery({
    queryKey: animeKeys.search(query),
    queryFn: () => animeApi.search(query),
    enabled: (options?.enabled ?? true) && query.length > 2,
    retry: 2,
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // Keep in cache for 10 minutes
    refetchOnWindowFocus: false, // Don't refetch on window focus for search
    placeholderData: options?.keepPreviousData
      ? (previousData) => previousData
      : undefined,
    meta: {
      // Custom metadata for debugging
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
    queryFn: () => animeApi.getById(id),
    enabled: !!id,
    staleTime: 10 * 60 * 1000, // 10 minutes - anime details don't change often
    gcTime: 30 * 60 * 1000, // Keep in cache for 30 minutes
    refetchOnWindowFocus: options?.backgroundRefetch ?? false,
    retry: 3, // More retries for detail views
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    meta: {
      description: `Anime detail for ID: ${id}`,
    },
  });
}

export function useTopAnime(page = 1) {
  return useQuery({
    queryKey: animeKeys.top(page),
    queryFn: () => animeApi.getTop(page),
    staleTime: 15 * 60 * 1000, // 15 minutes - top lists are relatively stable
    gcTime: 60 * 60 * 1000, // 1 hour cache time
    refetchOnWindowFocus: false,
    placeholderData: (previousData) => previousData, // Smooth pagination experience
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
    staleTime: 15 * 60 * 1000,
    gcTime: 60 * 60 * 1000,
    refetchOnWindowFocus: false,
    meta: {
      description: "Infinite scroll top anime list",
    },
  });
}

export function useSeasonalAnime(year: number, season: string) {
  return useQuery({
    queryKey: animeKeys.seasonal(year, season),
    queryFn: () => animeApi.getSeasonal(year, season),
    enabled: !!year && !!season,
    staleTime: 60 * 60 * 1000, // 1 hour - seasonal data is very stable
    gcTime: 24 * 60 * 60 * 1000, // 24 hour cache
    refetchOnWindowFocus: false,
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
      staleTime: 10 * 60 * 1000,
    });
  };

  const prefetchTopAnimePage = (page: number) => {
    queryClient.prefetchQuery({
      queryKey: animeKeys.top(page),
      queryFn: () => animeApi.getTop(page),
      staleTime: 15 * 60 * 1000,
    });
  };

  const prefetchSearchResults = (query: string) => {
    if (query.length > 2) {
      queryClient.prefetchQuery({
        queryKey: animeKeys.search(query),
        queryFn: () => animeApi.search(query),
        staleTime: 5 * 60 * 1000,
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
    queryClient.invalidateQueries({
      queryKey: animeKeys.searches(),
    });
  };

  const invalidateAnimeDetail = (id: string) => {
    queryClient.invalidateQueries({
      queryKey: animeKeys.detail(id),
    });
  };

  const clearAnimeCache = () => {
    queryClient.removeQueries({
      queryKey: animeKeys.all,
    });
  };

  const updateAnimeInCache = (id: string, updater: (old: any) => any) => {
    queryClient.setQueryData(animeKeys.detail(id), updater);
  };

  return {
    invalidateSearches,
    invalidateAnimeDetail,
    clearAnimeCache,
    updateAnimeInCache,
  };
}
