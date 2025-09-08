"use client";

import { useState, useRef, useEffect } from "react";
import type { Anime } from "@/types";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Star,
  Search,
  ArrowUpDown,
  Calendar,
  Play,
  Grid3X3,
  List,
  ChevronRight,
  Clock,
  TrendingUp,
  Filter,
  X,
  ChevronDown,
  Settings2,
} from "lucide-react";
import { cn } from "@/lib/utils";

import { useAnimeSearch } from "./use-anime-search";
import { useAnimeFilters } from "./use-anime-filters";
import { useAnimeProcessing } from "./use-anime-processing";
import { AnimeDrawer } from "./anime-drawer";

interface AnimeTableProps {
  animes: Anime[];
  onAnimeClick?: (anime: Anime) => void;
  onRemoveAnime?: (animeId: string) => void;
  selectedAnimes?: Set<string>;
  onSelectionChange?: (selected: Set<string>) => void;
}

export function AnimeTable({
  animes,
  onAnimeClick,
  onRemoveAnime,
}: AnimeTableProps) {
  const [viewMode, setViewMode] = useState<"detailed" | "compact">("detailed");
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());
  const [filtersExpanded, setFiltersExpanded] = useState(false);
  const [sortingExpanded, setSortingExpanded] = useState(false);
  const [selectedAnime, setSelectedAnime] = useState<Anime | null>(null);
  const [drawerOpen, setDrawerOpen] = useState(false);

  const searchInputRef = useRef<HTMLInputElement>(null);

  const {
    searchTerm,
    searchSuggestions,
    showSuggestions,
    recentSearches,
    handleSearch,
    setShowSuggestions,
  } = useAnimeSearch(animes);

  const {
    genreFilter,
    setGenreFilter,
    yearFilter,
    setYearFilter,
    statusFilter,
    setStatusFilter,
    typeFilter,
    sortBy,
    setSortBy,
    sortOrder,
    setSortOrder,
    groupBy,
    setGroupBy,
    activeFilters,
    addQuickFilter,
    removeFilter,
    clearAllFilters,
    uniqueGenres,
    uniqueYears,
    uniqueStatuses,
  } = useAnimeFilters(animes);

  const { filteredAndSortedAnimes, groupedAnimes } = useAnimeProcessing(
    animes,
    {
      searchTerm,
      genreFilter,
      yearFilter,
      statusFilter,
      typeFilter,
      sortBy,
      sortOrder,
      groupBy,
    },
  );

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
          case "f":
            e.preventDefault();
            searchInputRef.current?.focus();
            break;
          case "k":
            e.preventDefault();
            searchInputRef.current?.focus();
            break;
        }
      }
      if (e.key === "Escape") {
        setShowSuggestions(false);
        searchInputRef.current?.blur();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [setShowSuggestions]);

  const toggleGroup = (groupKey: string) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(groupKey)) {
      newExpanded.delete(groupKey);
    } else {
      newExpanded.add(groupKey);
    }
    setExpandedGroups(newExpanded);
  };

  const handleAnimeClick = (anime: Anime) => {
    setSelectedAnime(anime);
    setDrawerOpen(true);
    onAnimeClick?.(anime);
  };

  const formatYear = (dateString: string | null) => {
    if (!dateString) return "Unknown";
    return new Date(dateString).getFullYear().toString();
  };

  const renderAnimeItem = (anime: Anime, index: number) => {
    if (viewMode === "compact") {
      return (
        <div
          key={anime.id}
          className="group flex items-center gap-4 bg-card hover:bg-accent/50 border border-border hover:border-primary/20 rounded-lg transition-all duration-200 cursor-pointer p-4"
          onClick={() => handleAnimeClick(anime)}
        >
          <div className="w-8 h-8 bg-muted rounded-full flex items-center justify-center text-xs font-semibold">
            {index + 1}
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-foreground group-hover:text-primary transition-colors truncate">
              {anime.title}
            </h3>
            <div className="flex items-center gap-2 mt-1 text-xs text-muted-foreground">
              <span>{anime.genres[0]?.name || "Unknown"}</span>
              <span>•</span>
              <span>{anime.episodes || "?"} eps</span>
              {anime.studios[0] && (
                <>
                  <span>•</span>
                  <span>{anime.studios[0]}</span>
                </>
              )}
            </div>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-sm text-muted-foreground">
              {formatYear(anime.aired.from)}
            </span>
            <Badge variant="outline" className="text-xs">
              {anime.status.replace("_", " ")}
            </Badge>
            <div className="flex items-center gap-1">
              <Star className="h-3 w-3 fill-primary text-primary" />
              <span className="text-sm font-medium">
                {(anime.score || 0).toFixed(1)}
              </span>
            </div>
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <TrendingUp className="h-3 w-3" />#{anime.rank || "?"}
            </div>
          </div>
        </div>
      );
    }

    return (
      <div
        key={anime.id}
        className="group relative bg-card hover:bg-accent/50 border border-border hover:border-primary/20 rounded-xl transition-all duration-200 cursor-pointer p-6"
        onClick={() => handleAnimeClick(anime)}
      >
        <div className="flex items-start gap-6">
          <div className="flex-shrink-0 w-10 h-10 bg-muted rounded-full flex items-center justify-center text-sm font-semibold text-muted-foreground">
            {index + 1}
          </div>
          <div className="flex-1 min-w-0 space-y-3">
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <h3 className="font-semibold text-lg text-foreground group-hover:text-primary transition-colors truncate mb-1">
                  {anime.title}
                </h3>
                <p className="text-sm text-muted-foreground line-clamp-2 leading-relaxed">
                  {anime.synopsis}
                </p>
                {anime.studios[0] && (
                  <p className="text-xs text-muted-foreground mt-1">
                    Studio: {anime.studios[0]}
                  </p>
                )}
              </div>
              <div className="flex-shrink-0 flex items-center gap-2 bg-primary/10 px-4 py-2 rounded-lg">
                <Star className="h-4 w-4 fill-primary text-primary" />
                <span className="font-semibold text-foreground">
                  {(anime.score || 0).toFixed(1)}
                </span>
                <span className="text-muted-foreground text-sm">/10</span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                  <Calendar className="h-4 w-4" />
                  {formatYear(anime.aired.from)}
                </div>
                <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                  <Play className="h-4 w-4" />
                  {anime.episodes || "?"} eps
                </div>
                <Badge variant="outline" className="text-xs font-medium">
                  {anime.animeType}
                </Badge>
              </div>
              <div className="flex items-center gap-3">
                <div className="flex gap-1.5">
                  {anime.genres.slice(0, 2).map((genre) => (
                    <button
                      key={genre.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        addQuickFilter("genre", genre.name);
                      }}
                      className="text-xs bg-muted hover:bg-muted/80 px-2.5 py-1 rounded-full font-medium transition-colors"
                    >
                      {genre.name}
                    </button>
                  ))}
                  {anime.genres.length > 2 && (
                    <span className="text-xs bg-muted px-2.5 py-1 rounded-full font-medium">
                      +{anime.genres.length - 2}
                    </span>
                  )}
                </div>
                <div className="text-right bg-muted/50 px-3 py-1.5 rounded-lg">
                  <div className="text-xs text-muted-foreground">Rank</div>
                  <div className="text-sm font-medium">
                    #{anime.rank || "?"}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  };

  return (
    <>
      <div className="h-full">
        {/* Header with Search and Controls - Sticky */}
        <div className="sticky top-0 z-40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border pb-4">
          {/* Main Search Bar */}
          <div className="bg-card rounded-lg border border-border p-4">
            <div className="flex items-center gap-3">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  ref={searchInputRef}
                  placeholder="Search anime, genres, studios... (Ctrl+K)"
                  value={searchTerm}
                  onChange={(e) => handleSearch(e.target.value)}
                  onFocus={() =>
                    setShowSuggestions(searchSuggestions.length > 0)
                  }
                  onBlur={() =>
                    setTimeout(() => setShowSuggestions(false), 200)
                  }
                  className="pl-10 h-10 text-base"
                />
                {/* Search Suggestions */}
                {showSuggestions && (
                  <div className="absolute top-full left-0 right-0 mt-1 bg-popover border border-border rounded-lg shadow-lg z-50">
                    <div className="p-2 space-y-1">
                      {searchSuggestions.map((suggestion, index) => (
                        <button
                          key={index}
                          onClick={() => {
                            handleSearch(suggestion);
                            setShowSuggestions(false);
                          }}
                          className="w-full text-left px-3 py-2 text-sm hover:bg-accent rounded-md transition-colors"
                        >
                          {suggestion}
                        </button>
                      ))}
                    </div>
                    {recentSearches.length > 0 && (
                      <div className="border-t border-border p-2">
                        <div className="text-xs text-muted-foreground mb-2 px-3">
                          Recent searches
                        </div>
                        {recentSearches.slice(0, 3).map((search, index) => (
                          <button
                            key={index}
                            onClick={() => {
                              handleSearch(search);
                              setShowSuggestions(false);
                            }}
                            className="w-full text-left px-3 py-1 text-xs text-muted-foreground hover:bg-accent rounded-md transition-colors flex items-center gap-2"
                          >
                            <Clock className="h-3 w-3" />
                            {search}
                          </button>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>

              <div className="flex items-center gap-2">
                <div
                  className={cn(
                    "text-sm font-medium px-2 py-1 rounded-md",
                    filteredAndSortedAnimes.length < animes.length
                      ? "bg-blue-50 text-blue-700 border border-blue-200"
                      : "text-muted-foreground",
                  )}
                >
                  {filteredAndSortedAnimes.length}
                  {filteredAndSortedAnimes.length < animes.length && (
                    <span className="text-blue-600"> of {animes.length}</span>
                  )}{" "}
                  results
                  {filteredAndSortedAnimes.length < animes.length && (
                    <span className="ml-1 text-xs">(filtered)</span>
                  )}
                </div>

                <div className="flex items-center gap-1">
                  {/* Reset All Button - Only show when filters/sorting are active */}
                  {(genreFilter !== "all" ||
                    yearFilter !== "all" ||
                    statusFilter !== "all" ||
                    typeFilter !== "all" ||
                    sortBy !== "rating" ||
                    sortOrder !== "desc" ||
                    groupBy !== "none" ||
                    searchTerm) && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => {
                        clearAllFilters();
                        handleSearch("");
                        setSortBy("rating");
                        setSortOrder("desc");
                        setGroupBy("none");
                      }}
                      className="gap-1 text-muted-foreground hover:text-foreground"
                    >
                      <X className="h-3 w-3" />
                      Reset All
                    </Button>
                  )}

                  <Collapsible
                    open={filtersExpanded}
                    onOpenChange={setFiltersExpanded}
                  >
                    <CollapsibleTrigger asChild>
                      <Button
                        variant={
                          genreFilter !== "all" ||
                          yearFilter !== "all" ||
                          statusFilter !== "all" ||
                          typeFilter !== "all"
                            ? "secondary"
                            : "outline"
                        }
                        size="sm"
                        className={cn(
                          "gap-2 bg-transparent relative",
                          (genreFilter !== "all" ||
                            yearFilter !== "all" ||
                            statusFilter !== "all" ||
                            typeFilter !== "all") &&
                            "bg-primary/10 border-primary/30 text-primary hover:bg-primary/20",
                        )}
                      >
                        <Filter className="h-4 w-4" />
                        Filters
                        {(genreFilter !== "all" ||
                          yearFilter !== "all" ||
                          statusFilter !== "all" ||
                          typeFilter !== "all") && (
                          <span className="absolute -top-1 -right-1 bg-primary text-primary-foreground text-xs rounded-full w-5 h-5 flex items-center justify-center">
                            {
                              [
                                genreFilter,
                                yearFilter,
                                statusFilter,
                                typeFilter,
                              ].filter((f) => f !== "all").length
                            }
                          </span>
                        )}
                        <ChevronDown
                          className={cn(
                            "h-4 w-4 transition-transform",
                            filtersExpanded && "rotate-180",
                          )}
                        />
                      </Button>
                    </CollapsibleTrigger>
                  </Collapsible>

                  <Collapsible
                    open={sortingExpanded}
                    onOpenChange={setSortingExpanded}
                  >
                    <CollapsibleTrigger asChild>
                      <Button
                        variant={
                          sortBy !== "rating" ||
                          sortOrder !== "desc" ||
                          groupBy !== "none"
                            ? "secondary"
                            : "outline"
                        }
                        size="sm"
                        className={cn(
                          "gap-2 bg-transparent relative",
                          (sortBy !== "rating" ||
                            sortOrder !== "desc" ||
                            groupBy !== "none") &&
                            "bg-amber-500/10 border-amber-500/30 text-amber-700 hover:bg-amber-500/20",
                        )}
                      >
                        <Settings2 className="h-4 w-4" />
                        Sort & View
                        {(sortBy !== "rating" ||
                          sortOrder !== "desc" ||
                          groupBy !== "none") && (
                          <span className="absolute -top-1 -right-1 bg-amber-500 text-white text-xs rounded-full w-5 h-5 flex items-center justify-center">
                            ●
                          </span>
                        )}
                        <ChevronDown
                          className={cn(
                            "h-4 w-4 transition-transform",
                            sortingExpanded && "rotate-180",
                          )}
                        />
                      </Button>
                    </CollapsibleTrigger>
                  </Collapsible>
                </div>
              </div>
            </div>
          </div>

          {/* Filters Panel */}
          <Collapsible open={filtersExpanded} onOpenChange={setFiltersExpanded}>
            <CollapsibleContent className="mt-4">
              <div className="bg-card rounded-lg border border-border p-4">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground">
                      Genre
                    </label>
                    <Select value={genreFilter} onValueChange={setGenreFilter}>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="All Genres" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="all">All Genres</SelectItem>
                        {uniqueGenres.map((genre) => (
                          <SelectItem key={genre} value={genre}>
                            {genre}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground">
                      Year
                    </label>
                    <Select value={yearFilter} onValueChange={setYearFilter}>
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="All Years" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="all">All Years</SelectItem>
                        {uniqueYears.map((year) => (
                          <SelectItem key={year} value={year.toString()}>
                            {year}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground">
                      Status
                    </label>
                    <Select
                      value={statusFilter}
                      onValueChange={setStatusFilter}
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="All Statuses" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="all">All Statuses</SelectItem>
                        {uniqueStatuses.map((status) => (
                          <SelectItem key={status} value={status}>
                            {status.replace("_", " ")}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground">
                      Group By
                    </label>
                    <Select
                      value={groupBy}
                      onValueChange={(value) => setGroupBy(value as any)}
                    >
                      <SelectTrigger className="w-full">
                        <SelectValue placeholder="No Grouping" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="none">No Grouping</SelectItem>
                        <SelectItem value="letter">By Letter</SelectItem>
                        <SelectItem value="year">By Decade</SelectItem>
                        <SelectItem value="rating">By Rating</SelectItem>
                        <SelectItem value="genre">By Genre</SelectItem>
                        <SelectItem value="status">By Status</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          {/* Sorting & View Panel */}
          <Collapsible open={sortingExpanded} onOpenChange={setSortingExpanded}>
            <CollapsibleContent className="mt-3">
              <div className="bg-card rounded-lg border border-border p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-foreground">
                        Sort By
                      </label>
                      <div className="flex items-center gap-2">
                        <Select
                          value={sortBy}
                          onValueChange={(value) => setSortBy(value as any)}
                        >
                          <SelectTrigger className="w-32">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="rating">Rating</SelectItem>
                            <SelectItem value="year">Year</SelectItem>
                            <SelectItem value="title">Title</SelectItem>
                            <SelectItem value="popularity">
                              Popularity
                            </SelectItem>
                            <SelectItem value="episodes">Episodes</SelectItem>
                          </SelectContent>
                        </Select>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            setSortOrder(sortOrder === "asc" ? "desc" : "asc")
                          }
                          className="px-3"
                        >
                          <ArrowUpDown className="h-4 w-4" />
                          {sortOrder === "asc" ? "Asc" : "Desc"}
                        </Button>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium text-foreground">
                      View Mode
                    </label>
                    <div className="flex items-center gap-1 border border-border rounded-lg p-1">
                      <Button
                        variant={viewMode === "detailed" ? "default" : "ghost"}
                        size="sm"
                        onClick={() => setViewMode("detailed")}
                        className="gap-2"
                      >
                        <List className="h-4 w-4" />
                        Detailed
                      </Button>
                      <Button
                        variant={viewMode === "compact" ? "default" : "ghost"}
                        size="sm"
                        onClick={() => setViewMode("compact")}
                        className="gap-2"
                      >
                        <Grid3X3 className="h-4 w-4" />
                        Compact
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            </CollapsibleContent>
          </Collapsible>

          {/* Active Filters Display */}
          {(activeFilters.length > 0 || searchTerm) && (
            <div className="flex items-center gap-2 mt-3">
              <div className="flex flex-wrap gap-2">
                {searchTerm && (
                  <div className="flex items-center gap-1 bg-primary/10 text-primary px-3 py-1.5 rounded-full text-sm">
                    search: "{searchTerm}"
                    <button
                      onClick={() => handleSearch("")}
                      className="hover:bg-primary/20 rounded-full p-1"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </div>
                )}
                {activeFilters.map((filter) => (
                  <div
                    key={filter}
                    className="flex items-center gap-1 bg-muted text-muted-foreground px-3 py-1.5 rounded-full text-sm"
                  >
                    {filter}
                    <button
                      onClick={() => removeFilter(filter)}
                      className="hover:bg-muted-foreground/20 rounded-full p-1"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  </div>
                ))}
                {(activeFilters.length > 0 || searchTerm) && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      clearAllFilters();
                      handleSearch("");
                    }}
                    className="px-3 py-1.5 h-auto text-sm"
                  >
                    Clear all
                  </Button>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Anime List - Natural content flow */}
        <div className="space-y-6 p-4">
          {filteredAndSortedAnimes.length === 0 ? (
            <div className="text-center py-16">
              <div className="w-16 h-16 bg-muted rounded-full flex items-center justify-center mx-auto mb-4">
                <Search className="h-8 w-8 text-muted-foreground" />
              </div>
              <h3 className="text-lg font-semibold mb-2">No anime found</h3>
              <p className="text-muted-foreground">
                Try adjusting your search or filters
              </p>
            </div>
          ) : (
            Object.entries(groupedAnimes).map(([groupKey, groupAnimes]) => (
              <div key={groupKey} id={`group-${groupKey}`}>
                {groupBy !== "none" && (
                  <div className="flex items-center gap-3 mb-4">
                    <Button
                      variant="ghost"
                      onClick={() => toggleGroup(groupKey)}
                      className="flex items-center gap-2 h-8 px-3 text-sm font-medium"
                    >
                      <ChevronRight
                        className={cn(
                          "h-4 w-4 transition-transform",
                          expandedGroups.has(groupKey) && "rotate-90",
                        )}
                      />
                      {groupKey}
                      <span className="text-muted-foreground">
                        ({groupAnimes.length})
                      </span>
                    </Button>
                  </div>
                )}

                {(groupBy === "none" || expandedGroups.has(groupKey)) && (
                  <div
                    className={cn(
                      "space-y-3",
                      viewMode === "compact" && "space-y-2",
                    )}
                  >
                    {groupAnimes.map((anime, index) =>
                      renderAnimeItem(anime, index),
                    )}
                  </div>
                )}
              </div>
            ))
          )}
        </div>
      </div>

      {/* Anime Drawer */}
      <AnimeDrawer
        anime={selectedAnime}
        isOpen={drawerOpen}
        onClose={() => {
          setDrawerOpen(false);
          setSelectedAnime(null);
        }}
        onRemove={onRemoveAnime}
        onViewDetails={(anime) => {
          // Navigate to anime detail page
          window.location.href = `/anime/${anime.id}`;
        }}
      />
    </>
  );
}
