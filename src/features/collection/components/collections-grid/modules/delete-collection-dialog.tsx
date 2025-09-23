"use client";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Loader2, AlertTriangle } from "lucide-react";
import { useDeleteCollection } from "../../../hooks";
import type { Collection } from "@/types";

interface DeleteCollectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  collection: Collection;
  onDeleted?: () => void;
}

export function DeleteCollectionDialog({
  isOpen,
  onClose,
  collection,
  onDeleted,
}: DeleteCollectionDialogProps) {
  const deleteCollection = useDeleteCollection();

  const handleDelete = async () => {
    try {
      await deleteCollection.mutateAsync({
        id: collection.id,
      });
      onClose();
      onDeleted?.();
    } catch (error) {
      console.error("Failed to delete collection:", error);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-12 h-12 rounded-full bg-red-100">
              <AlertTriangle className="w-6 h-6 text-red-600" />
            </div>
            <div>
              <DialogTitle>Delete Collection</DialogTitle>
              <DialogDescription className="mt-1">
                This action cannot be undone.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="py-4">
          <p className="text-sm text-muted-foreground">
            Are you sure you want to delete{" "}
            <span className="font-medium text-foreground">
              "{collection.name}"
            </span>
            ? This will permanently remove the collection and all its anime
            associations.
          </p>
          {collection.animeIds.length > 0 && (
            <div className="mt-3 p-3 bg-muted rounded-lg">
              <p className="text-sm font-medium">
                This collection contains {collection.animeIds.length} anime
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                The anime themselves will not be deleted, only their association
                with this collection.
              </p>
            </div>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={onClose}
            disabled={deleteCollection.isPending}
          >
            Cancel
          </Button>
          <Button
            variant="destructive"
            onClick={handleDelete}
            disabled={deleteCollection.isPending}
          >
            {deleteCollection.isPending && (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            )}
            Delete Collection
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
