import { useState, useMemo, useCallback } from "react";
import type { Anime } from "@/types";
import { getPreferredTitle } from "@/lib/title-utils";
import { useSettingsStore } from "@/stores/settings-store";

export function useAnimeSearch(animes: Anime[]) {
  const { preferredTitleLanguage } = useSettingsStore();
  const [searchTerm, setSearchTerm] = useState("");
  const [searchSuggestions, setSearchSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [recentSearches, setRecentSearches] = useState<string[]>([]);

  const handleSearch = useCallback(
    (value: string) => {
      setSearchTerm(value);
      if (
        value.trim() &&
        value.length >= 2 &&
        !recentSearches.includes(value)
      ) {
        setRecentSearches((prev) => [value, ...prev.slice(0, 3)]); // Keep only 3 recent searches
      }
    },
    [recentSearches],
  );

  const filteredSuggestions = useMemo(() => {
    if (searchTerm.length < 2) return []; // Minimum 2 characters

    const searchLower = searchTerm.toLowerCase();

    // Enhanced search across all title variants
    const suggestions = animes
      .filter((anime) => {
        const titleVariants = [
          anime.title.main,
          anime.title.english,
          anime.title.japanese,
          anime.title.romaji,
          anime.title.native,
          ...anime.title.synonyms,
        ].filter(Boolean);

        const titleMatch = titleVariants.some((title) =>
          title?.toLowerCase().includes(searchLower),
        );

        const genreMatch = anime.genres.some((genre) =>
          genre.name.toLowerCase().includes(searchLower),
        );

        return titleMatch || genreMatch;
      })
      .sort((a, b) => {
        const aTitle = getPreferredTitle(
          a.title,
          preferredTitleLanguage,
        ).toLowerCase();
        const bTitle = getPreferredTitle(
          b.title,
          preferredTitleLanguage,
        ).toLowerCase();

        // Prioritize titles that start with search term
        if (aTitle.startsWith(searchLower) && !bTitle.startsWith(searchLower))
          return -1;
        if (bTitle.startsWith(searchLower) && !aTitle.startsWith(searchLower))
          return 1;

        // Then sort by score
        return (b.compositeScore || 0) - (a.compositeScore || 0);
      })
      .slice(0, 5) // Show fewer suggestions for cleaner look
      .map((anime) => getPreferredTitle(anime.title, preferredTitleLanguage));

    return suggestions;
  }, [searchTerm, animes, preferredTitleLanguage]);

  // Update suggestions when they change
  useMemo(() => {
    setSearchSuggestions(filteredSuggestions);
    setShowSuggestions(
      filteredSuggestions.length > 0 && searchTerm.length >= 2,
    );
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
