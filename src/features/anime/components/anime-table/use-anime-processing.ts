import { useMemo } from "react";
import type { Anime } from "@/types";
import type { SortBy, SortOrder, GroupBy } from "./use-anime-filters";

interface ProcessingOptions {
  searchTerm: string;
  genreFilter: string;
  yearFilter: string;
  statusFilter: string;
  typeFilter: string;
  sortBy: SortBy;
  sortOrder: SortOrder;
  groupBy: GroupBy;
}

export function useAnimeProcessing(animes: Anime[], options: ProcessingOptions) {
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
        const titleMatch = anime.title.toLowerCase().includes(searchLower);
        const genreMatch = anime.genres.some((genre) =>
          genre.name.toLowerCase().includes(searchLower)
        );
        const studioMatch = anime.studios.some((studio) =>
          studio.toLowerCase().includes(searchLower)
        );
        const synopsisMatch = anime.synopsis?.toLowerCase().includes(searchLower);

        if (!(titleMatch || genreMatch || studioMatch || synopsisMatch)) {
          return false;
        }
      }

      // Genre filter
      const matchesGenre =
        genreFilter === "all" ||
        anime.genres.some((genre) => genre.name === genreFilter);

      // Year filter
      const animeYear = anime.aired.from
        ? new Date(anime.aired.from).getFullYear().toString()
        : null;
      const matchesYear = yearFilter === "all" || animeYear === yearFilter;

      // Status filter
      const matchesStatus = statusFilter === "all" || anime.status === statusFilter;

      // Type filter
      const matchesType = typeFilter === "all" || anime.animeType === typeFilter;

      return matchesGenre && matchesYear && matchesStatus && matchesType;
    });

    // Sort animes
    if (searchTerm) {
      // If searching, prioritize relevance
      const searchLower = searchTerm.toLowerCase();
      filtered.sort((a, b) => {
        const aTitle = a.title.toLowerCase();
        const bTitle = b.title.toLowerCase();

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
            aValue = a.title.toLowerCase();
            bValue = b.title.toLowerCase();
            break;
          case "popularity":
            aValue = a.popularity || 999999;
            bValue = b.popularity || 999999;
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
          groupKey = anime.title[0].toUpperCase();
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
