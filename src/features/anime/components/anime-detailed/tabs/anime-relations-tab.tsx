"use client";

import { useState } from "react";
import {
  Link,
  Heart,
  Sparkles,
  Users,
  Calendar,
  TrendingUp,
  Eye,
  Play,
  Film,
  Info,
  RefreshCw,
} from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import { useAnimeWithRelations } from "../hooks";
import {
  getRelationIcon,
  getRelationColor,
  getCategoryDisplayName,
  getCategoryDescription,
  getRelationDisplayName,
  sortRelations,
  getCategoryPriority,
} from "@/features/relations/utils";
import type { AnimeDetailed, AnimeWithRelationMetadata } from "@/types";

// Local interfaces for relation display
interface RelationLink {
  target_id: string;
  title: string;
  relation_type: string;
  provider: "jikan" | "anilist";
  category: string;
}

interface RelationMetadata {
  title_english: string | null;
  title_romaji: string | null;
  title_main: string;
  title_native: string | null;
  synopsis: string | null;
  anime_type: string;
  status: string;
  episodes: number | null;
  score: number | null;
  air_date_from: string | null;
  air_date_to: string | null;
  thumbnail_url: string | null;
  provider_id: string;
  provider: "jikan" | "anilist";
  relation_type: string;
  category: string;
}

interface AnimeRelationsTabProps {
  anime: AnimeDetailed;
}

function EmptyState({
  icon: Icon,
  title,
  description,
}: {
  icon: any;
  title: string;
  description: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center text-center space-y-4 py-12">
      <div className="p-4 rounded-full bg-muted/50">
        <Icon className="h-8 w-8 text-muted-foreground" />
      </div>
      <div className="space-y-2">
        <h3 className="font-semibold text-lg">{title}</h3>
        <p className="text-muted-foreground text-balance max-w-md">
          {description}
        </p>
      </div>
    </div>
  );
}

function UnifiedLoadingState({
  message,
  submessage,
  icon,
}: {
  message: string;
  submessage: string;
  icon: string;
}) {
  const getIcon = () => {
    switch (icon) {
      case "search":
        return <Link className="h-6 w-6" />;
      case "enhance":
        return <Sparkles className="h-6 w-6" />;
      default:
        return <RefreshCw className="h-6 w-6" />;
    }
  };

  const getIconColor = () => {
    switch (icon) {
      case "search":
        return "text-blue-600 dark:text-blue-400";
      case "enhance":
        return "text-purple-600 dark:text-purple-400";
      default:
        return "text-primary";
    }
  };

  const getBgColor = () => {
    switch (icon) {
      case "search":
        return "bg-blue-50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-800";
      case "enhance":
        return "bg-purple-50 dark:bg-purple-950/20 border-purple-200 dark:border-purple-800";
      default:
        return "border-dashed";
    }
  };

  return (
    <Card className={`${getBgColor()}`}>
      <CardContent className="flex flex-col items-center justify-center py-16">
        <div className="relative mb-6">
          <div className="absolute inset-0 animate-ping">
            <div
              className={`p-4 rounded-full bg-current opacity-20 ${getIconColor()}`}
            />
          </div>
          <div
            className={`relative p-4 rounded-full bg-background border-2 ${getIconColor()}`}
          >
            {getIcon()}
          </div>
        </div>

        <div className="space-y-2 text-center">
          <h3 className="font-semibold text-lg">{message}</h3>
          <p className="text-sm text-muted-foreground max-w-md">{submessage}</p>
        </div>
      </CardContent>
    </Card>
  );
}

