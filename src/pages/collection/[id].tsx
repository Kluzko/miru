import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import {
  AnimeGrid,
  AnimeDetailSheet,
  AnimeTable,
} from "@/features/anime/components";
import {
  CollectionHeader,
  ImportDialog,
  EditCollectionDialog,
  DeleteCollectionDialog,
} from "@/features/collection/components";
import {
  useCollection,
  useCollectionAnime,
  useRemoveAnimeFromCollection,
} from "@/features/collection/hooks";
import { AnimeGridSkeleton } from "@/features/anime/components/anime-skeleton";
import { EmptyState } from "@/components/common/empty-state";
import type { Anime } from "@/types";
import { Skeleton } from "@/components/ui/skeleton";

export function CollectionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedAnime, setSelectedAnime] = useState<Anime | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [viewMode, setViewMode] = useState<"grid" | "table">("table");

  const { data: collection, isLoading: collectionLoading } = useCollection(id!);
  const { data: animeList = [], isLoading: animeLoading } = useCollectionAnime(
    id!,
  );
  const removeAnime = useRemoveAnimeFromCollection();

  if (collectionLoading || !collection) {
    return <CollectionDetailSkeleton />;
  }

  return (
    <div className="flex flex-col h-full">
      <CollectionHeader
        collection={collection}
        onImport={() => setImportOpen(true)}
        onEdit={() => setEditOpen(true)}
        onDelete={() => setDeleteOpen(true)}
        viewMode={viewMode}
        onViewModeChange={setViewMode}
      />

      <div
        className={
          viewMode === "table" ? "flex-1 p-6" : "flex-1 overflow-auto p-6"
        }
      >
        {animeLoading ? (
          <AnimeGridSkeleton />
        ) : animeList.length === 0 ? (
          <EmptyState
            title="No anime in this collection"
            description="Import anime to get started"
            action={
              <Button onClick={() => setImportOpen(true)}>Import Anime</Button>
            }
          />
        ) : viewMode === "table" ? (
          <AnimeTable
            animes={animeList}
            onRemoveAnime={async (animeId) => {
              try {
                await removeAnime.mutateAsync({
                  collection_id: id!,
                  anime_id: animeId,
                });
              } catch (error) {
                console.error("Failed to remove anime from collection:", error);
              }
            }}
          />
        ) : (
          <AnimeGrid
            anime={animeList}
            onAnimeClick={(anime) => {
              // Grid view can handle its own drawer/sheet
              setSelectedAnime(anime);
            }}
          />
        )}
      </div>

      {/* AnimeDetailSheet only for grid view */}
      {viewMode === "grid" && (
        <AnimeDetailSheet
          anime={selectedAnime}
          isOpen={!!selectedAnime}
          onClose={() => setSelectedAnime(null)}
        />
      )}

      <ImportDialog
        isOpen={importOpen}
        onClose={() => setImportOpen(false)}
        collectionId={id}
        onAnimesImported={(animeIds) => {
          console.log("Imported anime IDs:", animeIds);
          // The collection anime list will automatically refresh via query invalidation
        }}
      />

      <EditCollectionDialog
        isOpen={editOpen}
        onClose={() => setEditOpen(false)}
        collection={collection}
      />

      <DeleteCollectionDialog
        isOpen={deleteOpen}
        onClose={() => setDeleteOpen(false)}
        collection={collection}
        onDeleted={() => {
          // Navigate back to collections list after deletion
          navigate("/collections");
        }}
      />
    </div>
  );
}

function CollectionDetailSkeleton() {
  return (
    <div className="p-6">
      <div className="space-y-4 mb-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-96" />
      </div>
      <AnimeGridSkeleton />
    </div>
  );
}
