import type { AnimeDetailed, AnimeProvider } from "@/types/bindings";

export interface ProviderStatus {
  provider: AnimeProvider;
  hasId: boolean;
  hasUrl: boolean;
  id?: string;
  url?: string;
  isComplete: boolean;
}

export interface EnrichmentInfo {
  missingProviders: ProviderStatus[];
  availableProviders: ProviderStatus[];
  hasAnyMissing: boolean;
  canEnrich: boolean;
  enrichmentSuggestion: string;
}

// Actually implemented providers only
const PROVIDER_PRIORITY = {
  anilist: {
    priority: 1,
    name: "AniList",
    features: ["Relations", "GraphQL API"],
  },
  jikan: { priority: 2, name: "Jikan (MAL)", features: ["Ratings", "Reviews"] },
} as const;

/**
 * Analyze anime provider data completeness
 */
export function analyzeProviderData(anime: AnimeDetailed): EnrichmentInfo {
  const providers = Object.keys(
    PROVIDER_PRIORITY,
  ) as (keyof typeof PROVIDER_PRIORITY)[];
  const externalIds = anime.providerMetadata?.external_ids || {};
  const providerUrls = anime.providerMetadata?.provider_urls || {};

  const providerStatuses: ProviderStatus[] = providers.map((provider) => {
    const hasId = !!externalIds[provider];
    const hasUrl = !!providerUrls[provider];

    return {
      provider: provider as AnimeProvider,
      hasId,
      hasUrl,
      id: externalIds[provider],
      url: providerUrls[provider],
      isComplete: hasId, // Fixed: Only require ID (URL can be constructed)
    };
  });

  const missingProviders = providerStatuses.filter((p) => !p.isComplete);
  const availableProviders = providerStatuses.filter((p) => p.isComplete);

  const hasAnyMissing = missingProviders.length > 0;
  const canEnrich = availableProviders.length > 0; // Can use existing data to find missing

  // Generate enrichment suggestion
  let enrichmentSuggestion = "";
  if (hasAnyMissing && canEnrich) {
    const primaryProvider = availableProviders[0];
    const missingCount = missingProviders.length;
    const providerInfo =
      PROVIDER_PRIORITY[
        primaryProvider.provider as keyof typeof PROVIDER_PRIORITY
      ];
    enrichmentSuggestion = `Use ${providerInfo?.name || primaryProvider.provider} data to find ${missingCount} missing provider${missingCount > 1 ? "s" : ""}`;
  } else if (hasAnyMissing && !canEnrich) {
    enrichmentSuggestion =
      "Manual search required - no provider data available";
  }

  return {
    missingProviders,
    availableProviders,
    hasAnyMissing,
    canEnrich,
    enrichmentSuggestion,
  };
}

/**
 * Check if specific provider data is missing
 */
export function isMissingProvider(
  anime: AnimeDetailed,
  provider: AnimeProvider,
): boolean {
  const externalIds = anime.providerMetadata?.external_ids || {};
  return !externalIds[provider];
}

/**
 * Get missing critical providers (AniList, Jikan)
 */
export function getMissingCriticalProviders(
  anime: AnimeDetailed,
): AnimeProvider[] {
  const critical: AnimeProvider[] = ["anilist", "jikan"];
  return critical.filter((provider) => isMissingProvider(anime, provider));
}

/**
 * Generate enrichment priority list
 */
export function getEnrichmentPriority(anime: AnimeDetailed): ProviderStatus[] {
  const analysis = analyzeProviderData(anime);
  return analysis.missingProviders.sort((a, b) => {
    const priorityA =
      PROVIDER_PRIORITY[a.provider as keyof typeof PROVIDER_PRIORITY]
        ?.priority || 999;
    const priorityB =
      PROVIDER_PRIORITY[b.provider as keyof typeof PROVIDER_PRIORITY]
        ?.priority || 999;
    return priorityA - priorityB;
  });
}

/**
 * Check if anime has sufficient data for core features
 */
export function hasSufficientData(anime: AnimeDetailed): {
  hasBasicData: boolean;
  hasRelationsData: boolean;
  hasRatingsData: boolean;
  issues: string[];
} {
  const externalIds = anime.providerMetadata?.external_ids || {};
  const issues: string[] = [];

  const hasBasicData = Object.keys(externalIds).length > 0;
  const hasRelationsData = !!externalIds.anilist;
  const hasRatingsData = !!externalIds.jikan || !!externalIds.anilist;

  if (!hasBasicData) issues.push("No provider data available");
  if (!hasRelationsData) issues.push("Relations unavailable without AniList");
  if (!hasRatingsData) issues.push("Limited rating data without Jikan/AniList");

  return {
    hasBasicData,
    hasRelationsData,
    hasRatingsData,
    issues,
  };
}
