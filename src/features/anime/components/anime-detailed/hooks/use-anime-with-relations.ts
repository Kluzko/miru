import { useQuery, useQueryClient } from "@tanstack/react-query";
import { animeApi } from "@/features/anime/api";
import { animeKeys } from "@/features/anime/hooks";
import type { AnimeDetailed } from "@/types/bindings";
import { animeLogger } from "@/lib/logger";

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
  animeLogger.debug("useAnimeWithRelations called", {
    animeId,
    enabled: options.enabled,
  });

  const queryClient = useQueryClient();
  const { enabled = true } = options;

  const query = useQuery({
    queryKey: animeKeys.relations(animeId),
    queryFn: () => {
      animeLogger.debug("Fetching anime with relations", { animeId });
      return animeApi.getAnimeWithRelations(animeId) as Promise<
        AnimeWithRelationMetadata[]
      >;
    },
    enabled: enabled && !!animeId,
    staleTime: 30 * 60 * 1000, // 30 minutes
    gcTime: 2 * 60 * 60 * 1000, // 2 hours
    retry: 2,
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
    animeLogger.info("Refreshing relations", { animeId });
    queryClient.invalidateQueries({
      queryKey: animeKeys.relations(animeId),
    });
  };

  const preloadRelations = (targetAnimeId: string) => {
    animeLogger.debug("Preloading relations", { targetAnimeId });
    queryClient.prefetchQuery({
      queryKey: animeKeys.relations(targetAnimeId),
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
    animeLogger.info("Invalidating relations cache", { animeId });
    queryClient.invalidateQueries({
      queryKey: animeKeys.relations(animeId),
    });
  };

  const clearRelationsCache = (animeId: string) => {
    animeLogger.info("Clearing relations cache", { animeId });
    queryClient.removeQueries({
      queryKey: animeKeys.relations(animeId),
    });
  };

  const clearAllRelationsCache = () => {
    animeLogger.warn("Clearing ALL relations cache");
    queryClient.removeQueries({
      queryKey: animeKeys.relationsList(),
    });
  };

  return {
    invalidateRelations,
    clearRelationsCache,
    clearAllRelationsCache,
  };
}
