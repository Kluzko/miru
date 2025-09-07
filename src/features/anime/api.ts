import { invoke } from "@/lib/tauri";

export const animeApi = {
  search: (query: string) => invoke("searchAnime", { query }),

  getById: (id: string) => invoke("getAnimeById", { id }),

  getTop: (page = 1, limit = 25) => invoke("getTopAnime", { page, limit }),

  getSeasonal: (year: number, season: string, page = 1) =>
    invoke("getSeasonalAnime", { year, season, page }),
};
