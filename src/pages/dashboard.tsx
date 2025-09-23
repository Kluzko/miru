import { TrendingUp, Library, Clock, Star } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CollectionsGrid } from "@/features/collection/components";
import { useCollections } from "@/features/collection/hooks";

import { useMemo } from "react";

export function DashboardPage() {
  const { data: collections = [] } = useCollections();

  const stats = useMemo(() => {
    // Calculate unique anime across all collections
    const uniqueAnimeIds = new Set<string>();
    collections.forEach((collection) => {
      collection.animeIds.forEach((id) => uniqueAnimeIds.add(id));
    });

    // Get recent collections (created in last 30 days)
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);

    const recentlyAdded = collections
      .filter((collection) => new Date(collection.createdAt) > thirtyDaysAgo)
      .reduce((acc, c) => acc + c.animeIds.length, 0);

    return {
      totalCollections: collections.length,
      totalAnime: uniqueAnimeIds.size, // Use unique count instead of sum
      recentlyAdded,
      avgScore: 0, // TODO: Calculate based on user scores when available
    };
  }, [collections]);

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Dashboard</h1>
        <p className="text-muted-foreground">
          Welcome back! Here's your anime overview.
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Collections</CardTitle>
            <Library className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalCollections}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Anime</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalAnime}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              Recently Added
            </CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.recentlyAdded}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Score</CardTitle>
            <Star className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.avgScore.toFixed(1)}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Collections */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Your Collections</h2>
        <CollectionsGrid />
      </div>
    </div>
  );
}
