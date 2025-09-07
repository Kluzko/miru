import { invoke } from "@/lib/tauri";

export const collectionApi = {
  // Collection CRUD
  create: (name: string, description?: string) =>
    invoke("createCollection", { name, description: description ?? null }),

  get: (id: string) => invoke("getCollection", { id }),

  getAll: () => invoke("getAllCollections"),

  update: (id: string, updates: { name?: string; description?: string }) =>
    invoke("updateCollection", {
      id,
      name: updates.name ?? null,
      description: updates.description ?? null,
    }),

  delete: (id: string) => invoke("deleteCollection", { id }),

  // Anime management
  getAnime: (collectionId: string) =>
    invoke("getCollectionAnime", { collection_id: collectionId }),

  addAnime: (
    collectionId: string,
    animeId: string,
    userScore?: number,
    notes?: string,
  ) =>
    invoke("addAnimeToCollection", {
      collection_id: collectionId,
      anime_id: animeId,
      user_score: userScore ?? null,
      notes: notes ?? null,
    }),

  removeAnime: (collectionId: string, animeId: string) =>
    invoke("removeAnimeFromCollection", {
      collection_id: collectionId,
      anime_id: animeId,
    }),

  // Import
  importBatch: (titles: string[]) => invoke("importAnimeBatch", { titles }),

  importCsv: (csvContent: string) =>
    invoke("importFromCsv", { csv_content: csvContent }),
};
