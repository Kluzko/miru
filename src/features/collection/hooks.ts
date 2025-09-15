// src/features/collections/hooks.ts
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { collectionApi } from "./api";
import {
  AddAnimeToCollectionRequest,
  Collection,
  CreateCollectionRequest,
  GetCollectionAnimeRequest,
  GetCollectionRequest,
  DeleteCollectionRequest,
  UpdateCollectionRequest,
  ImportAnimeBatchRequest,
  ImportFromCsvRequest,
} from "@/types";

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
      } satisfies GetCollectionAnimeRequest),
    enabled: !!collectionId,
  });
}

// Mutations with optimistic updates
export function useCreateCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateCollectionRequest) => collectionApi.create(data),
    onSuccess: (newCollection: Collection) => {
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? [...old, newCollection] : [newCollection],
      );
    },
  });
}

export function useUpdateCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: UpdateCollectionRequest) => collectionApi.update(data),
    onSuccess: (updated: Collection) => {
      // update list
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? old.map((c) => (c.id === updated.id ? updated : c)) : [updated],
      );
      // update detail
      queryClient.setQueryData(collectionKeys.detail(updated.id), updated);
    },
  });
}

export function useDeleteCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: DeleteCollectionRequest) => collectionApi.delete(data),
    onSuccess: (_, variables) => {
      // remove from list
      queryClient.setQueryData(
        collectionKeys.all,
        (old: Collection[] | undefined) =>
          old ? old.filter((c) => c.id !== variables.id) : old,
      );
      // invalidate detail and anime queries
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.id),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.id),
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
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
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
    onError: (_, variables, context) => {
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
      // Refetch to ensure consistency
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
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
      await queryClient.cancelQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
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
    onError: (_, variables, context) => {
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
    onSuccess: (_, variables) => {
      // Only invalidate specific collection if targeting one
      if (variables.collection_id) {
        queryClient.invalidateQueries({
          queryKey: collectionKeys.detail(variables.collection_id),
        });
        queryClient.invalidateQueries({
          queryKey: collectionKeys.anime(variables.collection_id),
        });
      }

      // Invalidate anime search results since new anime may have been added
      queryClient.invalidateQueries({
        queryKey: ["anime", "search"],
        refetchType: "none", // Don't refetch automatically, let user trigger
      });

      // Update collections list to reflect new counts
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
    },
  });
}

export function useImportFromCsv() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: ImportFromCsvRequest) => collectionApi.importCsv(data),
    onSuccess: (_, variables) => {
      // Similar optimizations as batch import
      if (variables.collection_id) {
        queryClient.invalidateQueries({
          queryKey: collectionKeys.detail(variables.collection_id),
        });
        queryClient.invalidateQueries({
          queryKey: collectionKeys.anime(variables.collection_id),
        });
      }

      queryClient.invalidateQueries({
        queryKey: ["anime", "search"],
        refetchType: "none",
      });

      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
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
      const BATCH_SIZE = 5; // Process in smaller batches to avoid overwhelming the backend
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

        // Small delay between batches to be respectful to the backend
        if (i + BATCH_SIZE < animeIds.length) {
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }

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

      // Optimistically update collection count
      const previousCollection = queryClient.getQueryData(
        collectionKeys.detail(variables.collectionId),
      );
      const previousCollections = queryClient.getQueryData(collectionKeys.all);

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
      // If some operations failed, adjust the optimistic update
      const actualAdditions = result.successes;
      const optimisticAdditions = variables.animeIds.length;
      const difference = optimisticAdditions - actualAdditions;

      if (difference > 0) {
        // We were too optimistic, reduce the count
        queryClient.setQueryData(
          collectionKeys.detail(variables.collectionId),
          (old: Collection | undefined) =>
            old
              ? {
                  ...old,
                  animeCount: (old.animeCount || difference) - difference,
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
                    animeCount: (c.animeCount || difference) - difference,
                  }
                : c,
            ),
        );
      }
    },
    onError: (_, variables, context) => {
      // Rollback optimistic updates on complete failure
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
      // Always refetch to ensure data consistency
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collectionId),
      });
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collectionId),
      });
    },
  });
}
