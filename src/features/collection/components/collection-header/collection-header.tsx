"use client";

import type { Collection } from "@/types";

// Import hooks from the hooks directory
import { useCollectionHeader } from "./hooks";

// Import components from the modules directory
import { HeaderInfo, HeaderActions } from "./modules";

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
  const {
    collection: headerCollection,
    handleEdit,
    handleDelete,
    handleImport,
  } = useCollectionHeader({
    collection,
    onEdit,
    onDelete,
    onImport,
  });

  return (
    <div className="bg-card border-b px-6 py-4">
      <div className="flex items-center justify-between">
        <HeaderInfo collection={headerCollection} />
        <HeaderActions
          onEdit={handleEdit}
          onDelete={handleDelete}
          onImport={handleImport}
        />
      </div>
    </div>
  );
}
