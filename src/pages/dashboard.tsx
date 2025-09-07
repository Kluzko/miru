import { TrendingUp, Library, Clock, Star } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CollectionList } from "@/features/collection/components/collection-list";
import { useCollections } from "@/features/collection/hooks";
import { useTopAnime } from "@/features/anime/hooks";
import { AnimeGrid, AnimeGridSkeleton } from "@/features/anime/components";

export function DashboardPage() {
  const { data: collections = [], isLoading: collectionsLoading } =
    useCollections();
  const { data: topAnime = [], isLoading: topAnimeLoading } = useTopAnime();

  const stats = {
    totalCollections: collections.length,
    totalAnime: collections.reduce((acc, c) => acc + c.animeIds.length, 0),
    recentlyAdded: 0,
    avgScore: 0,
  };

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
        <CollectionList />
      </div>

      {/* Top Anime */}
      <div>
        <h2 className="text-xl font-semibold mb-4">Top Anime</h2>
        {topAnimeLoading ? (
          <AnimeGridSkeleton count={5} />
        ) : (
          <AnimeGrid anime={topAnime.slice(0, 5)} />
        )}
      </div>
    </div>
  );
}
