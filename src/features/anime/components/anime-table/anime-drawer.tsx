"use client";

import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Star,
  Calendar,
  Play,
  TrendingUp,
  Trash2,
  ExternalLink,
  Users,
  Heart,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { Anime } from "@/types";

interface AnimeDrawerProps {
  anime: Anime | null;
  isOpen: boolean;
  onClose: () => void;
  onRemove?: (animeId: string) => void;
  onViewDetails?: (anime: Anime) => void;
}

export function AnimeDrawer({
  anime,
  isOpen,
  onClose,
  onRemove,
  onViewDetails,
}: AnimeDrawerProps) {
  if (!anime) return null;

  const getStatusColor = (status: string) => {
    switch (status) {
      case "airing":
        return "bg-green-50 text-green-700 border-green-200 dark:bg-green-950 dark:text-green-300";
      case "finished":
        return "bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950 dark:text-blue-300";
      case "not_yet_aired":
        return "bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950 dark:text-orange-300";
      default:
        return "bg-muted text-muted-foreground border-border";
    }
  };

  const getTypeColor = (type: string) => {
    switch (type) {
      case "TV":
        return "bg-purple-50 text-purple-700 border-purple-200 dark:bg-purple-950 dark:text-purple-300";
      case "Movie":
        return "bg-red-50 text-red-700 border-red-200 dark:bg-red-950 dark:text-red-300";
      case "OVA":
        return "bg-indigo-50 text-indigo-700 border-indigo-200 dark:bg-indigo-950 dark:text-indigo-300";
      case "Special":
        return "bg-pink-50 text-pink-700 border-pink-200 dark:bg-pink-950 dark:text-pink-300";
      default:
        return "bg-muted text-muted-foreground border-border";
    }
  };

  const getRatingStars = (score: number | null) => {
    const validScore = Math.max(0, Math.min(10, score || 0));
    const fullStars = Math.floor(validScore / 2);
    const hasHalfStar = validScore % 2 >= 1;
    const emptyStars = Math.max(0, 5 - fullStars - (hasHalfStar ? 1 : 0));

    return (
      <div className="flex items-center gap-1">
        {Array.from({ length: fullStars }, (_, i) => (
          <Star key={i} className="h-4 w-4 fill-primary text-primary" />
        ))}
        {hasHalfStar && (
          <Star className="h-4 w-4 fill-primary/50 text-primary" />
        )}
        {Array.from({ length: emptyStars }, (_, i) => (
          <Star
            key={i + fullStars}
            className="h-4 w-4 text-muted-foreground/30"
          />
        ))}
        <span className="ml-2 text-sm font-semibold text-foreground">
          {validScore.toFixed(1)}/10
        </span>
      </div>
    );
  };

  const formatYear = (dateString: string | null) => {
    if (!dateString) return "Unknown";
    return new Date(dateString).getFullYear().toString();
  };

  const handleRemove = () => {
    if (onRemove && anime) {
      onRemove(anime.id);
      onClose(); // Close drawer after removing
    }
  };

  const handleViewDetails = () => {
    if (onViewDetails && anime) {
      onViewDetails(anime);
    } else if (anime) {
      // Navigate to anime detail page
      window.location.href = `/anime/${anime.id}`;
    }
  };

  return (
    <Sheet open={isOpen} onOpenChange={onClose}>
      <SheetContent
        side="right"
        className="w-full sm:w-[400px] sm:max-w-[400px] overflow-y-auto bg-background/95 backdrop-blur-sm p-6"
      >
        <SheetHeader className="pb-4 border-b border-border/30">
          <div className="space-y-3">
            <SheetTitle className="text-xl font-bold leading-tight text-balance">
              {anime.title}
            </SheetTitle>

            {/* English/Japanese titles */}
            {(anime.titleEnglish || anime.titleJapanese) && (
              <div className="space-y-1">
                {anime.titleEnglish && anime.titleEnglish !== anime.title && (
                  <p className="text-sm text-muted-foreground">
                    English: {anime.titleEnglish}
                  </p>
                )}
                {anime.titleJapanese && (
                  <p className="text-sm text-muted-foreground">
                    Japanese: {anime.titleJapanese}
                  </p>
                )}
              </div>
            )}

            <div className="flex items-center gap-2 flex-wrap">
              <Badge
                variant="outline"
                className={cn(
                  "font-medium px-2 py-1 text-xs rounded-full",
                  getStatusColor(anime.status),
                )}
              >
                {anime.status.replace("_", " ")}
              </Badge>
              <Badge
                variant="outline"
                className={cn(
                  "font-medium px-2 py-1 text-xs rounded-full",
                  getTypeColor(anime.animeType),
                )}
              >
                {anime.animeType}
              </Badge>
              <div className="flex items-center gap-1 text-xs text-muted-foreground bg-muted/40 px-2 py-1 rounded-full border border-border/30">
                <Calendar className="h-3 w-3" />
                {formatYear(anime.aired.from)}
              </div>
            </div>
          </div>
        </SheetHeader>

        <div className="space-y-4 py-4">
          {/* Stats Grid */}
          <div className="grid grid-cols-2 gap-2">
            <div className="text-center p-3 bg-card rounded-lg border border-border/40">
              <Play className="h-4 w-4 text-primary mx-auto mb-1" />
              <div className="text-xs text-muted-foreground mb-1">Episodes</div>
              <div className="font-semibold text-sm">
                {anime.episodes || "?"}
              </div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg border border-border/40">
              <TrendingUp className="h-4 w-4 text-primary mx-auto mb-1" />
              <div className="text-xs text-muted-foreground mb-1">Rank</div>
              <div className="font-semibold text-sm">#{anime.rank || "?"}</div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg border border-border/40">
              <Users className="h-4 w-4 text-primary mx-auto mb-1" />
              <div className="text-xs text-muted-foreground mb-1">Members</div>
              <div className="font-semibold text-sm">
                {anime.members?.toLocaleString() || "?"}
              </div>
            </div>
            <div className="text-center p-3 bg-card rounded-lg border border-border/40">
              <Heart className="h-4 w-4 text-primary mx-auto mb-1" />
              <div className="text-xs text-muted-foreground mb-1">
                Favorites
              </div>
              <div className="font-semibold text-sm">
                {anime.favorites?.toLocaleString() || "?"}
              </div>
            </div>
          </div>

          {/* Rating */}
          <div className="space-y-2">
            <h3 className="font-semibold text-sm">Rating</h3>
            <div className="p-3 bg-muted/40 rounded-lg border border-border/30">
              {getRatingStars(anime.score)}
              {anime.scoredBy && (
                <div className="text-xs text-muted-foreground mt-2">
                  Scored by {anime.scoredBy.toLocaleString()} users
                </div>
              )}
            </div>
          </div>

          {/* Studios */}
          {anime.studios.length > 0 && (
            <div className="space-y-2">
              <h3 className="font-semibold text-sm">Studios</h3>
              <div className="flex flex-wrap gap-1">
                {anime.studios.map((studio) => (
                  <Badge
                    key={studio}
                    variant="secondary"
                    className="px-2 py-1 text-xs rounded-full bg-muted/60 border border-border/30"
                  >
                    {studio}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Genres */}
          <div className="space-y-2">
            <h3 className="font-semibold text-sm">Genres</h3>
            <div className="flex flex-wrap gap-1">
              {anime.genres.slice(0, 6).map((genre) => (
                <Badge
                  key={genre.id}
                  variant="secondary"
                  className="px-2 py-1 text-xs rounded-full bg-muted/60 border border-border/30"
                >
                  {genre.name}
                </Badge>
              ))}
              {anime.genres.length > 6 && (
                <Badge
                  variant="secondary"
                  className="px-2 py-1 text-xs rounded-full bg-muted/60 border border-border/30"
                >
                  +{anime.genres.length - 6}
                </Badge>
              )}
            </div>
          </div>

          {/* Synopsis */}
          {anime.synopsis && (
            <div className="space-y-2">
              <h3 className="font-semibold text-sm">Synopsis</h3>
              <div className="p-3 bg-muted/40 rounded-lg border border-border/30">
                <p className="text-muted-foreground text-xs leading-relaxed line-clamp-4">
                  {anime.synopsis}
                </p>
              </div>
            </div>
          )}

          {/* Additional Info */}
          <div className="space-y-2">
            <h3 className="font-semibold text-sm">Additional Info</h3>
            <div className="grid grid-cols-2 gap-2 text-xs">
              {anime.source && (
                <div className="p-2 bg-muted/40 rounded border border-border/30">
                  <span className="text-muted-foreground">Source:</span>
                  <div className="font-medium">{anime.source}</div>
                </div>
              )}
              {anime.duration && (
                <div className="p-2 bg-muted/40 rounded border border-border/30">
                  <span className="text-muted-foreground">Duration:</span>
                  <div className="font-medium">{anime.duration}</div>
                </div>
              )}
              {anime.rating && (
                <div className="p-2 bg-muted/40 rounded border border-border/30">
                  <span className="text-muted-foreground">Rating:</span>
                  <div className="font-medium">{anime.rating}</div>
                </div>
              )}
              <div className="p-2 bg-muted/40 rounded border border-border/30">
                <span className="text-muted-foreground">Popularity:</span>
                <div className="font-medium">#{anime.popularity || "?"}</div>
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="space-y-2 mt-6 pt-4 border-t border-border/30">
            <Button
              onClick={handleViewDetails}
              className="w-full gap-2"
              variant="outline"
            >
              <ExternalLink className="h-4 w-4" />
              View Full Details
            </Button>

            {onRemove && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleRemove}
                className="w-full gap-2 text-destructive hover:text-destructive hover:bg-destructive/10 border-destructive/20 bg-transparent"
              >
                <Trash2 className="h-4 w-4" />
                Remove from Collection
              </Button>
            )}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
