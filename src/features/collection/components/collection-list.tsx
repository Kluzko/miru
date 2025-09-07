import { Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useNavigate } from "react-router-dom";
import { useCollections } from "../hooks";
import type { Collection } from "@/types";

export function CollectionList() {
  const navigate = useNavigate();
  const { data: collections, isLoading } = useCollections();

  if (isLoading) {
    return <CollectionListSkeleton />;
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {collections?.map((collection) => (
        <CollectionCard
          key={collection.id}
          collection={collection}
          onClick={() => navigate(`/collection/${collection.id}`)}
        />
      ))}
      <Card className="border-dashed cursor-pointer hover:bg-accent/50 transition-colors">
        <CardContent className="flex items-center justify-center h-full min-h-[150px]">
          <Button
            variant="ghost"
            className="flex flex-col gap-2"
            onClick={() => {
              const name = prompt("Collection name:");
              if (name) {
                // Create collection logic
              }
            }}
          >
            <Plus className="h-8 w-8" />
            <span>Create Collection</span>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}

function CollectionCard({
  collection,
  onClick,
}: {
  collection: Collection;
  onClick: () => void;
}) {
  return (
    <Card
      className="cursor-pointer hover:shadow-lg transition-all"
      onClick={onClick}
    >
      <CardHeader>
        <CardTitle className="line-clamp-1">{collection.name}</CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground line-clamp-2">
          {collection.description || "No description"}
        </p>
        <div className="flex items-center justify-between mt-4">
          <span className="text-sm font-medium">
            {collection.animeIds.length} anime
          </span>
          <span className="text-xs text-muted-foreground">
            {new Date(collection.updatedAt).toLocaleDateString()}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

function CollectionListSkeleton() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {Array.from({ length: 6 }).map((_, i) => (
        <Card key={i}>
          <CardHeader>
            <Skeleton className="h-6 w-3/4" />
          </CardHeader>
          <CardContent>
            <Skeleton className="h-4 w-full mb-2" />
            <Skeleton className="h-4 w-2/3" />
            <div className="flex justify-between mt-4">
              <Skeleton className="h-4 w-16" />
              <Skeleton className="h-4 w-20" />
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
