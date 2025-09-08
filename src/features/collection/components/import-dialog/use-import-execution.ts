import { useState } from "react";
import { commands } from "@/types";
import type { ValidationResult } from "@/types";
import { useAddMultipleAnimeToCollection } from "../../hooks";

export function useImportExecution() {
  const [isImporting, setIsImporting] = useState(false);
  const addToCollection = useAddMultipleAnimeToCollection();

  const handleImport = async (
    validationResult: ValidationResult,
    selectedExisting: Set<string>,
    collectionId?: string,
    onAnimesImported?: (animeIds: string[]) => void
  ) => {
    const hasNewAnime = validationResult.found.length > 0;
    const hasSelectedExisting = selectedExisting.size > 0;

    if (!hasNewAnime && !hasSelectedExisting) return;

    setIsImporting(true);
    try {
      let allAnimeIds: string[] = [];

      // Step 1: Import new anime to database
      if (hasNewAnime) {
        const result = await commands.importValidatedAnime({
          validated_anime: validationResult.found,
        });

        if (result.status === "ok") {
          const newImportedIds = result.data.imported.map((anime) => anime.id);
          allAnimeIds = [...allAnimeIds, ...newImportedIds];
        }
      }

      // Step 2: Add selected existing anime to the list
      if (hasSelectedExisting) {
        const existingIds = Array.from(selectedExisting);
        allAnimeIds = [...allAnimeIds, ...existingIds];
      }

      // Step 3: If we have a collection context, add all anime to the collection
      if (collectionId && allAnimeIds.length > 0) {
        await addToCollection.mutateAsync({
          collectionId,
          animeIds: allAnimeIds,
        });
      }

      onAnimesImported?.(allAnimeIds);
      return allAnimeIds;
    } catch (error) {
      console.error("Import failed:", error);
      throw error;
    } finally {
      setIsImporting(false);
    }
  };

  return {
    isImporting,
    handleImport,
  };
}
