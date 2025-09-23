import { Edit, Trash, MoreHorizontal } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Collection } from "@/types";

interface CollectionCardProps {
  collection: Collection;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function CollectionCard({
  collection,
  onClick,
  onEdit,
  onDelete,
}: CollectionCardProps) {
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
