import { Plus } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface CreateNewCollectionCardProps {
  onClick: () => void;
}

export function CreateNewCollectionCard({ onClick }: CreateNewCollectionCardProps) {
  return (
    <Card className="border-dashed cursor-pointer hover:bg-accent/50 transition-colors">
      <CardContent className="flex items-center justify-center h-full min-h-[150px]">
        <Button
          variant="ghost"
          className="flex flex-col gap-2"
          onClick={onClick}
        >
          <Plus className="h-8 w-8" />
          <span>Create Collection</span>
        </Button>
      </CardContent>
    </Card>
  );
}
