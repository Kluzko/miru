import { Edit, Trash, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";

interface HeaderActionsProps {
  onEdit?: () => void;
  onDelete?: () => void;
  onImport?: () => void;
}

export function HeaderActions({
  onEdit,
  onDelete,
  onImport,
}: HeaderActionsProps) {
  return (
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
  );
}
