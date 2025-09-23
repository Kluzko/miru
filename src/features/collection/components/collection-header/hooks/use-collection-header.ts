import type { Collection } from "@/types";

interface UseCollectionHeaderProps {
  collection: Collection;
  onEdit?: () => void;
  onDelete?: () => void;
  onImport?: () => void;
}

export function useCollectionHeader({
  collection,
  onEdit,
  onDelete,
  onImport,
}: UseCollectionHeaderProps) {
  const handleEdit = () => {
    onEdit?.();
  };

  const handleDelete = () => {
    onDelete?.();
  };

  const handleImport = () => {
    onImport?.();
  };

  return {
    collection,
    handleEdit,
    handleDelete,
    handleImport,
  };
}
