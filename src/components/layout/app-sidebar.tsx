import { Library, Search, Settings, Folder, Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { useNavigate, useLocation } from "react-router-dom";
import { useState, useEffect } from "react";

export function AppSidebar() {
  const navigate = useNavigate();
  const location = useLocation();
  const [isCollapsed, setIsCollapsed] = useState(false);

  const isActive = (path: string) => location.pathname === path;

  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth < 768) {
        setIsCollapsed(true);
      } else {
        setIsCollapsed(false);
      }
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  const toggleCollapse = () => {
    setIsCollapsed(!isCollapsed);
  };

  return (
    <aside
      className={`${isCollapsed ? "w-16" : "w-64"} bg-card border-r flex flex-col transition-all duration-300`}
    >
      {/* Logo/Brand Section */}
      <div className="p-4 border-b">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="sm"
            className="p-2"
            onClick={toggleCollapse}
          >
            <Menu className="h-4 w-4" />
          </Button>
          {!isCollapsed && <h1 className="text-xl font-bold">Miru</h1>}
        </div>
      </div>

      <nav className={`flex-1 p-4 ${isCollapsed ? "space-y-2" : "space-y-6"}`}>
        {/* Main Navigation */}
        <div className={isCollapsed ? "space-y-2" : "space-y-1"}>
          {!isCollapsed && (
            <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
              Overview
            </p>
          )}
          <Button
            variant={isActive("/") ? "secondary" : "ghost"}
            className={`w-full h-9 ${isCollapsed ? "justify-center px-2" : "justify-start"}`}
            onClick={() => navigate("/")}
            title={isCollapsed ? "Dashboard" : ""}
          >
            <Library className={`h-4 w-4 ${isCollapsed ? "" : "mr-3"}`} />
            {!isCollapsed && "Dashboard"}
          </Button>

          <Button
            variant={isActive("/explore") ? "secondary" : "ghost"}
            className={`w-full h-9 ${isCollapsed ? "justify-center px-2" : "justify-start"}`}
            onClick={() => navigate("/explore")}
            title={isCollapsed ? "Explore Anime" : ""}
          >
            <Search className={`h-4 w-4 ${isCollapsed ? "" : "mr-3"}`} />
            {!isCollapsed && "Explore Anime"}
          </Button>
        </div>

        {!isCollapsed && <Separator />}

        {/* Collections Section */}
        <div className={isCollapsed ? "space-y-2" : "space-y-1"}>
          {!isCollapsed && (
            <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
              Library
            </p>
          )}
          <Button
            variant={isActive("/collections") ? "secondary" : "ghost"}
            className={`w-full h-9 ${isCollapsed ? "justify-center px-2" : "justify-start"}`}
            onClick={() => navigate("/collections")}
            title={isCollapsed ? "Collections" : ""}
          >
            <Folder className={`h-4 w-4 ${isCollapsed ? "" : "mr-3"}`} />
            {!isCollapsed && "Collections"}
            {!isCollapsed && (
              <Badge variant="secondary" className="ml-auto text-xs">
                New
              </Badge>
            )}
          </Button>
        </div>

        {!isCollapsed && <Separator />}
      </nav>

      {/* Settings at bottom */}
      <div className="p-4 border-t">
        <Button
          variant={isActive("/settings") ? "secondary" : "ghost"}
          className={`w-full h-9 ${isCollapsed ? "justify-center px-2" : "justify-start"}`}
          onClick={() => navigate("/settings")}
          title={isCollapsed ? "Settings" : ""}
        >
          <Settings className={`h-4 w-4 ${isCollapsed ? "" : "mr-3"}`} />
          {!isCollapsed && "Settings"}
        </Button>
      </div>
    </aside>
  );
}
