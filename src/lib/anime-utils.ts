import type { AnimeTitle, AnimeTier } from "@/types/bindings";

/**
 * Get the display title, preferring English over main if available
 */
export function getDisplayTitle(title: AnimeTitle): string {
  return title.english || title.main;
}

/**
 * Get tier display info with color mapping
 */
export function getTierInfo(tier: AnimeTier): { name: string; color: string } {
  const tierMap = {
    S: { name: "S", color: "#ffd700" },
    A: { name: "A", color: "#c0c0c0" },
    B: { name: "B", color: "#cd7f32" },
    C: { name: "C", color: "#4CAF50" },
    D: { name: "D", color: "#9E9E9E" },
  };
  return tierMap[tier] || { name: "Unknown", color: "#9E9E9E" };
}

/**
 * Check if anime has English title different from main title
 */
export function hasEnglishTitle(title: AnimeTitle): boolean {
  return title.english !== null && title.english !== title.main;
}
