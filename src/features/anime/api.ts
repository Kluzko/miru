import { invoke } from "@/lib/tauri";

export const animeApi = {
  search: async (query: string) => {
    return invoke("searchAnime", { query });
  },

  getById: async (id: string) => {
    return invoke("getAnimeById", { id });
  },

  getTop: async (page = 1, limit = 25) => {
    return invoke("getTopAnime", { page, limit });
  },

  getSeasonal: async (year: number, season: string, page = 1) => {
    return invoke("getSeasonalAnime", { year, season, page });
  },

  // Relations API - Single optimized call with auto-discovery
  getAnimeWithRelations: async (animeId: string) => {
    return invoke("getAnimeWithRelations", animeId);
  },

  // Auto-enrichment API for background enrichment on loading
  // This is the only enrichment method - it handles both initial load and manual enrichment
  autoEnrichOnLoad: async (animeId: string) => {
    return invoke("autoEnrichOnLoad", { animeId });
  },
};
