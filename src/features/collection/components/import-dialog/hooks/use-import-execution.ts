import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/types";
import type { EnhancedValidationResult } from "@/types";
import { useAddMultipleAnimeToCollection } from "../../../hooks";
import { importLogger } from "@/lib/logger";

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
  importDurationMs: number;
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

          importLogger.importProgress(
            progress.processed,
            progress.total,
            progress.current_title,
          );
        });
      } catch (error) {
        importLogger.error("Failed to set up import progress listener", error);
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
      let importDuration = 0;

      // Step 1: Import new anime using already validated data (NO re-validation!)
      if (hasNewAnime) {
        importLogger.info("Starting import of pre-validated anime", {
          count: validationResult.found.length,
        });

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

          // Get import duration from backend result
          importDuration = importResult.duration_ms || 0;

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
            importDurationMs: importDuration,
          };

          setEnhancedMetrics(metrics);

          importLogger.success("Import completed (no re-validation needed)", {
            imported: newImportedIds.length,
            avgConfidence: `${(avgConfidence * 100).toFixed(1)}%`,
            avgQuality: `${(avgQuality * 10).toFixed(1)}/10`,
            durationMs: importDuration,
          });
        } else {
          importLogger.error(
            "Import of validated anime failed",
            new Error(result.error),
          );
          throw new Error(result.error);
        }
      }

      // Step 2: Add selected existing anime to the list
      if (hasSelectedExisting) {
        const existingIds = Array.from(selectedExisting);
        allAnimeIds = [...allAnimeIds, ...existingIds];
        importLogger.info("Adding existing anime to list", {
          count: existingIds.length,
        });
      }

      // Step 3: If we have a collection context, add all anime to the collection
      if (collectionId && allAnimeIds.length > 0) {
        importLogger.info("Adding anime to collection", {
          collectionId,
          count: allAnimeIds.length,
        });

        await addToCollection.mutateAsync({
          collectionId,
          animeIds: allAnimeIds,
        });
      }

      onAnimesImported?.(allAnimeIds);
      return allAnimeIds;
    } catch (error) {
      importLogger.error("Import failed", error);
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
