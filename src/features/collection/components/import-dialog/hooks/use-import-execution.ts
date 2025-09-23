import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/types";
import type { EnhancedValidationResult } from "@/types";
import { useAddMultipleAnimeToCollection } from "../../../hooks";

interface ImportProgress {
  current: number;
  total: number;
  current_title: string;
  processed: number;
  imported_count: number;
  failed_count: number;
  skipped_count: number;
}

interface EnhancedImportMetrics {
  enhancementsApplied: number;
  qualityImprovements: number;
  averageQualityScore: number;
  providersUsed: string[];
  gapsFilledCount: number;
}

export function useImportExecution() {
  const [isImporting, setIsImporting] = useState(false);
  const [importProgress, setImportProgress] = useState<ImportProgress | null>(
    null,
  );
  const [enhancedMetrics, setEnhancedMetrics] =
    useState<EnhancedImportMetrics | null>(null);
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
    validationResult: EnhancedValidationResult,
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

      // Step 1: Import new anime using already validated data (NO re-validation!)
      if (hasNewAnime) {
        console.log(
          `Starting import of ${validationResult.found.length} pre-validated anime...`,
        );

        // Transform enhanced validated anime to simple validated anime format
        const validatedAnime = validationResult.found.map((enhanced) => ({
          input_title: enhanced.input_title,
          anime_data: enhanced.anime_data,
        }));

        // Use import_validated_anime to avoid re-validation
        const result = await commands.importValidatedAnime({
          validated_anime: validatedAnime,
        });

        if (result.status === "ok") {
          const importResult = result.data;
          const newImportedIds = importResult.imported.map((anime) => anime.id);
          allAnimeIds = [...allAnimeIds, ...newImportedIds];

          // Calculate metrics from the validation result we already have
          const totalConfidence = validationResult.found.reduce(
            (sum, anime) => sum + anime.confidence_score,
            0,
          );
          const avgConfidence = totalConfidence / validationResult.found.length;
          const avgQuality =
            validationResult.data_quality_summary.average_completeness;

          const metrics: EnhancedImportMetrics = {
            enhancementsApplied: validationResult.found.length,
            qualityImprovements:
              validationResult.data_quality_summary.fields_with_gaps.length,
            averageQualityScore: avgQuality * 10, // Convert 0-1 to 0-10 scale
            providersUsed: [
              `${validationResult.data_quality_summary.total_providers_used} providers`,
            ],
            gapsFilledCount: 0, // No gaps filled since we're using pre-validated data
          };
          setEnhancedMetrics(metrics);

          console.log(
            `Successfully imported ${newImportedIds.length} pre-validated anime (no re-validation needed)`,
          );
          console.log(
            `Average confidence: ${(avgConfidence * 100).toFixed(1)}%, Quality: ${(avgQuality * 10).toFixed(1)}/10`,
          );
        } else {
          console.error("Import of validated anime failed:", result.error);
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
      // Keep enhanced metrics available for UI display
    }
  };

  return {
    isImporting,
    importProgress,
    enhancedMetrics,
    handleImport,
  };
}
