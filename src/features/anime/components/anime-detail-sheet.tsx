import { Star, Calendar, Play, Users, Trophy } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import type { Anime } from "@/types";
import { useAddAnimeToCollection } from "@/features/collection/hooks";
import { getTierInfo } from "@/lib/anime-utils";
import { hasEnglishTitle } from "@/lib/anime-utils";

interface AnimeDetailSheetProps {
  anime: Anime | null;
  isOpen: boolean;
  onClose: () => void;
  collectionId?: string;
}

export function AnimeDetailSheet({
  anime,
  isOpen,
  onClose,
  collectionId,
}: AnimeDetailSheetProps) {
  const addToCollection = useAddAnimeToCollection();

  if (!anime) return null;

  const year = anime.aired.from
    ? new Date(anime.aired.from).getFullYear()
    : null;

  const handleAddToCollection = async () => {
    if (collectionId) {
      await addToCollection.mutateAsync({
        collection_id: collectionId,
        anime_id: anime.id,
        user_score: null,
        notes: null,
      });
      onClose();
    }
  };

  return (
    <Sheet open={isOpen} onOpenChange={onClose}>
      <SheetContent className="w-full sm:max-w-xl overflow-y-auto p-4 rounded-sm">
        <SheetHeader className="pb-4">
          <SheetTitle className="text-2xl">{anime.title.main}</SheetTitle>
          {hasEnglishTitle(anime.title) && (
            <p className="text-muted-foreground">
              {anime.title.english || anime.title.main}
            </p>
          )}
        </SheetHeader>

        {anime.imageUrl && (
          <div className="relative aspect-[3/4] w-full max-w-[300px] mx-auto mb-6 overflow-hidden rounded-lg">
            <img
              src={anime.imageUrl}
              alt={anime.title.main}
              className="object-cover w-full h-full"
            />
            <Badge
              className="absolute top-2 right-2"
              style={{ backgroundColor: getTierInfo(anime.tier).color }}
            >
              {getTierInfo(anime.tier).name}
            </Badge>
          </div>
        )}

        <div className="space-y-6">
          {/* Stats Grid */}
          <div className="grid grid-cols-3 gap-3">
            <div className="text-center p-3 bg-muted rounded-lg">
              <Star className="h-5 w-5 mx-auto mb-1 text-primary" />
              <div className="font-semibold">
                {anime.compositeScore.toFixed(1)}
              </div>
              <div className="text-xs text-muted-foreground">Score</div>
            </div>
            <div className="text-center p-3 bg-muted rounded-lg">
              <Trophy className="h-5 w-5 mx-auto mb-1 text-primary" />
              <div className="font-semibold">#{anime.popularity ?? "N/A"}</div>
              <div className="text-xs text-muted-foreground">Popularity</div>
            </div>
            <div className="text-center p-3 bg-muted rounded-lg">
              <Users className="h-5 w-5 mx-auto mb-1 text-primary" />
              <div className="font-semibold">
                {anime.members
                  ? (anime.members / 1000).toFixed(0) + "k"
                  : "N/A"}
              </div>
              <div className="text-xs text-muted-foreground">Members</div>
            </div>
          </div>

          {/* Info */}
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{year ?? "TBA"}</span>
            </div>
            <div className="flex items-center gap-2">
              <Play className="h-4 w-4 text-muted-foreground" />
              <span className="text-sm">{anime.episodes ?? "?"} episodes</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Type:</span>
              <span className="text-sm">{anime.animeType}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Status:</span>
              <span className="text-sm capitalize">{anime.status}</span>
            </div>
          </div>

          {/* Genres */}
          <div>
            <h4 className="font-semibold mb-2">Genres</h4>
            <div className="flex flex-wrap gap-2">
              {anime.genres.map((genre) => (
                <Badge key={genre.id} variant="secondary">
                  {genre.name}
                </Badge>
              ))}
            </div>
          </div>

          <Separator />

          {/* Synopsis */}
          <div>
            <h4 className="font-semibold mb-2">Synopsis</h4>
            <p className="text-sm text-muted-foreground leading-relaxed">
              {anime.synopsis || "No synopsis available."}
            </p>
          </div>

          {/* Action Button */}
          {collectionId && (
            <Button
              className="w-full"
              onClick={handleAddToCollection}
              disabled={addToCollection.isPending}
            >
              Add to Collection
            </Button>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
