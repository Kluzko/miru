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
    onSuccess: (_, variables) => {
      // Invalidate anime list for this collection
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
      // Invalidate collection details to update anime count
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collection_id),
      });
      // Invalidate all collections to update counts in lists
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
    },
  });
}

export function useRemoveAnimeFromCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: { collection_id: string; anime_id: string }) =>
      collectionApi.removeAnime(data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collection_id),
      });
    },
  });
}

export function useImportAnimeBatch() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: ImportAnimeBatchRequest) =>
      collectionApi.importBatch(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: collectionKeys.all });
    },
  });
}

export function useImportFromCsv() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: ImportFromCsvRequest) => collectionApi.importCsv(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: collectionKeys.all });
    },
  });
}

// Hook for adding multiple anime to a collection after import
export function useAddMultipleAnimeToCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      collectionId,
      animeIds,
    }: {
      collectionId: string;
      animeIds: string[];
    }) => {
      const results = await Promise.allSettled(
        animeIds.map((animeId) =>
          collectionApi.addAnime({
            collection_id: collectionId,
            anime_id: animeId,
            user_score: null,
            notes: null,
          }),
        ),
      );

      const successes = results.filter(
        (result) => result.status === "fulfilled",
      ).length;
      const failures = results.filter(
        (result) => result.status === "rejected",
      ).length;

      return { successes, failures, total: animeIds.length };
    },
    onSuccess: (_, variables) => {
      // Invalidate anime list for this collection
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collectionId),
      });
      // Invalidate collection details to update anime count
      queryClient.invalidateQueries({
        queryKey: collectionKeys.detail(variables.collectionId),
      });
      // Invalidate all collections to update counts in lists
      queryClient.invalidateQueries({
        queryKey: collectionKeys.all,
      });
    },
  });
}
