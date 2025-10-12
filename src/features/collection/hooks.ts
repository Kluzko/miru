import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { collectionApi } from "./api";
import {
  AddAnimeToCollectionRequest,
  Collection,
  CreateCollectionRequest,
  GetCollectionRequest,
  DeleteCollectionRequest,
  UpdateCollectionRequest,
  ImportAnimeBatchRequest,
} from "@/types";
import { collectionLogger } from "@/lib/logger";

export const collectionKeys = {
  all: ["collections"] as const,
  detail: (id: string) => [...collectionKeys.all, id] as const,
  anime: (id: string) => [...collectionKeys.all, id, "anime"] as const,
};

// Queries
export function useCollections() {
  return useQuery({
    queryKey: collectionKeys.all,
    queryFn: collectionApi.getAll,
  });
}

export function useCollection(id: string) {
  return useQuery({
    queryKey: collectionKeys.detail(id),
    queryFn: () => collectionApi.get({ id } satisfies GetCollectionRequest),
    enabled: !!id,
  });
}

export function useCollectionAnime(collectionId: string) {
  return useQuery({
    queryKey: collectionKeys.anime(collectionId),
    queryFn: () =>
      collectionApi.getAnime({
        collection_id: collectionId,
      }),
    enabled: !!collectionId,
  });
}

// Mutations with optimistic updates
export function useCreateCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateCollectionRequest) => collectionApi.create(data),
    onSuccess: (newCollection: Collection) => {
      collectionLogger.info("Collection created", {
        id: newCollection.id,
        name: newCollection.name,
      });

      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? [...old, newCollection] : [newCollection],
      );
    },
    onError: (error) => {
      collectionLogger.error("Failed to create collection", { error });
    },
  });
}

export function useUpdateCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: UpdateCollectionRequest) => collectionApi.update(data),
    onSuccess: (updated: Collection) => {
      collectionLogger.info("Collection updated", {
        id: updated.id,
        name: updated.name,
      });

      // Update list
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? old.map((c) => (c.id === updated.id ? updated : c)) : [updated],
      );

      // Update detail
      queryClient.setQueryData(collectionKeys.detail(updated.id), updated);
    },
    onError: (error, variables) => {
      collectionLogger.error("Failed to update collection", {
        id: variables.id,
        error,
      });
    },
  });
}

export function useDeleteCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: DeleteCollectionRequest) => collectionApi.delete(data),
    onSuccess: (_, variables) => {
      collectionLogger.info("Collection deleted", { id: variables.id });

      // Remove from list
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? old.filter((c) => c.id !== variables.id) : old,
      );

      // Invalidate detail and anime queries
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.id),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.id),
      });
    },
    onError: (error, variables) => {
      collectionLogger.error("Failed to delete collection", {
        id: variables.id,
        error,
      });
    },
  });
}

export function useAddAnimeToCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: AddAnimeToCollectionRequest) =>
      collectionApi.addAnime(data),
    onMutate: async (variables) => {
      collectionLogger.debug("Adding anime to collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
      });

      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.all,
      });

      // Snapshot previous values for rollback
      const previousCollection = queryClient.getQueryData(
        collectionKeys.detail(variables.collection_id),
      );
      const previousCollections = queryClient.getQueryData(collectionKeys.all);

      // Optimistically update collection anime count
      queryClient.setQueryData(
        collectionKeys.detail(variables.collection_id),
        (old: Collection | undefined) =>
          old ? { ...old, animeCount: (old.animeCount || 0) + 1 } : old,
      );

      // Optimistically update collections list
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old?.map((c) =>
            c.id === variables.collection_id
              ? { ...c, animeCount: (c.animeCount || 0) + 1 }
              : c,
          ),
      );

      return { previousCollection, previousCollections };
    },
    onSuccess: (_, variables) => {
      collectionLogger.info("Anime added to collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
      });
    },
    onError: (error, variables, context) => {
      collectionLogger.error("Failed to add anime to collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
        error,
      });

      // Rollback on error
      if (context?.previousCollection) {
        queryClient.setQueryData(
          collectionKeys.detail(variables.collection_id),
          context.previousCollection,
        );
      }
      if (context?.previousCollections) {
        queryClient.setQueryData(
          collectionKeys.all,
          context.previousCollections,
        );
      }
    },
    onSettled: (_, __, variables) => {
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
    },
  });
}

export function useRemoveAnimeFromCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { collection_id: string; anime_id: string }) =>
      collectionApi.removeAnime(data),
    onMutate: async (variables) => {
      collectionLogger.debug("Removing anime from collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
      });

      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.all,
      });

      // Snapshot previous values
      const previousCollection = queryClient.getQueryData(
        collectionKeys.detail(variables.collection_id),
      );
      const previousCollections = queryClient.getQueryData(collectionKeys.all);

      // Optimistically decrease anime count
      queryClient.setQueryData(
        collectionKeys.detail(variables.collection_id),
        (old: Collection | undefined) =>
          old
            ? { ...old, animeCount: Math.max((old.animeCount || 1) - 1, 0) }
            : old,
      );

      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old?.map((c) =>
            c.id === variables.collection_id
              ? { ...c, animeCount: Math.max((c.animeCount || 1) - 1, 0) }
              : c,
          ),
      );

      return { previousCollection, previousCollections };
    },
    onSuccess: (_, variables) => {
      collectionLogger.info("Anime removed from collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
      });
    },
    onError: (error, variables, context) => {
      collectionLogger.error("Failed to remove anime from collection", {
        collectionId: variables.collection_id,
        animeId: variables.anime_id,
        error,
      });

      // Rollback on error
      if (context?.previousCollection) {
        queryClient.setQueryData(
          collectionKeys.detail(variables.collection_id),
          context.previousCollection,
        );
      }
      if (context?.previousCollections) {
        queryClient.setQueryData(
          collectionKeys.all,
          context.previousCollections,
        );
      }
    },
    onSettled: (_, __, variables) => {
      // FIX: Invalidate ALL affected queries including the list
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
    },
  });
}

