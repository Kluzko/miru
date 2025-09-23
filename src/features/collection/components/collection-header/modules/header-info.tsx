import type { Collection } from "@/types";

interface HeaderInfoProps {
  collection: Collection;
}

export function HeaderInfo({ collection }: HeaderInfoProps) {
  return (
    <div>
      <h1 className="text-2xl font-bold">{collection.name}</h1>
      {collection.description && (
        <p className="text-muted-foreground mt-1">
          {collection.description}
        </p>
      )}
      <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
        <span>{collection.animeIds.length} anime</span>
        <span>•</span>
        <span>
          Updated {new Date(collection.updatedAt).toLocaleDateString()}
        </span>
      </div>
    </div>
  );
}
