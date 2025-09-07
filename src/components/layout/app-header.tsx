import { Bell, Search, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useNavigate } from "react-router-dom";

export function AppHeader() {
  const navigate = useNavigate();

  return (
    <header className="h-16 border-b bg-card flex items-center px-6">
      <div className="flex-1">
        <Button
          variant="ghost"
          size="sm"
          className="gap-2"
          onClick={() => navigate("/search")}
        >
          <Search className="h-4 w-4" />
          <span className="text-muted-foreground">Search anime...</span>
        </Button>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon">
          <Bell className="h-5 w-5" />
        </Button>
        <Button variant="ghost" size="icon">
          <User className="h-5 w-5" />
        </Button>
      </div>
    </header>
  );
}
