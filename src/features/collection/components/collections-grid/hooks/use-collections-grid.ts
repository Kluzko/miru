import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useCollections } from "../../../hooks";
import type { Collection } from "@/types";

export function useCollectionsGrid() {
  const navigate = useNavigate();
  const { data: collections, isLoading } = useCollections();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingCollection, setEditingCollection] = useState<Collection | null>(null);
  const [deletingCollection, setDeletingCollection] = useState<Collection | null>(null);

  const handleCreateClick = () => {
    setDialogOpen(true);
  };

  const handleCollectionClick = (collectionId: string) => {
    navigate(`/collection/${collectionId}`);
  };

  const handleEditCollection = (collection: Collection) => {
    setEditingCollection(collection);
  };

  const handleDeleteCollection = (collection: Collection) => {
    setDeletingCollection(collection);
  };

  const handleCloseCreateDialog = () => {
    setDialogOpen(false);
  };

  const handleCloseEditDialog = () => {
    setEditingCollection(null);
  };

  const handleCloseDeleteDialog = () => {
    setDeletingCollection(null);
  };

  const handleDeleteSuccess = () => {
    setDeletingCollection(null);
  };

  return {
    // Data
    collections,
    isLoading,

    // Dialog states
    dialogOpen,
    editingCollection,
    deletingCollection,

    // Actions
    handleCreateClick,
    handleCollectionClick,
    handleEditCollection,
    handleDeleteCollection,
    handleCloseCreateDialog,
    handleCloseEditDialog,
    handleCloseDeleteDialog,
    handleDeleteSuccess,
  };
}
