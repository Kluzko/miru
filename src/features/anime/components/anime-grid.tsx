import { AnimeCard } from "./anime-card";
import type { Anime } from "@/types";

interface AnimeGridProps {
  anime: Anime[];
  onAnimeClick?: (anime: Anime) => void;
  emptyMessage?: string;
}

export function AnimeGrid({
  anime,
  onAnimeClick,
  emptyMessage,
}: AnimeGridProps) {
  if (anime.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-muted-foreground">
          {emptyMessage || "No anime found"}
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {anime.map((item) => (
        <AnimeCard
          key={item.id}
          anime={item}
          onClick={() => onAnimeClick?.(item)}
        />
      ))}
    </div>
  );
}
