import { useMemo } from "react";
import type { Anime } from "@/types";
import type { SortBy, SortOrder, GroupBy } from "./use-anime-filters";

interface ProcessingOptions {
  searchTerm: string;
  genreFilter: string; // Keep for backward compatibility
  genreFilters?: string[]; // New multi-select support
  yearFilter: string; // Keep for backward compatibility
  yearRange?: [number, number]; // New range support
  statusFilter: string;
  typeFilter: string;
  ageRestrictionFilter: string;
  scoreRange: [number, number];
  sortBy: SortBy;
  sortOrder: SortOrder;
  groupBy: GroupBy;
}

export function useAnimeProcessing(
  animes: Anime[],
  options: ProcessingOptions,
) {
  const filteredAndSortedAnimes = useMemo(() => {
    const {
      searchTerm,
      genreFilter,
      yearFilter,
      statusFilter,
      typeFilter,
      sortBy,
      sortOrder,
    } = options;

    // Filter animes
    const filtered = animes.filter((anime) => {
      // Search filter
      if (searchTerm) {
        const searchLower = searchTerm.toLowerCase();
        const titleMatch = anime.title.main.toLowerCase().includes(searchLower);
        const genreMatch = anime.genres.some((genre) =>
          genre.name.toLowerCase().includes(searchLower),
        );
        const studioMatch = anime.studios.some((studio) =>
          studio.toLowerCase().includes(searchLower),
        );
        const synopsisMatch = anime.synopsis
          ?.toLowerCase()
          .includes(searchLower);

        if (!(titleMatch || genreMatch || studioMatch || synopsisMatch)) {
          return false;
        }
      }

      // Genre filter - support both single and multiple genres
      const matchesGenre = (() => {
        // If genreFilters array is provided and not empty, use multi-select logic
        if (options.genreFilters && options.genreFilters.length > 0) {
          return anime.genres.some((genre) =>
            options.genreFilters!.includes(genre.name),
          );
        }
        // Fallback to single genre filter
        return (
          genreFilter === "all" ||
          anime.genres.some((genre) => genre.name === genreFilter)
        );
      })();

      // Year filter - support both single year and range
      const matchesYear = (() => {
        if (options.yearRange) {
          const animeYear = anime.aired.from
            ? new Date(anime.aired.from).getFullYear()
            : null;
          if (!animeYear) return false;
          return (
            animeYear >= options.yearRange[0] &&
            animeYear <= options.yearRange[1]
          );
        }
        // Fallback to single year filter
        const animeYear = anime.aired.from
          ? new Date(anime.aired.from).getFullYear().toString()
          : null;
        return yearFilter === "all" || animeYear === yearFilter;
      })();

      // Status filter
      const matchesStatus =
        statusFilter === "all" || anime.status === statusFilter;

      // Type filter
      const matchesType =
        typeFilter === "all" || anime.animeType === typeFilter;

      // Age restriction filter
      const matchesAgeRestriction =
        options.ageRestrictionFilter === "all" ||
        anime.ageRestriction === options.ageRestrictionFilter;

      // Score filter
      const matchesScore = (() => {
        const score = anime.score || 0;
        return score >= options.scoreRange[0] && score <= options.scoreRange[1];
      })();

      return (
        matchesGenre &&
        matchesYear &&
        matchesStatus &&
        matchesType &&
        matchesAgeRestriction &&
        matchesScore
      );
    });

    // Sort animes
    if (searchTerm) {
      // If searching, prioritize relevance
      const searchLower = searchTerm.toLowerCase();
      filtered.sort((a, b) => {
        const aTitle = a.title.main.toLowerCase();
        const bTitle = b.title.main.toLowerCase();

        // Exact match first
        if (aTitle === searchLower && bTitle !== searchLower) return -1;
        if (bTitle === searchLower && aTitle !== searchLower) return 1;

        // Starts with search term
        const aTitleMatch = aTitle.startsWith(searchLower);
        const bTitleMatch = bTitle.startsWith(searchLower);
        if (aTitleMatch && !bTitleMatch) return -1;
        if (bTitleMatch && !aTitleMatch) return 1;

        // Contains search term in title
        const aContainsTitle = aTitle.includes(searchLower);
        const bContainsTitle = bTitle.includes(searchLower);
        if (aContainsTitle && !bContainsTitle) return -1;
        if (bContainsTitle && !aContainsTitle) return 1;

        // Fallback to rating
        return (b.score || 0) - (a.score || 0);
      });
    } else {
      // Normal sorting
      filtered.sort((a, b) => {
        let aValue: number | string;
        let bValue: number | string;

        switch (sortBy) {
          case "rating":
            aValue = a.score || 0;
            bValue = b.score || 0;
            break;
          case "year":
            aValue = a.aired.from ? new Date(a.aired.from).getFullYear() : 0;
            bValue = b.aired.from ? new Date(b.aired.from).getFullYear() : 0;
            break;
          case "title":
            aValue = a.title.main.toLowerCase();
            bValue = b.title.main.toLowerCase();
            break;

          case "episodes":
            aValue = a.episodes || 0;
            bValue = b.episodes || 0;
            break;
          default:
            return 0;
        }

        if (typeof aValue === "string" && typeof bValue === "string") {
          return sortOrder === "asc"
            ? aValue.localeCompare(bValue)
            : bValue.localeCompare(aValue);
        }

        return sortOrder === "asc"
          ? (aValue as number) - (bValue as number)
          : (bValue as number) - (aValue as number);
      });
    }

    return filtered;
  }, [animes, options]);

  const groupedAnimes = useMemo(() => {
    if (options.groupBy === "none") {
      return { All: filteredAndSortedAnimes };
    }

    const groups: Record<string, Anime[]> = {};

    filteredAndSortedAnimes.forEach((anime) => {
      let groupKey: string;

      switch (options.groupBy) {
        case "letter":
          groupKey = anime.title.main[0].toUpperCase();
          break;
        case "year":
          const year = anime.aired.from
            ? new Date(anime.aired.from).getFullYear()
            : 2000;
          const decade = Math.floor(year / 10) * 10;
          groupKey = `${decade}s`;
          break;
        case "rating":
          const rating = Math.floor(anime.score || 0);
          groupKey = `${rating}-${rating + 1} Stars`;
          break;
        case "genre":
          groupKey = anime.genres[0]?.name || "Unknown";
          break;
        case "status":
          groupKey = anime.status;
          break;
        default:
          groupKey = "All";
      }

      if (!groups[groupKey]) groups[groupKey] = [];
      groups[groupKey].push(anime);
    });

    return groups;
  }, [filteredAndSortedAnimes, options.groupBy]);

  return {
    filteredAndSortedAnimes,
    groupedAnimes,
  };
}
