import { Library, Search, Settings, Folder, Bell } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { useNavigate, useLocation } from "react-router-dom";

export function AppSidebar() {
  const navigate = useNavigate();
  const location = useLocation();

  const isActive = (path: string) => location.pathname === path;

  return (
    <aside className="w-64 bg-card border-r flex flex-col">
      {/* Logo/Brand Section */}
      <div className="p-6 border-b">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold">Miru</h1>
        </div>
      </div>

      <nav className="flex-1 p-4 space-y-6">
        {/* Main Navigation */}
        <div className="space-y-1">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
            Overview
          </p>
          <Button
            variant={isActive("/") ? "secondary" : "ghost"}
            className="w-full justify-start h-9"
            onClick={() => navigate("/")}
          >
            <Library className="mr-3 h-4 w-4" />
            Dashboard
          </Button>

          <Button
            variant={isActive("/search") ? "secondary" : "ghost"}
            className="w-full justify-start h-9"
            onClick={() => navigate("/search")}
          >
            <Search className="mr-3 h-4 w-4" />
            Search Anime
          </Button>
        </div>

        <Separator />

        {/* Collections Section */}
        <div className="space-y-1">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
            Library
          </p>
          <Button
            variant={isActive("/collections") ? "secondary" : "ghost"}
            className="w-full justify-start h-9"
            onClick={() => navigate("/collections")}
          >
            <Folder className="mr-3 h-4 w-4" />
            Collections
            <Badge variant="secondary" className="ml-auto text-xs">
              New
            </Badge>
          </Button>
        </div>

        <Separator />

        {/* User Section */}
        <div className="space-y-1">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
            Account
          </p>

          <Button variant="ghost" className="w-full justify-start h-9 relative">
            <Bell className="mr-3 h-4 w-4" />
            Notifications
            <Badge
              variant="destructive"
              className="ml-auto text-xs min-w-5 h-5 p-0 flex items-center justify-center"
            >
              3{/*For now static*/}
            </Badge>
          </Button>
        </div>
      </nav>

      {/* Settings at bottom */}
      <div className="p-4 border-t">
        <Button
          variant={isActive("/settings") ? "secondary" : "ghost"}
          className="w-full justify-start h-9"
          onClick={() => navigate("/settings")}
        >
          <Settings className="mr-3 h-4 w-4" />
          Settings
        </Button>
      </div>
    </aside>
  );
}
