import { useState, useEffect } from "react";
import { animeApi } from "@/features/anime/api";
import { animeLogger } from "@/lib/logger";

interface AutoEnrichmentResult {
  enrichmentPerformed: boolean;
  providersFound: string[];
  shouldReload: boolean;
}

/**
 * Hook for automatic background enrichment when anime loads
 *
 * This hook automatically runs enrichment in the background when an anime
 * detail page loads, silently finding missing provider data.
 */
export function useAutoEnrichment(animeId: string | undefined) {
  const [enrichmentResult, setEnrichmentResult] =
    useState<AutoEnrichmentResult | null>(null);
  const [isEnriching, setIsEnriching] = useState(false);

  useEffect(() => {
    if (!animeId) return;

    let isCancelled = false;

    const performAutoEnrichment = async () => {
      setIsEnriching(true);

      try {
        animeLogger.debug("Starting auto-enrichment", { animeId });
        const result = await animeApi.autoEnrichOnLoad(animeId);

        if (isCancelled) return;

        // invoke() automatically unwraps Result<T, E>, so we get AutoEnrichResult directly
        if (result.enrichmentPerformed) {
          animeLogger.success("Auto-enrichment completed", {
            animeId,
            providersFound: result.providersFound,
            shouldReload: result.shouldReload,
          });

          setEnrichmentResult(result);

          // If enrichment was successful and we should reload,
          // reload the page after a short delay
          if (result.shouldReload) {
            setTimeout(() => {
              if (!isCancelled) {
                animeLogger.info("Reloading page due to successful enrichment");
                window.location.reload();
              }
            }, 1500); // Give user time to see the enrichment happened
          }
        } else {
          animeLogger.debug("No missing data found for auto-enrichment", {
            animeId,
          });
        }
      } catch (error) {
        if (!isCancelled) {
          animeLogger.warn("Auto-enrichment failed", {
            animeId,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      } finally {
        if (!isCancelled) {
          setIsEnriching(false);
        }
      }
    };

    // Run auto-enrichment after a short delay to not block initial page load
    const timeoutId = setTimeout(performAutoEnrichment, 800);

    return () => {
      isCancelled = true;
      clearTimeout(timeoutId);
    };
  }, [animeId]);

  return {
    enrichmentResult,
    isEnriching,
    hasEnrichmentData: enrichmentResult?.enrichmentPerformed === true,
  };
}
