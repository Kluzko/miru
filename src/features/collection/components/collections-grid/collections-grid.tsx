"use client";

// Import hooks from the hooks directory
import { useCollectionsGrid } from "./hooks";

// Import components from the modules directory
import {
  CollectionCard,
  CreateNewCollectionCard,
  CollectionsSkeleton,
  CreateCollectionDialog,
  EditCollectionDialog,
  DeleteCollectionDialog,
} from "./modules";

export function CollectionsGrid() {
  const {
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
  } = useCollectionsGrid();

  if (isLoading) {
    return <CollectionsSkeleton />;
  }

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {collections?.map((collection) => (
          <CollectionCard
            key={collection.id}
            collection={collection}
            onClick={() => handleCollectionClick(collection.id)}
            onEdit={() => handleEditCollection(collection)}
            onDelete={() => handleDeleteCollection(collection)}
          />
        ))}
        <CreateNewCollectionCard onClick={handleCreateClick} />
      </div>

      {/* Dialogs */}
      <CreateCollectionDialog
        open={dialogOpen}
        onOpenChange={handleCloseCreateDialog}
      />

      {editingCollection && (
        <EditCollectionDialog
          isOpen={!!editingCollection}
          onClose={handleCloseEditDialog}
          collection={editingCollection}
        />
      )}

      {deletingCollection && (
        <DeleteCollectionDialog
          isOpen={!!deletingCollection}
          onClose={handleCloseDeleteDialog}
          collection={deletingCollection}
          onDeleted={handleDeleteSuccess}
        />
      )}
    </>
  );
}