function RelationCard({
  relation,
  metadata,
  colorClass,
  onAnimeClick,
}: {
  relation: RelationLink;
  metadata?: RelationMetadata;
  colorClass: string;
  onAnimeClick?: (animeId: string) => void;
}) {
  const handleClick = () => {
    if (onAnimeClick && relation.target_id) {
      onAnimeClick(relation.target_id);
    }
  };

  const getTypeIcon = (type: string) => {
    switch (type?.toLowerCase()) {
      case "tv":
      case "tv series":
        return <Play className="h-3 w-3" />;
      case "movie":
        return <Film className="h-3 w-3" />;
      case "ova":
      case "special":
        return <Calendar className="h-3 w-3" />;
      default:
        return <Calendar className="h-3 w-3" />;
    }
  };

  // Use rich metadata when available, fallback to basic relation data
  const displayTitle =
    metadata?.title_english ||
    metadata?.title_romaji ||
    metadata?.title_main ||
    relation.title ||
    "Unknown Title";
  const animeType = metadata?.anime_type || "TV";
  const status = metadata?.status;
  const episodes = metadata?.episodes;
  const score = metadata?.score;
  const airDate = metadata?.air_date_from;
  const thumbnailUrl = metadata?.thumbnail_url;

  const getStatusColor = (status: string) => {
    switch (status?.toLowerCase()) {
      case "finished airing":
      case "completed":
        return "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300";
      case "currently airing":
      case "ongoing":
        return "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300";
      default:
        return "bg-gray-100 text-gray-700 dark:bg-gray-900 dark:text-gray-300";
    }
  };

  return (
    <div
      className="group relative overflow-hidden border bg-card/50 backdrop-blur-sm hover:bg-card/80 hover:shadow-lg hover:shadow-primary/5 transition-all duration-300 cursor-pointer rounded-lg"
      onClick={handleClick}
    >
      <div className="flex gap-4 p-4">
        <div className="relative shrink-0">
          {thumbnailUrl ? (
            <img
              src={thumbnailUrl}
              alt={displayTitle}
              className="w-16 h-20 bg-muted rounded-lg object-cover"
              onError={(e) => {
                // Fallback to placeholder on image load error
                const target = e.target as HTMLImageElement;
                target.style.display = "none";
                target.nextElementSibling?.classList.remove("hidden");
              }}
            />
          ) : null}
          <div
            className={`w-16 h-20 bg-muted rounded-lg flex items-center justify-center ${thumbnailUrl ? "hidden" : ""}`}
          >
            <Film className="h-6 w-6 text-muted-foreground/50" />
          </div>
        </div>

        <div className="flex-1 min-w-0 space-y-3">
          <div className="space-y-2">
            <h3 className="font-semibold text-base leading-tight text-balance group-hover:text-primary transition-colors duration-200">
              {displayTitle}
            </h3>

            <div className="flex items-center gap-2 flex-wrap">
              <Badge
                variant="secondary"
                className={`text-xs font-medium ${colorClass} w-fit`}
              >
                {getRelationDisplayName(relation.relation_type)}
              </Badge>

              {status && (
                <Badge
                  variant="outline"
                  className={`text-xs ${getStatusColor(status)}`}
                >
                  {status}
                </Badge>
              )}
            </div>
          </div>

          <div className="flex items-center gap-4 text-sm text-muted-foreground">
            <div className="flex items-center gap-1.5">
              {getTypeIcon(animeType)}
              <span className="capitalize font-medium">{animeType}</span>
            </div>

            {episodes && (
              <div className="flex items-center gap-1">
                <span>
                  {episodes} ep{episodes !== 1 ? "s" : ""}
                </span>
              </div>
            )}

            {airDate && (
              <div className="flex items-center gap-1">
                <Calendar className="h-3 w-3" />
                <span>{new Date(airDate).getFullYear()}</span>
              </div>
            )}

            {score && (
              <div className="flex items-center gap-1">
                <span className="text-yellow-600 dark:text-yellow-400">★</span>
                <span>{score.toFixed(1)}</span>
              </div>
            )}
          </div>
        </div>

        <div className="absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
          <Eye className="h-4 w-4 text-primary" />
        </div>
      </div>
    </div>
  );
}

function RelationGroup({
  category,
  relations,
  detailedRelations,
  onAnimeClick,
}: {
  category: string;
  relations: RelationLink[];
  detailedRelations?: { relation: RelationLink; metadata: RelationMetadata }[];
  onAnimeClick?: (animeId: string) => void;
}) {
  const IconComponent = getRelationIcon(category);
  const colorClass = getRelationColor(category);

  const displayName = getCategoryDisplayName(category);
  const description = getCategoryDescription(category);
  const sortedRelations = sortRelations(relations);

  // Create a map of target_id -> metadata for quick lookup
  const metadataMap = new Map(
    detailedRelations?.map((dr) => [dr.relation.target_id, dr.metadata]) || [],
  );

  return (
    <section className="space-y-6">
      <div className="space-y-4">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-start gap-4">
            <div className={`p-3 rounded-xl border ${colorClass}`}>
              <IconComponent className="h-5 w-5" />
            </div>
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <h2 className="text-xl font-bold text-balance">
                  {displayName}
                </h2>
                {category === "mainStory" && (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="h-4 w-4 text-muted-foreground hover:text-foreground cursor-help transition-colors" />
                      </TooltipTrigger>
                      <TooltipContent side="top" className="max-w-xs">
                        <p className="text-sm">
                          Anime directly connected through the same franchise,
                          story timeline, or production universe. Items are
                          arranged in the recommended viewing order for optimal
                          story comprehension.
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                )}
              </div>
              <p className="text-sm text-muted-foreground leading-relaxed max-w-2xl">
                {description}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2 shrink-0">
            <Badge variant="secondary" className="text-sm font-medium">
              {relations.length} {relations.length === 1 ? "item" : "items"}
            </Badge>
            {category === "mainStory" && (
              <Badge className="text-xs bg-emerald-100 text-emerald-700 dark:bg-emerald-900 dark:text-emerald-300">
                Priority
              </Badge>
            )}
            {detailedRelations && (
              <Badge variant="outline" className="text-xs">
                Enhanced
              </Badge>
            )}
          </div>
        </div>

        <Separator className="opacity-60" />
      </div>

      <div className="grid gap-4">
        {sortedRelations.map((relation, index) => (
          <div key={relation.target_id} className="relative">
            {category === "mainStory" && index > 0 && (
              <div className="absolute -top-2 left-8 w-px h-4 bg-gradient-to-b from-emerald-300 to-transparent dark:from-emerald-700"></div>
            )}
            <RelationCard
              relation={relation}
              metadata={metadataMap.get(relation.target_id)}
              colorClass={colorClass}
              onAnimeClick={onAnimeClick}
            />
          </div>
        ))}
      </div>
    </section>
  );
}

