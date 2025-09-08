import { Outlet } from "react-router-dom";
import { AppSidebar } from "./app-sidebar";

export function AppLayout() {
  return (
    <div className="flex h-screen bg-background">
      <AppSidebar />
      <div className="flex-1 overflow-hidden">
        <main className="flex-1 overflow-auto h-full">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
