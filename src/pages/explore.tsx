// src/pages/search.tsx
import { useState } from "react";
import {
  Search,
  Filter,
  SlidersHorizontal,
  Star,
  Calendar,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { SearchInput } from "@/components/common/search-input";
import { AnimeGrid } from "@/features/anime/components";
import { AnimeGridSkeleton } from "@/features/anime/components/anime-skeleton";
import { EmptyState } from "@/components/common/empty-state";
import { useAnimeSearch } from "@/features/anime/hooks";
import { useDebounce } from "@/hooks";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { MultiSelect } from "@/components/common/multi-select";

import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
import type { AnimeDetailed } from "@/types";

export function ExplorePage() {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState({
    type: "any",
    status: "any",
    genres: [] as string[],
    yearRange: [1950, new Date().getFullYear()],
    score: [0],
    episodes: "any",
    rating: "any",
  });

  // Genre options for multi-select
  const genreOptions = [
    { label: "Action", value: "action" },
    { label: "Adventure", value: "adventure" },
    { label: "Comedy", value: "comedy" },
    { label: "Drama", value: "drama" },
    { label: "Fantasy", value: "fantasy" },
    { label: "Horror", value: "horror" },
    { label: "Mystery", value: "mystery" },
    { label: "Psychological", value: "psychological" },
    { label: "Romance", value: "romance" },
    { label: "Sci-Fi", value: "sci-fi" },
    { label: "Slice of Life", value: "slice-of-life" },
    { label: "Sports", value: "sports" },
    { label: "Supernatural", value: "supernatural" },
    { label: "Thriller", value: "thriller" },
    { label: "Music", value: "music" },
    { label: "School", value: "school" },
    { label: "Military", value: "military" },
    { label: "Historical", value: "historical" },
  ];
  const debouncedQuery = useDebounce(query, 500);

  const { data: results = [], isLoading } = useAnimeSearch(debouncedQuery);

  const handleAnimeClick = (anime: AnimeDetailed) => {
    navigate(`/anime/${anime.id}`);
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Explore Anime</h1>
        <p className="text-muted-foreground mb-6">
          Discover anime from MyAnimeList database with advanced filters
        </p>

        <div className="flex gap-4 items-end">
          <SearchInput
            value={query}
            onChange={setQuery}
            placeholder="Search for anime titles..."
            className="max-w-xl"
          />
          <Button
            variant="outline"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center gap-2"
          >
            <SlidersHorizontal className="h-4 w-4" />
            Advanced Filters
            {showFilters && (
              <Badge variant="secondary" className="ml-2">
                ON
              </Badge>
            )}
          </Button>
        </div>
      </div>

      {/* Advanced Filters */}
      <Collapsible open={showFilters} onOpenChange={setShowFilters}>
        <CollapsibleContent className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Filter className="h-5 w-5" />
                Advanced Search Filters
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {/* Type Filter */}
                <div className="space-y-2">
                  <label className="text-sm font-medium">Type</label>
                  <Select
                    value={filters.type}
                    onValueChange={(value) =>
                      setFilters((prev) => ({ ...prev, type: value }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Any type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="any">Any type</SelectItem>
                      <SelectItem value="tv">TV Series</SelectItem>
                      <SelectItem value="movie">Movie</SelectItem>
                      <SelectItem value="ova">OVA</SelectItem>
                      <SelectItem value="special">Special</SelectItem>
                      <SelectItem value="ona">ONA</SelectItem>
                      <SelectItem value="music">Music</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Status Filter */}
                <div className="space-y-2">
                  <label className="text-sm font-medium">Status</label>
                  <Select
                    value={filters.status}
                    onValueChange={(value) =>
                      setFilters((prev) => ({ ...prev, status: value }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Any status" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="any">Any status</SelectItem>
                      <SelectItem value="airing">Currently Airing</SelectItem>
                      <SelectItem value="complete">Finished Airing</SelectItem>
                      <SelectItem value="upcoming">Not Yet Aired</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Genre Filter - Multi-select */}
                <div className="space-y-2 md:col-span-2">
                  <label className="text-sm font-medium">Genres</label>
                  <MultiSelect
                    options={genreOptions}
                    onValueChange={(values) =>
                      setFilters((prev) => ({ ...prev, genres: values }))
                    }
                    defaultValue={filters.genres}
                    placeholder="Select genres..."
                    variant="default"
                    animation={0.1}
                    maxCount={3}
                  />
                </div>

                {/* Episodes Filter */}
                <div className="space-y-2">
                  <label className="text-sm font-medium">Episodes</label>
                  <Select
                    value={filters.episodes}
                    onValueChange={(value) =>
                      setFilters((prev) => ({ ...prev, episodes: value }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Any length" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="any">Any length</SelectItem>
                      <SelectItem value="short">
                        Short (1-12 episodes)
                      </SelectItem>
                      <SelectItem value="medium">
                        Medium (13-25 episodes)
                      </SelectItem>
                      <SelectItem value="long">Long (26+ episodes)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Rating Filter */}
                <div className="space-y-2">
                  <label className="text-sm font-medium">Rating</label>
                  <Select
                    value={filters.rating}
                    onValueChange={(value) =>
                      setFilters((prev) => ({ ...prev, rating: value }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Any rating" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="any">Any rating</SelectItem>
                      <SelectItem value="g">G - All Ages</SelectItem>
                      <SelectItem value="pg">PG - Children</SelectItem>
                      <SelectItem value="pg13">PG-13 - Teens 13+</SelectItem>
                      <SelectItem value="r">R - 17+</SelectItem>
                      <SelectItem value="rx">Rx - Hentai</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {/* Year Range Filter */}
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <Calendar className="h-4 w-4" />
                  <label className="text-sm font-medium">
                    Year Range: {filters.yearRange[0]} - {filters.yearRange[1]}
                  </label>
                </div>
                <Slider
                  value={filters.yearRange}
                  onValueChange={(value) =>
                    setFilters((prev) => ({ ...prev, yearRange: value }))
                  }
                  max={new Date().getFullYear()}
                  min={1950}
                  step={1}
                  className="w-full"
                />
              </div>

              {/* Score Filter */}
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <Star className="h-4 w-4" />
                  <label className="text-sm font-medium">
                    Minimum Score: {filters.score[0]}
                  </label>
                </div>
                <Slider
                  value={filters.score}
                  onValueChange={(value) =>
                    setFilters((prev) => ({ ...prev, score: value }))
                  }
                  max={10}
                  min={0}
                  step={0.1}
                  className="w-full"
                />
              </div>

              {/* Filter Actions */}
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  onClick={() =>
                    setFilters({
                      type: "any",
                      status: "any",
                      genres: [],
                      yearRange: [1950, new Date().getFullYear()],
                      score: [0],
                      episodes: "any",
                      rating: "any",
                    })
                  }
                  size="sm"
                >
                  Clear Filters
                </Button>
                <Button size="sm">Apply Filters</Button>
              </div>
            </CardContent>
          </Card>
        </CollapsibleContent>
      </Collapsible>

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
          <AnimeGrid anime={results} onAnimeClick={handleAnimeClick} />
        ) : (
          <EmptyState
            icon={<Search className="h-8 w-8 text-muted-foreground" />}
            title="Start searching"
            description="Type an anime title to explore or use advanced filters"
          />
        )}
      </div>
    </div>
  );
}
