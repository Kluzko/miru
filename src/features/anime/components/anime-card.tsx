import { Star, Calendar, Play } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { Anime } from "@/types";
import { cn } from "@/lib/utils";
import { getTierInfo } from "@/lib/anime-utils";

interface AnimeCardProps {
  anime: Anime;
  onClick?: () => void;
  className?: string;
}

export function AnimeCard({ anime, onClick, className }: AnimeCardProps) {
  const year = anime.aired.from
    ? new Date(anime.aired.from).getFullYear()
    : null;

  return (
    <Card
      className={cn(
        "group cursor-pointer transition-all hover:shadow-lg",
        className,
      )}
      onClick={onClick}
    >
      {anime.imageUrl && (
        <div className="aspect-[3/4] relative overflow-hidden">
          <img
            src={anime.imageUrl}
            alt={anime.title.main}
            className="object-cover w-full h-full group-hover:scale-105 transition-transform"
          />
          <Badge
            className="absolute top-2 right-2"
            style={{ backgroundColor: getTierInfo(anime.tier).color }}
          >
            {getTierInfo(anime.tier).name}
          </Badge>
        </div>
      )}
      <CardContent className="p-4">
        <h3 className="font-semibold line-clamp-1">
          {anime.title.main}
        </h3>

        <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
          <div className="flex items-center gap-1">
            <Star className="h-3 w-3" />
            {anime.compositeScore.toFixed(1)}
          </div>
          <div className="flex items-center gap-1">
            <Calendar className="h-3 w-3" />
            {year ?? "TBA"}
          </div>
          <div className="flex items-center gap-1">
            <Play className="h-3 w-3" />
            {anime.episodes ?? "?"}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
