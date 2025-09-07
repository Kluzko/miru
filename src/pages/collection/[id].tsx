// src/pages/collection/[id].tsx
import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AnimeGrid, AnimeDetailSheet } from "@/features/anime/components";
import {
  CollectionHeader,
  ImportDialog,
} from "@/features/collection/components";
import { useCollection, useCollectionAnime } from "@/features/collection/hooks";
import { AnimeGridSkeleton } from "@/features/anime/components/anime-skeleton";
import { EmptyState } from "@/components/common/empty-state";
import type { Anime } from "@/types";
import { Skeleton } from "@/components/ui/skeleton";

export function CollectionDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedAnime, setSelectedAnime] = useState<Anime | null>(null);
  const [importOpen, setImportOpen] = useState(false);

  const { data: collection, isLoading: collectionLoading } = useCollection(id!);
  const { data: animeList = [], isLoading: animeLoading } = useCollectionAnime(
    id!,
  );

  if (collectionLoading || !collection) {
    return <CollectionDetailSkeleton />;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="px-6 py-4">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => navigate(-1)}
          className="mb-4"
        >
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back
        </Button>
      </div>

      <CollectionHeader
        collection={collection}
        onImport={() => setImportOpen(true)}
      />

      <div className="flex-1 overflow-auto p-6">
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
        ) : (
          <AnimeGrid anime={animeList} onAnimeClick={setSelectedAnime} />
        )}
      </div>

      <AnimeDetailSheet
        anime={selectedAnime}
        isOpen={!!selectedAnime}
        onClose={() => setSelectedAnime(null)}
      />

      <ImportDialog
        isOpen={importOpen}
        onClose={() => setImportOpen(false)}
        collectionId={id!}
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
