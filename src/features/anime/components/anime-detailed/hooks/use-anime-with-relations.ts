import { useQuery, useQueryClient } from "@tanstack/react-query";
import { animeApi } from "@/features/anime/api";
import type { AnimeDetailed } from "@/types/bindings";

// Temporary type definition until bindings are regenerated
interface AnimeWithRelationMetadata {
  anime: AnimeDetailed;
  relation_type: string;
  synced_at: string;
}

interface UseAnimeWithRelationsOptions {
  enabled?: boolean;
}

/**
 * Optimized hook that fetches anime relations with complete metadata in a single call
 * This replaces the progressive loading approach with a more efficient batch method
 *
 * Benefits:
 * - Single API call instead of multiple enrichment calls
 * - Complete anime data with all title variants for language preference
 * - Semantic relation categories (mainStory, sideStory, movie, ova)
 * - Relation metadata (relation_type, synced_at)
 */
export function useAnimeWithRelations(
  animeId: string,
  options: UseAnimeWithRelationsOptions = {},
) {
  console.log("🎯 useAnimeWithRelations called with:", {
    animeId,
    enabled: options.enabled,
  });

  const queryClient = useQueryClient();
  const { enabled = true } = options;

  const query = useQuery({
    queryKey: ["anime-with-relations", animeId],
    queryFn: () => {
      console.log("🔍 Calling getAnimeWithRelations for animeId:", animeId);
      return animeApi.getAnimeWithRelations(animeId) as Promise<
        AnimeWithRelationMetadata[]
      >;
    },
    enabled: enabled && !!animeId,
    staleTime: 30 * 60 * 60 * 1000, // 30 minutes
    gcTime: 2 * 60 * 60 * 1000, // 2 hours
    retry: 1, // Reduced since backend handles auto-discovery
    refetchOnWindowFocus: false,
    meta: {
      description: `Batch anime with relations for ${animeId}`,
    },
  });

  // Note: Auto-discovery is now handled by the backend service
  // When relations are empty, the backend automatically discovers and saves them
  // No need for frontend orchestration

  // Computed properties
  const relations = (query.data as AnimeWithRelationMetadata[]) || [];
  const hasRelations = relations.length > 0;

  const relationsByCategory = relations.reduce(
    (acc, item) => {
      const category = item.relation_type || "other";
      if (!acc[category]) {
        acc[category] = [];
      }
      acc[category].push(item);
      return acc;
    },
    {} as Record<string, AnimeWithRelationMetadata[]>,
  );

  // Action handlers
  const refreshRelations = () => {
    queryClient.invalidateQueries({
      queryKey: ["anime-with-relations", animeId],
    });
  };

  const preloadRelations = (targetAnimeId: string) => {
    queryClient.prefetchQuery({
      queryKey: ["anime-with-relations", targetAnimeId],
      queryFn: () => animeApi.getAnimeWithRelations(targetAnimeId),
      staleTime: 30 * 60 * 1000,
    });
  };

  return {
    // Data
    relations,
    relationsByCategory,

    // State
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    hasRelations,

    // Actions
    refreshRelations,
    preloadRelations,

    // Raw query for advanced usage
    query,
  } as const;
}

/**
 * Utility hook for managing anime relations cache
 */
export function useAnimeWithRelationsCache() {
  const queryClient = useQueryClient();

  const invalidateRelations = (animeId: string) => {
    queryClient.invalidateQueries({
      queryKey: ["anime-with-relations", animeId],
    });
  };

  const clearRelationsCache = (animeId: string) => {
    queryClient.removeQueries({
      queryKey: ["anime-with-relations", animeId],
    });
  };

  const clearAllRelationsCache = () => {
    queryClient.removeQueries({
      queryKey: ["anime-with-relations"],
    });
  };

  return {
    invalidateRelations,
    clearRelationsCache,
    clearAllRelationsCache,
  };
}
