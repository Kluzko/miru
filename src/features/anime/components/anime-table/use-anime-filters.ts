import { useState, useMemo, useCallback } from "react";
import type { Anime } from "@/types";

export type SortBy = "rating" | "year" | "title" | "popularity" | "episodes";
export type SortOrder = "asc" | "desc";
export type GroupBy = "none" | "letter" | "year" | "rating" | "genre" | "status";

export function useAnimeFilters(animes: Anime[]) {
  const [genreFilter, setGenreFilter] = useState<string>("all");
  const [yearFilter, setYearFilter] = useState<string>("all");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [sortBy, setSortBy] = useState<SortBy>("rating");
  const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
  const [groupBy, setGroupBy] = useState<GroupBy>("none");
  const [activeFilters, setActiveFilters] = useState<string[]>([]);

  const addQuickFilter = useCallback((type: string, value: string) => {
    const filterText = `${type}:${value}`;
    if (!activeFilters.includes(filterText)) {
      setActiveFilters((prev) => [...prev, filterText]);

      switch (type) {
        case "genre":
          setGenreFilter(value);
          break;
        case "year":
          setYearFilter(value);
          break;
        case "status":
          setStatusFilter(value);
          break;
        case "type":
          setTypeFilter(value);
          break;
      }
    }
  }, [activeFilters]);

  const removeFilter = useCallback((filterToRemove: string) => {
    setActiveFilters((prev) => prev.filter((f) => f !== filterToRemove));
    const [type] = filterToRemove.split(":");

    switch (type) {
      case "genre":
        setGenreFilter("all");
        break;
      case "year":
        setYearFilter("all");
        break;
      case "status":
        setStatusFilter("all");
        break;
      case "type":
        setTypeFilter("all");
        break;
    }
  }, []);

  const clearAllFilters = useCallback(() => {
    setActiveFilters([]);
    setGenreFilter("all");
    setYearFilter("all");
    setStatusFilter("all");
    setTypeFilter("all");
  }, []);

  const uniqueGenres = useMemo(() => {
    const genres = new Set<string>();
    animes.forEach((anime) =>
      anime.genres.forEach((genre) => genres.add(genre.name))
    );
    return Array.from(genres).sort();
  }, [animes]);

  const uniqueYears = useMemo(() => {
    const years = new Set<number>();
    animes.forEach((anime) => {
      if (anime.aired.from) {
        const year = new Date(anime.aired.from).getFullYear();
        years.add(year);
      }
    });
    return Array.from(years).sort((a, b) => b - a);
  }, [animes]);

  const uniqueStatuses = useMemo(() => {
    const statuses = new Set<string>();
    animes.forEach((anime) => statuses.add(anime.status));
    return Array.from(statuses);
  }, [animes]);

  const uniqueTypes = useMemo(() => {
    const types = new Set<string>();
    animes.forEach((anime) => types.add(anime.animeType));
    return Array.from(types);
  }, [animes]);

  return {
    genreFilter,
    setGenreFilter,
    yearFilter,
    setYearFilter,
    statusFilter,
    setStatusFilter,
    typeFilter,
    setTypeFilter,
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
    uniqueTypes,
  };
}
