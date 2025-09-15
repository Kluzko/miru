import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/types";
import type { ValidationResult } from "@/types";
import { useAddMultipleAnimeToCollection } from "../../hooks";

interface ImportProgress {
  current: number;
  total: number;
  current_title: string;
  processed: number;
  imported_count: number;
  failed_count: number;
  skipped_count: number;
}

export function useImportExecution() {
  const [isImporting, setIsImporting] = useState(false);
  const [importProgress, setImportProgress] = useState<ImportProgress | null>(
    null,
  );
  const addToCollection = useAddMultipleAnimeToCollection();

  // Set up real-time import progress listener
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        unlisten = await listen<ImportProgress>("import_progress", (event) => {
          const progress = event.payload;
          setImportProgress(progress);

          // Log import progress for debugging
          console.log(
            `Import progress: ${progress.processed}/${progress.total} - ${progress.current_title}`,
          );
        });
      } catch (error) {
        console.warn("Failed to set up import progress listener:", error);
      }
    };

    if (isImporting) {
      setupListener();
    }

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [isImporting]);

  const handleImport = async (
    validationResult: ValidationResult,
    selectedExisting: Set<string>,
    collectionId?: string,
    onAnimesImported?: (animeIds: string[]) => void,
  ) => {
    const hasNewAnime = validationResult.found.length > 0;
    const hasSelectedExisting = selectedExisting.size > 0;

    if (!hasNewAnime && !hasSelectedExisting) return;

    setIsImporting(true);
    setImportProgress(null);

    try {
      let allAnimeIds: string[] = [];

      // Step 1: Import new anime to database with real-time progress
      if (hasNewAnime) {
        console.log(
          `Starting import of ${validationResult.found.length} new anime...`,
        );

        const result = await commands.importValidatedAnime({
          validated_anime: validationResult.found,
        });

        if (result.status === "ok") {
          const newImportedIds = result.data.imported.map((anime) => anime.id);
          allAnimeIds = [...allAnimeIds, ...newImportedIds];

          console.log(
            `Successfully imported ${newImportedIds.length} new anime`,
          );
        } else {
          console.error("Import failed:", result.error);
          throw new Error(result.error);
        }
      }

      // Step 2: Add selected existing anime to the list
      if (hasSelectedExisting) {
        const existingIds = Array.from(selectedExisting);
        allAnimeIds = [...allAnimeIds, ...existingIds];
        console.log(`Adding ${existingIds.length} existing anime to the list`);
      }

      // Step 3: If we have a collection context, add all anime to the collection
      if (collectionId && allAnimeIds.length > 0) {
        console.log(
          `Adding ${allAnimeIds.length} anime to collection ${collectionId}`,
        );

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
      setImportProgress(null);
    }
  };

  return {
    isImporting,
    importProgress,
    handleImport,
  };
}
