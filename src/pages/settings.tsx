import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useSettingsStore, type TitleLanguage } from "@/stores/settings-store";

import { Info, Monitor, Database } from "lucide-react";

export function SettingsPage() {
  const { preferredTitleLanguage, setPreferredTitleLanguage } =
    useSettingsStore();

  const handleTitleLanguageChange = (value: string) => {
    setPreferredTitleLanguage(value as TitleLanguage);
  };

  return (
    <TooltipProvider>
      <div className="min-h-screen bg-background">
        <div className="container mx-auto p-6 space-y-8">
          <div className="space-y-2">
            <h1 className="text-4xl font-bold tracking-tight">Settings</h1>
            <p className="text-lg text-muted-foreground">
              Configure your preferences and customize your experience.
            </p>
          </div>

          <div className="grid grid-cols-1 xl:grid-cols-2 gap-8 w-full">
            {/* Display Section */}
            <Card className="shadow-sm border-0 bg-card/50 backdrop-blur-sm">
              <CardHeader className="pb-4">
                <div className="flex items-center gap-3">
                  <div className="p-2 rounded-lg bg-primary/10">
                    <Monitor className="h-5 w-5 text-primary" />
                  </div>
                  <div>
                    <CardTitle className="text-xl">Display</CardTitle>
                    <CardDescription className="text-sm">
                      Customize how content is displayed throughout the
                      application.
                    </CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* Preferred Title Language */}
                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Label htmlFor="title-language">
                      Preferred Title Language
                    </Label>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Info className="h-4 w-4 text-muted-foreground cursor-help" />
                      </TooltipTrigger>
                      <TooltipContent className="max-w-xs">
                        <p className="text-xs">
                          If the preferred title language isn't available for an
                          anime, the system will automatically fall back to the
                          next best available option, with the main title as the
                          final fallback.
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </div>
                  <Select
                    value={preferredTitleLanguage}
                    onValueChange={handleTitleLanguageChange}
                  >
                    <SelectTrigger
                      id="title-language"
                      className="w-full max-w-xs"
                    >
                      <SelectValue placeholder="Select preferred language" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="main">Main Title (Default)</SelectItem>
                      <SelectItem value="english">English</SelectItem>
                      <SelectItem value="japanese">Japanese</SelectItem>
                      <SelectItem value="romaji">Romaji</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-sm text-muted-foreground">
                    Choose which title language to display first in anime lists
                    and details.
                  </p>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </TooltipProvider>
  );
}
