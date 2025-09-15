import { useState, useMemo, useCallback } from "react";
import type { AnimeDetailed } from "@/types";

export type SortBy = "rating" | "year" | "title" | "episodes";
export type SortOrder = "asc" | "desc";
export type GroupBy =
  | "none"
  | "letter"
  | "year"
  | "rating"
  | "genre"
  | "status";

export function useAnimeFilters(animes: AnimeDetailed[]) {
  const [genreFilters, setGenreFilters] = useState<string[]>([]);
  const [yearRange, setYearRange] = useState<[number, number]>([
    1950,
    new Date().getFullYear(),
  ]);
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [typeFilter, setTypeFilter] = useState<string>("all");
  const [ageRestrictionFilter, setAgeRestrictionFilter] =
    useState<string>("all");
  const [scoreRange, setScoreRange] = useState<[number, number]>([0, 10]);
  const [sortBy, setSortBy] = useState<SortBy>("rating");
  const [sortOrder, setSortOrder] = useState<SortOrder>("desc");
  const [groupBy, setGroupBy] = useState<GroupBy>("none");
  const [activeFilters, setActiveFilters] = useState<string[]>([]);

  // Keep backward compatibility with single genre filter
  const genreFilter =
    genreFilters.length === 1
      ? genreFilters[0]
      : genreFilters.length > 1
        ? "multiple"
        : "all";
  const setGenreFilter = useCallback((genre: string) => {
    if (genre === "all") {
      setGenreFilters([]);
    } else {
      setGenreFilters([genre]);
    }
  }, []);

  // Keep backward compatibility with single year filter
  const yearFilter =
    yearRange[0] === 1950 && yearRange[1] === new Date().getFullYear()
      ? "all"
      : yearRange.toString();
  const setYearFilter = useCallback((year: string) => {
    if (year === "all") {
      setYearRange([1950, new Date().getFullYear()]);
    } else {
      const yearNum = parseInt(year);
      setYearRange([yearNum, yearNum]);
    }
  }, []);

  const addQuickFilter = useCallback(
    (type: string, value: string) => {
      const filterText = `${type}:${value}`;
      if (!activeFilters.includes(filterText)) {
        setActiveFilters((prev) => [...prev, filterText]);

        switch (type) {
          case "genre":
            setGenreFilters((prev) =>
              prev.includes(value) ? prev : [...prev, value],
            );
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
    },
    [activeFilters],
  );

  const removeFilter = useCallback(
    (filterToRemove: string) => {
      setActiveFilters((prev) => prev.filter((f) => f !== filterToRemove));
      const [type, value] = filterToRemove.split(":");

      switch (type) {
        case "genre":
          setGenreFilters((prev) => prev.filter((g) => g !== value));
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
    },
    [setYearFilter],
  );

  const clearAllFilters = useCallback(() => {
    setActiveFilters([]);
    setGenreFilters([]);
    setYearRange([1950, new Date().getFullYear()]);
    setStatusFilter("all");
    setTypeFilter("all");
    setAgeRestrictionFilter("all");
    setScoreRange([0, 10]);
  }, []);

  const uniqueGenres = useMemo(() => {
    const genres = new Set<string>();
    animes.forEach((anime) =>
      anime.genres.forEach((genre) => genres.add(genre.name)),
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

  const uniqueAgeRestrictions = useMemo(() => {
    const ageRestrictions = new Set<string>();
    animes.forEach((anime) => {
      if (anime.ageRestriction) {
        ageRestrictions.add(anime.ageRestriction);
      }
    });
    return Array.from(ageRestrictions).sort();
  }, [animes]);

  return {
    genreFilter,
    setGenreFilter,
    genreFilters,
    setGenreFilters,
    yearFilter,
    setYearFilter,
    yearRange,
    setYearRange,
    statusFilter,
    setStatusFilter,
    typeFilter,
    setTypeFilter,
    ageRestrictionFilter,
    setAgeRestrictionFilter,
    scoreRange,
    setScoreRange,
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
    uniqueAgeRestrictions,
  };
}
