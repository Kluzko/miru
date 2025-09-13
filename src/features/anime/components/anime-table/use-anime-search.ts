import { useState, useMemo, useCallback } from "react";
import type { Anime } from "@/types";

export function useAnimeSearch(animes: Anime[]) {
  const [searchTerm, setSearchTerm] = useState("");
  const [searchSuggestions, setSearchSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [recentSearches, setRecentSearches] = useState<string[]>([]);

  const handleSearch = useCallback(
    (value: string) => {
      setSearchTerm(value);
      if (value && !recentSearches.includes(value)) {
        setRecentSearches((prev) => [value, ...prev.slice(0, 4)]);
      }
    },
    [recentSearches],
  );

  const filteredSuggestions = useMemo(() => {
    if (searchTerm.length <= 1) return [];

    const searchLower = searchTerm.toLowerCase();
    const suggestions = animes
      .filter((anime) => {
        const titleMatch = anime.title.main
          .toLowerCase()
          .includes(searchLower);
        const genreMatch = anime.genres.some((genre) =>
          genre.name.toLowerCase().includes(searchLower),
        );
        const studioMatch = anime.studios.some((studio) =>
          studio.toLowerCase().includes(searchLower),
        );
        return titleMatch || genreMatch || studioMatch;
      })
      .sort((a, b) => {
        const aTitle = a.title.main.toLowerCase();
        const bTitle = b.title.main.toLowerCase();

        if (aTitle.startsWith(searchLower) && !bTitle.startsWith(searchLower))
          return -1;
        if (bTitle.startsWith(searchLower) && !aTitle.startsWith(searchLower))
          return 1;

        return (b.score || 0) - (a.score || 0);
      })
      .slice(0, 8)
      .map((anime) => anime.title.main);

    return suggestions;
  }, [searchTerm, animes]);

  // Update suggestions when they change
  useMemo(() => {
    setSearchSuggestions(filteredSuggestions);
    setShowSuggestions(filteredSuggestions.length > 0 && searchTerm.length > 1);
  }, [filteredSuggestions, searchTerm]);

  return {
    searchTerm,
    searchSuggestions,
    showSuggestions,
    recentSearches,
    handleSearch,
    setShowSuggestions,
  };
}
