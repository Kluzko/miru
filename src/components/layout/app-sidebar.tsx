import { Library, Search, Settings, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  useCollections,
  useCreateCollection,
} from "@/features/collection/hooks";
import { useNavigate, useLocation } from "react-router-dom";

export function AppSidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const { data: collections = [] } = useCollections();
  const createCollection = useCreateCollection();

  const isActive = (path: string) => location.pathname === path;

  const handleCreateCollection = async () => {
    const name = prompt("Collection name:");
    if (name) {
      const result = await createCollection.mutateAsync({ name });
      navigate(`/collection/${result.id}`);
    }
  };

  return (
    <aside className="w-64 bg-card border-r flex flex-col">
      <div className="p-6 border-b">
        <h1 className="text-xl font-bold">Miru</h1>
      </div>

      <nav className="flex-1 p-4">
        <div className="space-y-1">
          <Button
            variant={isActive("/") ? "secondary" : "ghost"}
            className="w-full justify-start"
            onClick={() => navigate("/")}
          >
            <Library className="mr-2 h-4 w-4" />
            Dashboard
          </Button>

          <Button
            variant={isActive("/search") ? "secondary" : "ghost"}
            className="w-full justify-start"
            onClick={() => navigate("/search")}
          >
            <Search className="mr-2 h-4 w-4" />
            Search
          </Button>
        </div>

        <div className="mt-6">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium">Collections</span>
            <Button
              size="sm"
              variant="ghost"
              className="h-6 w-6 p-0"
              onClick={handleCreateCollection}
            >
              <Plus className="h-3 w-3" />
            </Button>
          </div>

          <div className="space-y-1">
            {collections.map((collection) => (
              <Button
                key={collection.id}
                variant={
                  isActive(`/collection/${collection.id}`)
                    ? "secondary"
                    : "ghost"
                }
                className="w-full justify-start text-sm"
                onClick={() => navigate(`/collection/${collection.id}`)}
              >
                {collection.name}
                <span className="ml-auto text-xs text-muted-foreground">
                  {collection.animeIds.length}
                </span>
              </Button>
            ))}
          </div>
        </div>
      </nav>

      <div className="p-4 border-t">
        <Button
          variant="ghost"
          className="w-full justify-start"
          onClick={() => navigate("/settings")}
        >
          <Settings className="mr-2 h-4 w-4" />
          Settings
        </Button>
      </div>
    </aside>
  );
}
