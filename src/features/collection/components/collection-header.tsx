import { Edit, Trash, Upload, Grid, List } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Collection } from "@/types";

interface CollectionHeaderProps {
  collection: Collection;
  onEdit?: () => void;
  onDelete?: () => void;
  onImport?: () => void;
  viewMode?: "grid" | "table";
  onViewModeChange?: (mode: "grid" | "table") => void;
}

export function CollectionHeader({
  collection,
  onEdit,
  onDelete,
  onImport,
  viewMode = "table",
  onViewModeChange,
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
          {onViewModeChange && (
            <div className="flex items-center gap-1 border border-border rounded-lg p-1 mr-2">
              <Button
                variant={viewMode === "table" ? "default" : "ghost"}
                size="sm"
                onClick={() => onViewModeChange("table")}
                className="h-8 px-2"
              >
                <List className="h-4 w-4" />
              </Button>
              <Button
                variant={viewMode === "grid" ? "default" : "ghost"}
                size="sm"
                onClick={() => onViewModeChange("grid")}
                className="h-8 px-2"
              >
                <Grid className="h-4 w-4" />
              </Button>
            </div>
          )}
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