export function useImportAnimeBatch() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: ImportAnimeBatchRequest) =>
      collectionApi.importBatch(data),
    onSuccess: (result) => {
      collectionLogger.info("Anime batch imported", { result });

      // FIX: Removed refetchType: "none" - let React Query handle refetching normally
      queryClient.invalidateQueries({
        queryKey: ["anime", "search"],
      });

      // Update collections list to reflect new counts
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
    },
    onError: (error) => {
      collectionLogger.error("Failed to import anime batch", { error });
    },
  });
}

// Optimized hook for adding multiple anime to a collection
export function useAddMultipleAnimeToCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      collectionId,
      animeIds,
      onProgress,
    }: {
      collectionId: string;
      animeIds: string[];
      onProgress?: (progress: {
        current: number;
        total: number;
        successes: number;
        failures: number;
      }) => void;
    }) => {
      collectionLogger.info("Starting batch add", {
        collectionId,
        count: animeIds.length,
      });

      const BATCH_SIZE = 5;
      const total = animeIds.length;
      let successes = 0;
      let failures = 0;
      let current = 0;
      const failedItems: Array<{ animeId: string; error: string }> = [];

      // Process in batches
      for (let i = 0; i < animeIds.length; i += BATCH_SIZE) {
        const batch = animeIds.slice(i, i + BATCH_SIZE);
        const batchResults = await Promise.allSettled(
          batch.map((animeId) =>
            collectionApi.addAnime({
              collection_id: collectionId,
              anime_id: animeId,
              user_score: null,
              notes: null,
            }),
          ),
        );

        // Process batch results
        batchResults.forEach((result, index) => {
          current++;
          if (result.status === "fulfilled") {
            successes++;
          } else {
            failures++;
            failedItems.push({
              animeId: batch[index],
              error: result.reason?.message || "Unknown error",
            });
          }
        });

        // Report progress
        onProgress?.({ current, total, successes, failures });

        // Small delay between batches
        if (i + BATCH_SIZE < animeIds.length) {
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }

      collectionLogger.info("Batch add completed", {
        collectionId,
        successes,
        failures,
        total,
      });

      return {
        successes,
        failures,
        total,
        failedItems: failures > 0 ? failedItems : undefined,
      };
    },
    onMutate: async (variables) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: collectionKeys.detail(variables.collectionId),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.all,
      });

      // Snapshot for rollback
      const previousCollection = queryClient.getQueryData(
        collectionKeys.detail(variables.collectionId),
      );
      const previousCollections = queryClient.getQueryData(collectionKeys.all);

      // Optimistically update counts
      queryClient.setQueryData(
        collectionKeys.detail(variables.collectionId),
        (old: Collection | undefined) =>
          old
            ? {
                ...old,
                animeCount: (old.animeCount || 0) + variables.animeIds.length,
              }
            : old,
      );

      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old?.map((c) =>
            c.id === variables.collectionId
              ? {
                  ...c,
                  animeCount: (c.animeCount || 0) + variables.animeIds.length,
                }
              : c,
          ),
      );

      return { previousCollection, previousCollections };
    },
    onSuccess: (result, variables) => {
      // Adjust optimistic update if some failed
      const actualAdditions = result.successes;
      const optimisticAdditions = variables.animeIds.length;
      const difference = optimisticAdditions - actualAdditions;

      if (difference > 0) {
        collectionLogger.warn("Adjusting optimistic update", {
          expected: optimisticAdditions,
          actual: actualAdditions,
        });

        queryClient.setQueryData(
          collectionKeys.detail(variables.collectionId),
          (old: Collection | undefined) =>
            old
              ? {
                  ...old,
                  animeCount: Math.max((old.animeCount || 0) - difference, 0),
                }
              : old,
        );

        queryClient.setQueryData(
          collectionKeys.all,
          (old: Collection[] | undefined) =>
            old?.map((c) =>
              c.id === variables.collectionId
                ? {
                    ...c,
                    animeCount: Math.max((c.animeCount || 0) - difference, 0),
                  }
                : c,
            ),
        );
      }
    },
    onError: (error, variables, context) => {
      collectionLogger.error("Batch add failed", {
        collectionId: variables.collectionId,
        error,
      });

      // Rollback on complete failure
      if (context?.previousCollection) {
        queryClient.setQueryData(
          collectionKeys.detail(variables.collectionId),
          context.previousCollection,
        );
      }
      if (context?.previousCollections) {
        queryClient.setQueryData(
          collectionKeys.all,
          context.previousCollections,
        );
      }
    },
    onSettled: (_, __, variables) => {
      // FIX: Invalidate all affected queries
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collectionId),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collectionId),
      });
    },
  });
}
