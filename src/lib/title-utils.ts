import type { AnimeTitle } from "@/types";

export type TitleLanguage = 'main' | 'english' | 'japanese' | 'romaji' | 'native';

/**
 * Gets the preferred title based on user settings with fallback logic
 * Falls back to the next best available option, with main title as final fallback
 */
export function getPreferredTitle(title: AnimeTitle, preferredLanguage: TitleLanguage): string {
  // Try preferred language first
  switch (preferredLanguage) {
    case 'english':
      if (title.english) return title.english;
      break;
    case 'japanese':
      if (title.japanese) return title.japanese;
      break;
    case 'romaji':
      if (title.romaji) return title.romaji;
      break;
    case 'native':
      if (title.native) return title.native;
      break;
    case 'main':
      return title.main;
  }

  // Fallback logic: try in order of preference
  const fallbackOrder: Array<keyof AnimeTitle> = ['english', 'romaji', 'japanese', 'native'];

  for (const lang of fallbackOrder) {
    const titleValue = title[lang];
    if (titleValue && typeof titleValue === 'string') {
      return titleValue;
    }
  }

  // Final fallback to main title
  return title.main;
}
