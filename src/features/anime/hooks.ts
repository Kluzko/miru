import { useQuery } from "@tanstack/react-query";
import { animeApi } from "./api";

// Simple query keys factory
export const animeKeys = {
  all: ["anime"] as const,
  search: (q: string) => [...animeKeys.all, "search", q] as const,
  detail: (id: string) => [...animeKeys.all, id] as const,
  top: (page: number) => [...animeKeys.all, "top", page] as const,
  seasonal: (year: number, season: string) =>
    [...animeKeys.all, "seasonal", year, season] as const,
};

export function useAnimeSearch(query: string) {
  return useQuery({
    queryKey: animeKeys.search(query),
    queryFn: () => animeApi.search(query),
    enabled: query.length > 2,
    // React Query handles retries, caching, etc.
    retry: 2,
    staleTime: 5 * 60 * 1000,
  });
}

export function useAnimeDetail(id: string) {
  return useQuery({
    queryKey: animeKeys.detail(id),
    queryFn: () => animeApi.getById(id),
    enabled: !!id,
  });
}

export function useTopAnime(page = 1) {
  return useQuery({
    queryKey: animeKeys.top(page),
    queryFn: () => animeApi.getTop(page),
  });
}
