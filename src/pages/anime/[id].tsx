import { useParams } from "react-router-dom";
import { Star, Calendar, Play, Users, ExternalLink } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function AnimeDetailPage() {
  const { id } = useParams<{ id: string }>();

  return (
    <div className="container mx-auto px-6 py-8 max-w-6xl">
      <Card>
        <CardHeader className="text-center py-12">
          <div className="w-24 h-24 bg-muted rounded-full flex items-center justify-center mx-auto mb-6">
            <Play className="h-12 w-12 text-muted-foreground" />
          </div>
          <CardTitle className="text-3xl mb-2">Anime Details Page</CardTitle>
          <p className="text-muted-foreground text-lg mb-4">
            This page is not implemented yet
          </p>
          <Badge variant="outline" className="text-sm">
            Anime ID: {id}
          </Badge>
        </CardHeader>

        <CardContent className="pb-12">
          <div className="max-w-2xl mx-auto space-y-6">
            <div className="text-center space-y-4">
              <h3 className="text-xl font-semibold">Coming Soon</h3>
              <p className="text-muted-foreground leading-relaxed">
                The full anime details page will include comprehensive
                information such as:
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Card className="p-4">
                <div className="flex items-start gap-3">
                  <Star className="h-5 w-5 text-primary mt-1" />
                  <div>
                    <h4 className="font-medium">Detailed Information</h4>
                    <p className="text-sm text-muted-foreground">
                      Synopsis, ratings, reviews, and user scores
                    </p>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <div className="flex items-start gap-3">
                  <Calendar className="h-5 w-5 text-primary mt-1" />
                  <div>
                    <h4 className="font-medium">Release Information</h4>
                    <p className="text-sm text-muted-foreground">
                      Air dates, seasons, and episode information
                    </p>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <div className="flex items-start gap-3">
                  <Users className="h-5 w-5 text-primary mt-1" />
                  <div>
                    <h4 className="font-medium">Cast & Crew</h4>
                    <p className="text-sm text-muted-foreground">
                      Voice actors, directors, and production staff
                    </p>
                  </div>
                </div>
              </Card>

              <Card className="p-4">
                <div className="flex items-start gap-3">
                  <ExternalLink className="h-5 w-5 text-primary mt-1" />
                  <div>
                    <h4 className="font-medium">External Links</h4>
                    <p className="text-sm text-muted-foreground">
                      MAL, streaming platforms, and official sites
                    </p>
                  </div>
                </div>
              </Card>
            </div>

            <div className="text-center pt-6">
              <p className="text-sm text-muted-foreground">
                In the meantime, you can use the anime drawer for quick
                information
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
