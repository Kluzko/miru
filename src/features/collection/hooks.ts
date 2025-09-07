import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { collectionApi } from "./api";

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
    queryFn: () => collectionApi.get(id),
    enabled: !!id,
  });
}

export function useCollectionAnime(collectionId: string) {
  return useQuery({
    queryKey: collectionKeys.anime(collectionId),
    queryFn: () => collectionApi.getAnime(collectionId),
    enabled: !!collectionId,
  });
}

// Mutations with optimistic updates
export function useCreateCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      name,
      description,
    }: {
      name: string;
      description?: string;
    }) => collectionApi.create(name, description),
    onSuccess: (newCollection) => {
      // Update cache with new collection
      queryClient.setQueryData(collectionKeys.all, (old: any) =>
        old ? [...old, newCollection] : [newCollection],
      );
    },
  });
}

export function useAddAnimeToCollection() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      collectionId,
      animeId,
      userScore,
      notes,
    }: {
      collectionId: string;
      animeId: string;
      userScore?: number;
      notes?: string;
    }) => collectionApi.addAnime(collectionId, animeId, userScore, notes),

    onSuccess: (_, variables) => {
      // Invalidate the anime list for this collection
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collectionId),
      });
    },
  });
}

export function useImportAnimeBatch() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: collectionApi.importBatch,
    onSuccess: () => {
      // Invalidate all collections as counts may have changed
      queryClient.invalidateQueries({ queryKey: collectionKeys.all });
    },
  });
}
