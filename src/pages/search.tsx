// src/pages/search.tsx
import { useState } from "react";
import { Search } from "lucide-react";
import { SearchInput } from "@/components/common/search-input";
import { AnimeGrid, AnimeDetailSheet } from "@/features/anime/components";
import { AnimeGridSkeleton } from "@/features/anime/components/anime-skeleton";
import { EmptyState } from "@/components/common/empty-state";
import { useAnimeSearch } from "@/features/anime/hooks";
import { useDebounce } from "@/hooks";
import type { Anime } from "@/types";

export function SearchPage() {
  const [query, setQuery] = useState("");
  const [selectedAnime, setSelectedAnime] = useState<Anime | null>(null);
  const debouncedQuery = useDebounce(query, 500);

  const { data: results = [], isLoading } = useAnimeSearch(debouncedQuery);

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Search Anime</h1>
        <p className="text-muted-foreground mb-6">
          Find anime from MyAnimeList database
        </p>

        <SearchInput
          value={query}
          onChange={setQuery}
          placeholder="Search for anime titles..."
          className="max-w-xl"
        />
      </div>

      <div>
        {isLoading ? (
          <AnimeGridSkeleton />
        ) : query && results.length === 0 ? (
          <EmptyState
            icon={<Search className="h-8 w-8 text-muted-foreground" />}
            title="No results found"
            description={`No anime found for "${query}"`}
          />
        ) : query ? (
          <AnimeGrid anime={results} onAnimeClick={setSelectedAnime} />
        ) : (
          <EmptyState
            icon={<Search className="h-8 w-8 text-muted-foreground" />}
            title="Start searching"
            description="Type an anime title to search"
          />
        )}
      </div>

      <AnimeDetailSheet
        anime={selectedAnime}
        isOpen={!!selectedAnime}
        onClose={() => setSelectedAnime(null)}
      />
    </div>
  );
}
