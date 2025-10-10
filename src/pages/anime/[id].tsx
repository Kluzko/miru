import { useParams } from "react-router-dom";
import { useAnimeDetail } from "@/features/anime/hooks";
import { AnimeDetailTabs } from "@/features/anime/components";
import { Skeleton } from "@/components/ui/skeleton";

function AnimeDetailSkeleton() {
  return (
    <div className="w-full max-w-6xl mx-auto">
      <div className="relative bg-background border-b">
        <div className="px-4 sm:px-6 py-4 sm:py-8">
          <div className="flex flex-col sm:flex-row gap-4 sm:gap-8 items-start">
            {/* Anime poster skeleton */}
            <div className="mx-auto sm:mx-0">
              <Skeleton className="aspect-[3/4] w-32 sm:w-40 lg:w-48 rounded-lg" />
            </div>

            {/* Content skeleton */}
            <div className="flex-1 space-y-4 sm:space-y-6 w-full">
              {/* Badges skeleton */}
              <div className="space-y-2 sm:space-y-3 text-center sm:text-left">
                <div className="flex flex-wrap gap-2 justify-center sm:justify-start">
                  <Skeleton className="h-6 w-16" />
                  <Skeleton className="h-6 w-20" />
                  <Skeleton className="h-6 w-12" />
                </div>

                {/* Title skeleton */}
                <div className="space-y-2">
                  <Skeleton className="h-8 sm:h-10 w-3/4 mx-auto sm:mx-0" />
                  <Skeleton className="h-5 sm:h-6 w-1/2 mx-auto sm:mx-0" />
                </div>
              </div>

              {/* Stats grid skeleton */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 sm:gap-4">
                {[...Array(4)].map((_, i) => (
                  <Skeleton key={i} className="h-16 sm:h-20 rounded-lg" />
                ))}
              </div>

              {/* Action buttons skeleton */}
              <div className="flex flex-col sm:flex-row gap-2 sm:gap-3">
                <Skeleton className="h-8 w-full sm:w-32" />
                <Skeleton className="h-8 w-full sm:w-24" />
                <Skeleton className="h-8 w-full sm:w-28" />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs skeleton */}
      <div className="bg-background border-b">
        <div className="px-4 sm:px-6">
          <div className="flex space-x-6 py-4">
            <Skeleton className="h-6 w-20" />
            <Skeleton className="h-6 w-16" />
          </div>
        </div>
      </div>

      {/* Tab content skeleton */}
      <div className="px-4 sm:px-6 py-6">
        <div className="space-y-4">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-4 w-5/6" />
          <Skeleton className="h-4 w-4/5" />
          <div className="pt-4">
            <Skeleton className="h-8 w-1/3 mb-4" />
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {[...Array(6)].map((_, i) => (
                <Skeleton key={i} className="h-24 rounded-lg" />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function AnimeDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data: anime, isLoading, error } = useAnimeDetail(id!);

  if (isLoading) {
    return (
      <div className="container mx-auto px-6 py-8">
        <AnimeDetailSkeleton />
      </div>
    );
  }

  if (error) {
    return (
      <div className="container mx-auto px-6 py-8 max-w-6xl">
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-2">
            <h2 className="text-xl font-semibold text-red-600">
              Error Loading Anime
            </h2>
            <p className="text-gray-600">
              Failed to load anime details. Please try again.
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (!anime) {
    return (
      <div className="container mx-auto px-6 py-8 max-w-6xl">
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-2">
            <h2 className="text-xl font-semibold">Anime Not Found</h2>
            <p className="text-gray-600">
              The requested anime could not be found.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-6 py-8">
      <AnimeDetailTabs anime={anime} />
    </div>
  );
}
