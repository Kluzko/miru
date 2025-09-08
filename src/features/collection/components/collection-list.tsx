import { useState } from "react";
import { Plus, Edit, Trash, MoreHorizontal } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useNavigate } from "react-router-dom";
import { useCollections } from "../hooks";
import type { Collection } from "@/types";
import { CreateCollectionDialog } from "./create-collection-dialog";
import { EditCollectionDialog } from "./edit-collection-dialog";
import { DeleteCollectionDialog } from "./delete-collection-dialog";

export function CollectionList() {
  const navigate = useNavigate();
  const { data: collections, isLoading } = useCollections();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingCollection, setEditingCollection] = useState<Collection | null>(
    null,
  );
  const [deletingCollection, setDeletingCollection] =
    useState<Collection | null>(null);

  if (isLoading) {
    return <CollectionListSkeleton />;
  }

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {collections?.map((collection) => (
          <CollectionCard
            key={collection.id}
            collection={collection}
            onClick={() => navigate(`/collection/${collection.id}`)}
            onEdit={() => setEditingCollection(collection)}
            onDelete={() => setDeletingCollection(collection)}
          />
        ))}
        <Card className="border-dashed cursor-pointer hover:bg-accent/50 transition-colors">
          <CardContent className="flex items-center justify-center h-full min-h-[150px]">
            <Button
              variant="ghost"
              className="flex flex-col gap-2"
              onClick={() => setDialogOpen(true)}
            >
              <Plus className="h-8 w-8" />
              <span>Create Collection</span>
            </Button>
          </CardContent>
        </Card>
      </div>
      <CreateCollectionDialog open={dialogOpen} onOpenChange={setDialogOpen} />

      {editingCollection && (
        <EditCollectionDialog
          isOpen={!!editingCollection}
          onClose={() => setEditingCollection(null)}
          collection={editingCollection}
        />
      )}

      {deletingCollection && (
        <DeleteCollectionDialog
          isOpen={!!deletingCollection}
          onClose={() => setDeletingCollection(null)}
          collection={deletingCollection}
          onDeleted={() => setDeletingCollection(null)}
        />
      )}
    </>
  );
}

function CollectionCard({
  collection,
  onClick,
  onEdit,
  onDelete,
}: {
  collection: Collection;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const handleEdit = (e: React.MouseEvent) => {
    e.stopPropagation();
    onEdit();
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete();
  };

  return (
    <Card className="hover:shadow-lg transition-all group relative">
      <div className="cursor-pointer" onClick={onClick}>
        <CardHeader>
          <div className="flex items-start justify-between">
            <CardTitle className="line-clamp-1 pr-2">
              {collection.name}
            </CardTitle>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="sm"
                  className="h-8 w-8 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
                  onClick={(e) => e.stopPropagation()}
                >
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={handleEdit}>
                  <Edit className="h-4 w-4 mr-2" />
                  Edit
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={handleDelete}
                  className="text-destructive"
                >
                  <Trash className="h-4 w-4 mr-2" />
                  Delete
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
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
      </div>
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
