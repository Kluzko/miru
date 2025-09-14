import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { AnimeTable } from "@/features/anime/components";
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

import { EmptyState } from "@/components/common/empty-state";
import { Skeleton } from "@/components/ui/skeleton";

export function CollectionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [importOpen, setImportOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

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
      />

      <div className="flex-1 p-6">
        {animeLoading ? (
          <CollectionDetailSkeleton />
        ) : animeList.length === 0 ? (
          <EmptyState
            title="No anime in this collection"
            description="Import anime to get started"
            action={
              <Button onClick={() => setImportOpen(true)}>Import Anime</Button>
            }
          />
        ) : (
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
        )}
      </div>

      <ImportDialog
        isOpen={importOpen}
        onClose={() => setImportOpen(false)}
        collectionId={id}
        onAnimesImported={(animeIds: string[]) => {
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
    <div className="space-y-4">
      <div className="space-y-4 mb-6">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-96" />
      </div>
      <div className="space-y-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <div key={i} className="flex items-center space-x-4">
            <Skeleton className="h-16 w-16 rounded" />
            <div className="space-y-2 flex-1">
              <Skeleton className="h-4 w-1/3" />
              <Skeleton className="h-3 w-1/2" />
            </div>
            <Skeleton className="h-4 w-16" />
          </div>
        ))}
      </div>
    </div>
  );
}
