"use client";

import { useState, useEffect } from "react";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2 } from "lucide-react";
import { commands } from "@/types/bindings";
import type { AnimeProvider, ProviderStatus } from "@/types";

const providerInfo: Record<
  AnimeProvider,
  { name: string; description: string; features: string[] }
> = {
  jikan: {
    name: "Jikan (MyAnimeList)",
    description:
      "Official MyAnimeList API with comprehensive anime database, ratings, and community reviews",
    features: [
      "Comprehensive metadata",
      "User ratings",
      "Community reviews",
      "Popularity rankings",
    ],
  },
  anilist: {
    name: "AniList",
    description:
      "Modern anime tracking platform with GraphQL API and rich metadata",
    features: [
      "GraphQL API",
      "Rich metadata",
      "Modern tracking",
      "Social features",
    ],
  },
  kitsu: {
    name: "Kitsu",
    description:
      "Community-driven anime platform with social features and detailed information",
    features: [
      "Community-driven",
      "Social features",
      "Detailed info",
      "JSON API",
    ],
  },
  tmdb: {
    name: "TMDB",
    description:
      "The Movie Database - excellent for anime movies and international releases",
    features: [
      "Movie focused",
      "International releases",
      "Cast/crew info",
      "Images/posters",
    ],
  },
  anidb: {
    name: "AniDB",
    description:
      "Technical anime database with detailed episode information and file data",
    features: [
      "Technical details",
      "Episode info",
      "File data",
      "Comprehensive database",
    ],
  },
};

export function ProviderSelection() {
  const [providers, setProviders] = useState<ProviderStatus[]>([]);
  const [primaryProvider, setPrimaryProvider] = useState<AnimeProvider | null>(
    null,
  );
  const [loading, setLoading] = useState(true);
  const [changing, setChanging] = useState(false);

  const fetchProviders = async () => {
    try {
      const result = await commands.listProviders();
      if (result.status === "ok") {
        setProviders(result.data.providers);
        setPrimaryProvider(result.data.primary_provider);
      } else {
        console.error("Failed to load providers:", result.error);
      }
    } catch (error) {
      console.error("Error fetching providers:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchProviders();
  }, []);

  const handleProviderChange = async (provider: string) => {
    setChanging(true);
    try {
      const result = await commands.setPrimaryProvider({
        provider: provider as AnimeProvider,
      });
      if (result.status === "ok") {
        setPrimaryProvider(provider as AnimeProvider);
        await fetchProviders();
      } else {
        console.error("Failed to set primary provider:", result.error);
      }
    } catch (error) {
      console.error("Error setting primary provider:", error);
    } finally {
      setChanging(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2 p-4">
        <Loader2 className="h-4 w-4 animate-spin" />
        <span className="text-sm text-muted-foreground">
          Loading providers...
        </span>
      </div>
    );
  }

  const enabledProviders = providers.filter((p) => p.enabled);
  const selectedProvider = primaryProvider
    ? providerInfo[primaryProvider]
    : null;

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="provider-select">Primary Provider</Label>
        <Select
          value={primaryProvider || ""}
          onValueChange={handleProviderChange}
          disabled={changing}
        >
          <SelectTrigger id="provider-select" className="w-full max-w-xs">
            <SelectValue placeholder="Select provider" />
          </SelectTrigger>
          <SelectContent>
            {enabledProviders.map((providerStatus) => (
              <SelectItem
                key={providerStatus.provider}
                value={providerStatus.provider}
              >
                {providerInfo[providerStatus.provider].name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {selectedProvider && (
        <div className="space-y-3">
          <div className="text-sm text-muted-foreground max-w-md">
            {selectedProvider.description}
          </div>
          <div className="flex flex-wrap gap-2">
            {selectedProvider.features.map((feature) => (
              <span
                key={feature}
                className="inline-flex items-center px-2.5 py-1 rounded-full text-xs bg-muted text-muted-foreground"
              >
                {feature}
              </span>
            ))}
          </div>
        </div>
      )}

      {changing && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="h-3 w-3 animate-spin" />
          Updating provider...
        </div>
      )}
    </div>
  );
}
