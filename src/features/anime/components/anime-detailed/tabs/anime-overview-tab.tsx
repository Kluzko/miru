"use client";

import {
  Star,
  Award,
  Eye,
  ChevronDown,
  ChevronUp,
  Calendar,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { AnimeDetailed } from "@/types";
import { useState } from "react";

interface AnimeOverviewTabProps {
  anime: AnimeDetailed;
}

const AnimeOverviewTab: React.FC<AnimeOverviewTabProps> = ({ anime }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [showFullSynopsis, setShowFullSynopsis] = useState(false);

  const startDate = anime.aired.from
    ? new Date(anime.aired.from).toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : "TBA";

  const endDate = anime.aired.to
    ? new Date(anime.aired.to).toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
      })
    : anime.status === "Airing"
      ? "Ongoing"
      : "TBA";

  const normalizeScore = (score: number) => Math.min(Math.max(score, 0), 10);

  const popularityScore = normalizeScore(anime.qualityMetrics.popularityScore);
  const engagementScore = normalizeScore(anime.qualityMetrics.engagementScore);
  const consistencyScore = normalizeScore(
    anime.qualityMetrics.consistencyScore,
  );
  const reachScore = normalizeScore(anime.qualityMetrics.audienceReachScore);

  const synopsis =
    anime.synopsis || anime.description || "No synopsis available.";
  const truncatedSynopsis =
    synopsis.length > 300 ? synopsis.slice(0, 300) + "..." : synopsis;
  const shouldShowReadMore = synopsis.length > 300;

  return (
    <div className="space-y-6">
      {/* Enhanced Synopsis Section */}
      <Card className="transition-all duration-300 hover:shadow-md">
        <CardContent className="p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Eye className="h-5 w-5 text-primary" />
            Synopsis
          </h3>
          <div className="space-y-3">
            <p className="text-muted-foreground leading-relaxed">
              {showFullSynopsis ? synopsis : truncatedSynopsis}
            </p>
            {shouldShowReadMore && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowFullSynopsis(!showFullSynopsis)}
                className="text-primary hover:text-primary/80 p-0 h-auto font-medium"
              >
                {showFullSynopsis ? "Read Less" : "Read More"}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Enhanced Basic Information */}
        <Card className="transition-all duration-300 hover:shadow-md">
          <CardContent className="p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Calendar className="h-5 w-5 text-primary" />
              Information
            </h3>
            <div className="space-y-3">
              <div className="flex justify-between items-center py-2 border-b transition-colors hover:bg-muted/50 px-2 rounded">
                <span className="text-muted-foreground">Status</span>
                <Badge
                  variant={
                    anime.status === "Finished" ? "default" : "secondary"
                  }
                  className="transition-colors hover:scale-105"
                >
                  {anime.status}
                </Badge>
              </div>
              <div className="flex justify-between items-center py-2 border-b transition-colors hover:bg-muted/50 px-2 rounded">
                <span className="text-muted-foreground">Source</span>
                <span className="font-medium">{anime.source || "Unknown"}</span>
              </div>
              {anime.ageRestriction && (
                <div className="flex justify-between items-center py-2 border-b transition-colors hover:bg-muted/50 px-2 rounded">
                  <span className="text-muted-foreground">Age Rating</span>
                  <Badge
                    variant="outline"
                    className="transition-colors hover:scale-105"
                  >
                    {anime.ageRestriction}
                  </Badge>
                </div>
              )}
              <div className="flex justify-between items-center py-2 transition-colors hover:bg-muted/50 px-2 rounded">
                <span className="text-muted-foreground">Aired</span>
                <div className="text-right">
                  <div className="font-medium">{startDate}</div>
                  {endDate !== startDate && (
                    <div className="text-sm text-muted-foreground">
                      to {endDate}
                    </div>
                  )}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Enhanced Quality Metrics */}
        <Card className="transition-all duration-300 hover:shadow-md">
          <CardContent className="p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Award className="h-5 w-5 text-primary" />
              Quality Metrics
            </h3>
            <div className="space-y-4">
              <div className="transition-all duration-300 hover:bg-muted/50 p-2 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium">Popularity</span>
                  <span className="text-sm text-muted-foreground">
                    {popularityScore.toFixed(1)}/10
                  </span>
                </div>
                <Progress
                  value={popularityScore * 10}
                  className="h-2 transition-all duration-500"
                />
              </div>

              <div className="transition-all duration-300 hover:bg-muted/50 p-2 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium">Engagement</span>
                  <span className="text-sm text-muted-foreground">
                    {engagementScore.toFixed(1)}/10
                  </span>
                </div>
                <Progress
                  value={engagementScore * 10}
                  className="h-2 transition-all duration-500"
                />
              </div>

              <div className="transition-all duration-300 hover:bg-muted/50 p-2 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium">Consistency</span>
                  <span className="text-sm text-muted-foreground">
                    {consistencyScore.toFixed(1)}/10
                  </span>
                </div>
                <Progress
                  value={consistencyScore * 10}
                  className="h-2 transition-all duration-500"
                />
              </div>

              <div className="transition-all duration-300 hover:bg-muted/50 p-2 rounded">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium">Audience Reach</span>
                  <span className="text-sm text-muted-foreground">
                    {reachScore.toFixed(1)}/10
                  </span>
                </div>
                <Progress
                  value={reachScore * 10}
                  className="h-2 transition-all duration-500"
                />
              </div>

              {/* Overall Score */}
              <div className="pt-4 border-t">
                <div className="flex items-center justify-between">
                  <span className="font-medium flex items-center gap-2">
                    <Star className="h-4 w-4 text-yellow-500" />
                    Overall Score
                  </span>
                  <div className="flex items-center gap-2">
                    {Array.from({ length: 5 }, (_, i) => {
                      const score = anime.score || 0;
                      const starValue = (i + 1) * 2;
                      const isFull = score >= starValue;
                      const isHalf =
                        score >= starValue - 1 && score < starValue;
                      return (
                        <Star
                          key={i}
                          className={`h-4 w-4 transition-all duration-300 hover:scale-110 ${
                            isFull
                              ? "fill-yellow-400 text-yellow-400"
                              : isHalf
                                ? "fill-yellow-400/50 text-yellow-400"
                                : "text-muted-foreground/30"
                          }`}
                        />
                      );
                    })}
                    <span className="ml-1 font-semibold">
                      {(anime.score || 0).toFixed(1)}/10
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="transition-all duration-300 hover:shadow-md">
          <CardContent className="p-6">
            <h3 className="text-lg font-semibold mb-4">Genres</h3>
            <div className="flex flex-wrap gap-2">
              {anime.genres.slice(0, 6).map((genre) => (
                <Badge
                  key={genre.id}
                  variant="secondary"
                  className="hover:bg-primary hover:text-primary-foreground transition-all duration-300 cursor-pointer hover:scale-105"
                >
                  {genre.name}
                </Badge>
              ))}
              {anime.genres.length > 6 && (
                <Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
                  <CollapsibleTrigger asChild>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-6 px-2 text-xs"
                    >
                      {isExpanded ? (
                        <>
                          <ChevronUp className="h-3 w-3 mr-1" />
                          Show Less
                        </>
                      ) : (
                        <>
                          <ChevronDown className="h-3 w-3 mr-1" />+
                          {anime.genres.length - 6} More
                        </>
                      )}
                    </Button>
                  </CollapsibleTrigger>
                  <CollapsibleContent className="mt-2">
                    <div className="flex flex-wrap gap-2">
                      {anime.genres.slice(6).map((genre) => (
                        <Badge
                          key={genre.id}
                          variant="secondary"
                          className="hover:bg-primary hover:text-primary-foreground transition-all duration-300 cursor-pointer hover:scale-105"
                        >
                          {genre.name}
                        </Badge>
                      ))}
                    </div>
                  </CollapsibleContent>
                </Collapsible>
              )}
            </div>
          </CardContent>
        </Card>

        <Card className="transition-all duration-300 hover:shadow-md">
          <CardContent className="p-6">
            <h3 className="text-lg font-semibold mb-4">Studios</h3>
            <div className="flex flex-wrap gap-2">
              {anime.studios.length > 0 ? (
                anime.studios.map((studio, index) => (
                  <Badge
                    key={index}
                    variant="outline"
                    className="hover:bg-muted transition-all duration-300 cursor-pointer hover:scale-105"
                  >
                    {studio}
                  </Badge>
                ))
              ) : (
                <span className="text-muted-foreground">
                  No studio information available
                </span>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default AnimeOverviewTab;
