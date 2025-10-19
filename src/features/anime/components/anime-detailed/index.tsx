"use client";

import { useState } from "react";
import {
  Calendar,
  Link,
  Play,
  ChevronDown,
  Check,
  Clock,
  Eye,
  X,
  Star,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

import type { AnimeDetailed } from "@/types/bindings";
import { getTierInfo, hasEnglishTitle } from "@/lib/anime-utils";
import { animeLogger, uiLogger } from "@/lib/logger";
import { AnimeOverviewTab, AnimeRelationsTab } from "./tabs";

// import { AnimeRatingModal } from "./anime-rating-modal"; // TODO: Create this component

interface AnimeDetailTabsProps {
  anime: AnimeDetailed;
  collectionId?: string;
  onAddToCollection?: () => void;
}

type WatchStatus =
  | "plan-to-watch"
  | "watching"
  | "completed"
  | "dropped"
  | null;

const watchStatusConfig = {
  "plan-to-watch": {
    label: "Plan to Watch",
    icon: Clock,
    color: "text-blue-500",
  },
  watching: { label: "Watching", icon: Eye, color: "text-green-500" },
  completed: { label: "Completed", icon: Check, color: "text-emerald-500" },
  dropped: { label: "Dropped", icon: X, color: "text-red-500" },
};

function WatchStatusDropdown({
  currentStatus,
  onStatusChange,
}: {
  currentStatus: WatchStatus;
  onStatusChange: (status: WatchStatus) => void;
}) {
  const currentConfig = currentStatus ? watchStatusConfig[currentStatus] : null;
  const CurrentIcon = currentConfig?.icon;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          className="transition-all duration-300 hover:scale-105 bg-transparent w-full sm:w-auto"
          size="sm"
        >
          {currentConfig && CurrentIcon ? (
            <>
              <CurrentIcon className={`h-4 w-4 mr-2 ${currentConfig.color}`} />
              {currentConfig.label}
            </>
          ) : (
            <>
              <Clock className="h-4 w-4 mr-2" />
              Add to List
            </>
          )}
          <ChevronDown className="h-4 w-4 ml-2" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-48">
        {Object.entries(watchStatusConfig).map(([status, config]) => {
          const Icon = config.icon;
          return (
            <DropdownMenuItem
              key={status}
              onClick={() => onStatusChange(status as WatchStatus)}
              className="flex items-center gap-2 cursor-pointer"
            >
              <Icon className={`h-4 w-4 ${config.color}`} />
              {config.label}
            </DropdownMenuItem>
          );
        })}
        {currentStatus && (
          <DropdownMenuItem
            onClick={() => onStatusChange(null)}
            className="flex items-center gap-2 cursor-pointer text-muted-foreground"
          >
            <X className="h-4 w-4" />
            Remove from List
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function QuickRateComponent({
  currentRating,
  animeId,
}: {
  currentRating: number;
  onRatingChange: (rating: number) => void;
  animeTitle: string;
  animeId: string;
}) {
  return (
    <>
      <Button
        variant="outline"
        className="transition-all duration-300 hover:scale-105 bg-transparent w-full sm:w-auto"
        size="sm"
        onClick={() =>
          uiLogger.debug("Rating modal not implemented yet", {
            animeId,
          })
        }
      >
        <Star
          className={`h-4 w-4 mr-2 ${currentRating > 0 ? "fill-yellow-400 text-yellow-400" : ""}`}
        />
        {currentRating > 0 ? `${currentRating}/10` : "Rate"}
      </Button>
    </>
  );
}

export function AnimeDetailTabs({
  anime,
  collectionId,
  onAddToCollection,
}: AnimeDetailTabsProps) {
  const [isAddingToCollection, setIsAddingToCollection] = useState(false);
  const [watchStatus, setWatchStatus] = useState<WatchStatus>(null);
  const [userRating, setUserRating] = useState(0);

  // Auto-enrichment runs silently in background - no UI needed

  const handleAddToCollection = async () => {
    if (collectionId) {
      setIsAddingToCollection(true);
      animeLogger.debug("Adding anime to collection", {
        collection_id: collectionId,
        anime_id: anime.id,
      });
      await new Promise((resolve) => setTimeout(resolve, 1000));
      setIsAddingToCollection(false);
      onAddToCollection?.();
    }
  };

  const handleWatchStatusChange = (newStatus: WatchStatus) => {
    setWatchStatus(newStatus);
    animeLogger.userAction("Watch status changed", {
      anime_id: anime.id,
      status: newStatus,
      component: "AnimeDetailTabs",
    });
  };

  const handleRatingChange = (rating: number) => {
    setUserRating(rating);
    animeLogger.userAction("User rating changed", {
      anime_id: anime.id,
      rating,
      component: "AnimeDetailTabs",
    });
  };

  return (
    <div className="w-full max-w-6xl mx-auto">
      <div className="relative bg-background">
        <div className="px-4 sm:px-6 py-4 sm:py-8">
          <div className="flex flex-col sm:flex-row gap-4 sm:gap-8 items-start">
            {anime.imageUrl && (
              <div className="relative group mx-auto sm:mx-0">
                <div className="aspect-[3/4] w-32 sm:w-40 lg:w-48 overflow-hidden rounded-lg shadow-lg transition-transform duration-300 group-hover:scale-105">
                  <img
                    src={anime.imageUrl || "/placeholder.svg"}
                    alt={anime.title.main}
                    className="object-cover w-full h-full"
                    loading="lazy"
                  />
                </div>
                <div className="absolute -top-2 -right-2 bg-primary text-primary-foreground px-2 py-1 rounded-md text-xs sm:text-sm font-semibold shadow-lg">
                  {anime.tier}
                </div>
                {watchStatus && (
                  <div className="absolute -bottom-2 -left-2 bg-background/90 backdrop-blur-sm border rounded-md px-2 py-1 shadow-lg">
                    <div className="flex items-center gap-1">
                      {(() => {
                        const config = watchStatusConfig[watchStatus];
                        const Icon = config.icon;
                        return <Icon className={`h-3 w-3 ${config.color}`} />;
                      })()}
                      <span className="text-xs font-medium">
                        {watchStatusConfig[watchStatus].label}
                      </span>
                    </div>
                  </div>
                )}
                {userRating > 0 && (
                  <div className="absolute -top-2 -left-2 bg-yellow-500 text-white px-2 py-1 rounded-md text-xs font-semibold shadow-lg flex items-center gap-1">
                    <Star className="h-3 w-3 fill-current" />
                    {userRating}
                  </div>
                )}
              </div>
            )}

            <div className="flex-1 space-y-4 sm:space-y-6 w-full">
              <div className="space-y-2 sm:space-y-3 text-center sm:text-left">
                <div className="flex flex-wrap gap-2 justify-center sm:justify-start">
                  <Badge
                    variant="secondary"
                    className="transition-colors hover:bg-secondary/80 text-xs"
                  >
                    {anime.animeType}
                  </Badge>
                  <Badge
                    variant="outline"
                    className="transition-colors hover:bg-accent text-xs"
                  >
                    {anime.status}
                  </Badge>
                  {anime.aired.from && (
                    <Badge
                      variant="outline"
                      className="transition-colors hover:bg-accent text-xs"
                    >
                      {new Date(anime.aired.from).getFullYear()}
                    </Badge>
                  )}
                </div>

                <div>
                  <h1 className="text-xl sm:text-2xl lg:text-4xl font-bold text-balance transition-colors duration-300">
                    {hasEnglishTitle(anime.title)
                      ? anime.title.english
                      : anime.title.main}
                  </h1>
                  {hasEnglishTitle(anime.title) && (
                    <p className="text-sm sm:text-base lg:text-lg text-muted-foreground mt-1">
                      {anime.title.main}
                    </p>
                  )}
                </div>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 sm:gap-4">
                <div className="text-center p-2 sm:p-3 bg-muted rounded-lg transition-all duration-300 hover:bg-muted/80 hover:scale-105 cursor-pointer">
                  <div className="text-lg sm:text-2xl font-bold text-primary">
                    {(anime.score || 0).toFixed(1)}
                  </div>
                  <div className="text-xs sm:text-sm text-muted-foreground">
                    Rating
                  </div>
                </div>
                <div className="text-center p-2 sm:p-3 bg-muted rounded-lg transition-all duration-300 hover:bg-muted/80 hover:scale-105 cursor-pointer">
                  <div className="text-lg sm:text-2xl font-bold text-primary">
                    {anime.episodes || "?"}
                  </div>
                  <div className="text-xs sm:text-sm text-muted-foreground">
                    Episodes
                  </div>
                </div>
                <div className="text-center p-2 sm:p-3 bg-muted rounded-lg transition-all duration-300 hover:bg-muted/80 hover:scale-105 cursor-pointer">
                  <div className="text-sm sm:text-2xl font-bold text-primary">
                    {getTierInfo(anime.tier).name}
                  </div>
                  <div className="text-xs sm:text-sm text-muted-foreground">
                    Quality
                  </div>
                </div>
                <div className="text-center p-2 sm:p-3 bg-muted rounded-lg transition-all duration-300 hover:bg-muted/80 hover:scale-105 cursor-pointer">
                  <div className="text-sm sm:text-lg font-bold text-primary">
                    {anime.status}
                  </div>
                  <div className="text-xs sm:text-sm text-muted-foreground">
                    Status
                  </div>
                </div>
              </div>

              <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
                <WatchStatusDropdown
                  currentStatus={watchStatus}
                  onStatusChange={handleWatchStatusChange}
                />
                <QuickRateComponent
                  currentRating={userRating}
                  onRatingChange={handleRatingChange}
                  animeTitle={
                    hasEnglishTitle(anime.title)
                      ? anime.title.english!
                      : anime.title.main
                  }
                  animeId={anime.id}
                />
                {collectionId && (
                  <Button
                    onClick={handleAddToCollection}
                    disabled={isAddingToCollection}
                    className="transition-all duration-300 hover:scale-105 w-full sm:w-auto"
                    size="sm"
                  >
                    {isAddingToCollection ? "Adding..." : "Add to Collection"}
                  </Button>
                )}
                <Button
                  variant="outline"
                  className="transition-all duration-300 hover:scale-105 bg-transparent w-full sm:w-auto"
                  size="sm"
                >
                  <Play className="h-4 w-4 mr-2" />
                  Watch Trailer
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="bg-background border-b">
        <div className="px-4 sm:px-6">
          <div className="border-t border-border/50"></div>
        </div>

        <div className="px-4 sm:px-6">
          <Tabs defaultValue="overview" className="w-full">
            <TabsList className="h-auto bg-transparent border-b rounded-none p-0 w-full justify-start overflow-x-auto">
              <div className="flex min-w-max">
                <TabsTrigger
                  value="overview"
                  className="flex items-center gap-2 px-3 sm:px-6 py-3 sm:py-4 rounded-none bg-transparent text-muted-foreground data-[state=active]:text-primary hover:text-foreground transition-all duration-200 text-sm whitespace-nowrap"
                >
                  <Calendar className="h-4 w-4" />
                  <span className="hidden sm:inline">Overview</span>
                </TabsTrigger>
                <TabsTrigger
                  value="related"
                  className="flex items-center gap-2 px-3 sm:px-6 py-3 sm:py-4 rounded-none bg-transparent text-muted-foreground data-[state=active]:text-primary hover:text-foreground transition-all duration-200 text-sm whitespace-nowrap"
                >
                  <Link className="h-4 w-4" />
                  <span className="hidden sm:inline">Related</span>
                </TabsTrigger>
              </div>
            </TabsList>

            <div className="py-4 sm:py-6">
              <TabsContent value="overview" className="mt-0">
                <AnimeOverviewTab anime={anime} />
              </TabsContent>

              <TabsContent value="related" className="mt-0">
                <AnimeRelationsTab anime={anime} />
              </TabsContent>
            </div>
          </Tabs>
        </div>

        <div className="px-4 sm:px-6">
          <div className="border-b border-border/50"></div>
        </div>
      </div>
    </div>
  );
}
