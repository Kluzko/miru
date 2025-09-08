import { useMutation, useQueryClient } from "@tanstack/react-query"
import { commands } from "@/types"
import { collectionKeys } from "../../hooks"

interface AddMultipleAnimeToCollectionParams {
  collectionId: string
  animeIds: string[]
}

export function useAddMultipleAnimeToCollection() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ collectionId, animeIds }: AddMultipleAnimeToCollectionParams) => {
      // Add each anime individually - TODO: Backend could optimize this with batch API
      const results = await Promise.allSettled(
        animeIds.map(animeId =>
          commands.addAnimeToCollection({
            collection_id: collectionId,
            anime_id: animeId,
            user_score: null,
            notes: null,
          })
        )
      )

      // Count successes and failures
      const successes = results.filter(result => result.status === "fulfilled").length
      const failures = results.filter(result => result.status === "rejected").length

      return { successes, failures, total: animeIds.length }
    },
    onSuccess: (_, variables) => {
      // Invalidate collection anime query to refresh the UI
      queryClient.invalidateQueries({
        queryKey: collectionKeys.anime(variables.collectionId),
      })
    },
  })
}
