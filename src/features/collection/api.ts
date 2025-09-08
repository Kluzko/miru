// src/features/collections/api.ts
import { invoke } from "@/lib/tauri";
import {
  AddAnimeToCollectionRequest,
  CreateCollectionRequest,
  DeleteCollectionRequest,
  GetCollectionAnimeRequest,
  GetCollectionRequest,
  ImportAnimeBatchRequest,
  ImportFromCsvRequest,
  UpdateCollectionRequest,
} from "@/types";

export const collectionApi = {
  // Collection CRUD
  create: (request: CreateCollectionRequest) =>
    invoke("createCollection", request),

  get: (request: GetCollectionRequest) => invoke("getCollection", request),

  getAll: () => invoke("getAllCollections"),

  update: (request: UpdateCollectionRequest) =>
    invoke("updateCollection", request),

  delete: (request: DeleteCollectionRequest) =>
    invoke("deleteCollection", request),

  // Anime management
  getAnime: (request: GetCollectionAnimeRequest) =>
    invoke("getCollectionAnime", request),

  addAnime: (request: AddAnimeToCollectionRequest) =>
    invoke("addAnimeToCollection", request),

  removeAnime: (request: { collection_id: string; anime_id: string }) =>
    invoke("removeAnimeFromCollection", request),

  // Import
  importBatch: (request: ImportAnimeBatchRequest) =>
    invoke("importAnimeBatch", request),

  importCsv: (request: ImportFromCsvRequest) =>
    invoke("importFromCsv", request),
};