function RecommendationPreviewCard({
  title,
  badge,
  description,
  icon: Icon,
}: {
  title: string;
  badge: string;
  description: string;
  icon: any;
}) {
  return (
    <Card className="border-dashed bg-muted/20 hover:bg-muted/30 transition-colors">
      <CardContent className="p-4">
        <div className="flex items-start gap-3">
          <div className="p-2 rounded-lg bg-primary/10">
            <Icon className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-2 flex-1">
            <div className="flex items-center gap-2">
              <h3 className="font-medium">{title}</h3>
              <Badge variant="outline" className="text-xs">
                {badge}
              </Badge>
            </div>
            <p className="text-sm text-muted-foreground leading-relaxed">
              {description}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

const recommendationPreviews = [
  {
    title: "Similar Themes",
    badge: "Genre Match",
    description:
      "Anime with similar storytelling themes and narrative elements.",
    icon: TrendingUp,
  },
  {
    title: "Same Studio",
    badge: "Studio",
    description: "Other acclaimed works from the same animation studio.",
    icon: Users,
  },
  {
    title: "Trending Now",
    badge: "Popular",
    description: "Currently popular among users with similar tastes.",
    icon: TrendingUp,
  },
  {
    title: "AI Curated",
    badge: "Smart",
    description: "Personalized picks based on advanced content analysis.",
    icon: Sparkles,
  },
];

export function AnimeRelationsTab({ anime }: AnimeRelationsTabProps) {
  const {
    relations,
    relationsByCategory,
    isLoading,
    error,
    hasRelations,
    refreshRelations,
  } = useAnimeWithRelations(anime.id);
  const [activeTab, setActiveTab] = useState("relations");

  const relationCount = relations.length;

  const sortedCategories = hasRelations
    ? Object.entries(relationsByCategory).sort(([a], [b]) => {
        return getCategoryPriority(a) - getCategoryPriority(b);
      })
    : [];

  // Simplified loading state
  const loadingStatus = {
    isLoading: isLoading,
    message: "Loading anime relations...",
    submessage: "Fetching related anime with complete metadata",
    icon: "search",
  };

  const handleAnimeClick = (animeId: string) => {
    // Navigate to anime detail page
    window.location.href = `/anime/${animeId}`;
  };

  const handleRefresh = () => {
    refreshRelations();
  };

  return (
    <div className="space-y-6">
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-2 h-12 p-1 bg-muted/50">
          <TabsTrigger
            value="relations"
            className="flex items-center gap-2 text-sm font-medium data-[state=active]:bg-background data-[state=active]:shadow-sm"
          >
            <Link className="h-4 w-4" />
            <span>Franchise Relations</span>
            {relationCount > 0 && (
              <Badge variant="secondary" className="ml-1 text-xs h-5">
                {relationCount}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger
            value="recommendations"
            className="flex items-center gap-2 text-sm font-medium data-[state=active]:bg-background data-[state=active]:shadow-sm"
          >
            <Heart className="h-4 w-4" />
            <span>Recommendations</span>
            <Badge variant="outline" className="ml-1 text-xs h-5">
              Soon
            </Badge>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="relations" className="space-y-6 mt-6">
          {/* Header with refresh action */}
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <h1 className="text-2xl font-bold">Relations</h1>
              <p className="text-sm text-muted-foreground">
                Franchise relations and detailed metadata
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={handleRefresh}
              disabled={loadingStatus.isLoading}
            >
              <RefreshCw
                className={`h-4 w-4 mr-2 ${loadingStatus.isLoading ? "animate-spin" : ""}`}
              />
              Refresh
            </Button>
          </div>

          <div className="space-y-8">
            {loadingStatus.isLoading && (
              <UnifiedLoadingState
                message={loadingStatus.message}
                submessage={loadingStatus.submessage}
                icon={loadingStatus.icon}
              />
            )}

            {error && (
              <Card className="border-destructive/20 bg-destructive/5">
                <CardContent className="py-16">
                  <EmptyState
                    icon={Link}
                    title="Unable to Load Relations"
                    description={
                      typeof error === "object" &&
                      error !== null &&
                      "message" in error
                        ? String(error.message)
                        : "We encountered an issue while discovering franchise relations. This anime may not have an AniList ID, or the provider service may be temporarily unavailable. Please try refreshing."
                    }
                  />
                  <div className="mt-6 flex justify-center">
                    <Button onClick={handleRefresh} variant="outline" size="sm">
                      <RefreshCw className="h-4 w-4 mr-2" />
                      Try Again
                    </Button>
                  </div>
                </CardContent>
              </Card>
            )}

            {!loadingStatus.isLoading && !error && !hasRelations && (
              <Card className="border-dashed bg-muted/20">
                <CardContent className="py-16">
                  <EmptyState
                    icon={Link}
                    title="No Relations Available"
                    description={`${anime.title.main} appears to be a standalone work with no direct sequels, prequels, or spin-offs currently in our database.`}
                  />
                </CardContent>
              </Card>
            )}

            {!loadingStatus.isLoading && hasRelations && (
              <div className="space-y-12">
                {sortedCategories.map(
                  ([category, animeWithRelations]: [
                    string,
                    AnimeWithRelationMetadata[],
                  ]) => {
                    // Convert AnimeWithRelationMetadata to RelationLink format for compatibility
                    const relations = animeWithRelations.map((item) => ({
                      target_id: item.anime.id,
                      title: item.anime.title.main,
                      relation_type: item.relation_type,
                      provider: "jikan" as const,
                      category: category,
                    }));

                    // Create detailed relations data
                    const detailedRelations = animeWithRelations.map(
                      (item) => ({
                        relation: {
                          target_id: item.anime.id,
                          title: item.anime.title.main,
                          relation_type: item.relation_type,
                          provider: "jikan" as const,
                          category: category,
                        },
                        metadata: {
                          title_english: item.anime.title.english,
                          title_romaji: item.anime.title.romaji,
                          title_main: item.anime.title.main,
                          title_native: item.anime.title.native,
                          synopsis: item.anime.synopsis,
                          anime_type: item.anime.animeType,
                          status: item.anime.status,
                          episodes: item.anime.episodes,
                          score: item.anime.score,
                          air_date_from: item.anime.aired?.from,
                          air_date_to: item.anime.aired?.to,
                          thumbnail_url: item.anime.imageUrl,
                          provider_id: item.anime.id,
                          provider: "jikan" as const,
                          relation_type: item.relation_type,
                          category: category,
                        },
                      }),
                    );

                    return (
                      <RelationGroup
                        key={category}
                        category={category}
                        relations={relations}
                        detailedRelations={detailedRelations}
                        onAnimeClick={handleAnimeClick}
                      />
                    );
                  },
                )}
              </div>
            )}
          </div>
        </TabsContent>

        <TabsContent value="recommendations" className="space-y-6 mt-6">
          <div className="space-y-4">
            <div className="flex items-start gap-4">
              <div className="p-3 rounded-xl bg-pink-50 dark:bg-pink-950/20 border border-pink-200 dark:border-pink-800">
                <Heart className="h-6 w-6 text-pink-600 dark:text-pink-400" />
              </div>
              <div className="space-y-2 flex-1">
                <h1 className="text-2xl font-bold text-balance">
                  Smart Recommendations
                </h1>
                <p className="text-muted-foreground text-balance leading-relaxed">
                  Personalized anime suggestions powered by advanced algorithms,
                  community insights, and your viewing preferences.
                </p>
              </div>
            </div>
            <Separator />
          </div>

          <Card className="border-dashed bg-gradient-to-br from-pink-50/50 to-purple-50/50 dark:from-pink-950/10 dark:to-purple-950/10">
            <CardContent className="py-16">
              <EmptyState
                icon={Heart}
                title="AI Recommendations Coming Soon"
                description={`Intelligent recommendations based on ${anime.title.main}'s unique characteristics and your personal viewing history will be available soon.`}
              />
            </CardContent>
          </Card>

          <div className="space-y-6">
            <div className="flex items-center gap-3">
              <Sparkles className="h-5 w-5 text-pink-600 dark:text-pink-400" />
              <h2 className="text-lg font-semibold">
                Planned Recommendation Features
              </h2>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              {recommendationPreviews.map((preview) => (
                <RecommendationPreviewCard key={preview.title} {...preview} />
              ))}
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
