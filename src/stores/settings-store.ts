import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type TitleLanguage = 'main' | 'english' | 'japanese' | 'romaji';

interface SettingsState {
  // Display preferences
  preferredTitleLanguage: TitleLanguage;

  // Actions
  setPreferredTitleLanguage: (language: TitleLanguage) => void;
  resetSettings: () => void;
}

const defaultSettings = {
  preferredTitleLanguage: 'main' as TitleLanguage,
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      ...defaultSettings,

      setPreferredTitleLanguage: (language: TitleLanguage) =>
        set({ preferredTitleLanguage: language }),

      resetSettings: () => set(defaultSettings),
    }),
    {
      name: 'miru-settings',
      version: 1,
    }
  )
);

// Helper function to get the preferred title from an anime object
export const getPreferredTitle = (
  title: { main: string; english?: string | null; japanese?: string | null; romaji?: string | null },
  preferredLanguage: TitleLanguage
): string => {
  switch (preferredLanguage) {
    case 'english':
      return title.english || title.main;
    case 'japanese':
      return title.japanese || title.main;
    case 'romaji':
      return title.romaji || title.main;
    case 'main':
    default:
      return title.main;
  }
};
