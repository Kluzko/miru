import { Edit, Trash, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Collection } from "@/types";

interface CollectionHeaderProps {
  collection: Collection;
  onEdit?: () => void;
  onDelete?: () => void;
  onImport?: () => void;
}

export function CollectionHeader({
  collection,
  onEdit,
  onDelete,
  onImport,
}: CollectionHeaderProps) {
  return (
    <div className="bg-card border-b px-6 py-4">
      <div className="flex items-center justify-between">
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
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={onImport}>
            <Upload className="h-4 w-4 mr-2" />
            Import
          </Button>
          <Button variant="outline" size="sm" onClick={onEdit}>
            <Edit className="h-4 w-4 mr-2" />
            Edit
          </Button>
          <Button variant="outline" size="sm" onClick={onDelete}>
            <Trash className="h-4 w-4 mr-2" />
            Delete
          </Button>
        </div>
      </div>
    </div>
  );
}
