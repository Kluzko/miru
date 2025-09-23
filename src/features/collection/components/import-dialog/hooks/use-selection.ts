import { useState, useCallback } from "react";

export function useSelection() {
  const [selectedExisting, setSelectedExisting] = useState<Set<string>>(
    new Set(),
  );

  const toggleExistingSelection = useCallback((animeId: string) => {
    setSelectedExisting((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(animeId)) {
        newSet.delete(animeId);
      } else {
        newSet.add(animeId);
      }
      return newSet;
    });
  }, []);

  const setInitialSelection = useCallback((animeIds: string[]) => {
    setSelectedExisting(new Set(animeIds));
  }, []);

  const resetSelection = useCallback(() => {
    setSelectedExisting(new Set());
  }, []);

  return {
    selectedExisting,
    toggleExistingSelection,
    setInitialSelection,
    resetSelection,
  };
}
