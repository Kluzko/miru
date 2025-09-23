import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Checkbox } from "@/components/ui/checkbox";
import { CheckCircle, XCircle, Plus } from "lucide-react";
import type { EnhancedValidationResult } from "@/types";
import { QualityMetricsSection } from "./quality-metrics-section";

interface QualityMetrics {
  averageConfidenceScore: number;
  providersUsed: string[];
  enhancementOpportunities: number;
  dataQualityScore: number;
}

interface ResultsStepProps {
  validationResult: EnhancedValidationResult;
  qualityMetrics: QualityMetrics | null;
  selectedExisting: Set<string>;
  collectionId?: string;
  isImporting: boolean;
  onToggleExistingSelection: (id: string) => void;
  onImport: () => void;
  onBack: () => void;
}

export function ResultsStep({
  validationResult,
  qualityMetrics,
  selectedExisting,
  collectionId,
  isImporting,
  onToggleExistingSelection,
  onImport,
  onBack,
}: ResultsStepProps) {
  const canImport =
    validationResult.found.length > 0 || selectedExisting.size > 0;

  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="text-center p-4 bg-green-50 rounded-lg border border-green-200">
          <div className="text-2xl font-bold text-green-700">
            {validationResult.found.length}
          </div>
          <div className="text-sm text-green-600">New Anime</div>
        </div>
        <div className="text-center p-4 bg-blue-50 rounded-lg border border-blue-200">
          <div className="text-2xl font-bold text-blue-700">
            {selectedExisting.size}
          </div>
          <div className="text-sm text-blue-600">
            Selected Existing ({validationResult.already_exists.length}{" "}
            available)
          </div>
        </div>
        <div className="text-center p-4 bg-red-50 rounded-lg border border-red-200">
          <div className="text-2xl font-bold text-red-700">
            {validationResult.not_found.length}
          </div>
          <div className="text-sm text-red-600">Not Found</div>
        </div>
      </div>

      {/* Enhanced Quality Metrics */}
      {qualityMetrics && (
        <QualityMetricsSection qualityMetrics={qualityMetrics} />
      )}

      {/* Results List */}
      <ScrollArea className="h-80 border rounded-lg p-4">
        <div className="space-y-2">
          {/* New anime (automatically included) */}
          {validationResult.found.map((item, index) => (
            <div
              key={`found-${index}`}
              className="flex items-center gap-3 p-2 rounded bg-green-50"
            >
              <CheckCircle className="w-4 h-4 text-green-500" />
              <span className="flex-1">{item.input_title}</span>
              <Badge className="bg-green-100 text-green-800">New</Badge>
            </div>
          ))}

          {/* Existing anime (selectable if we have collection context) */}
          {validationResult.already_exists.map((item, index) => (
            <div
              key={`exists-${index}`}
              className="flex items-center gap-3 p-2 rounded bg-blue-50"
            >
              {collectionId ? (
                <Checkbox
                  checked={selectedExisting.has(item.anime.id)}
                  onCheckedChange={() =>
                    onToggleExistingSelection(item.anime.id)
                  }
                  className="w-4 h-4"
                />
              ) : (
                <CheckCircle className="w-4 h-4 text-blue-500" />
              )}
              <span className="flex-1">{item.input_title}</span>
              <div className="flex items-center gap-2">
                <Badge className="bg-blue-100 text-blue-800">
                  In DB ({item.matched_field})
                </Badge>
                {collectionId && selectedExisting.has(item.anime.id) && (
                  <Badge className="bg-green-100 text-green-800">
                    <Plus className="w-3 h-3 mr-1" />
                    Will Add
                  </Badge>
                )}
              </div>
            </div>
          ))}

          {/* Failed anime */}
          {validationResult.not_found.map((item, index) => (
            <div
              key={`failed-${index}`}
              className="flex items-center gap-3 p-2 rounded bg-red-50"
            >
              <XCircle className="w-4 h-4 text-red-500" />
              <span className="flex-1">{item.title}</span>
              <Badge variant="destructive">Failed</Badge>
            </div>
          ))}
        </div>
      </ScrollArea>

      {/* Action Buttons */}
      <div className="flex gap-3 justify-end">
        <Button variant="outline" onClick={onBack}>
          Back to Input
        </Button>
        <Button
          onClick={onImport}
          disabled={!canImport || isImporting}
          className="min-w-[120px]"
        >
          {isImporting
            ? "Importing..."
            : `Import ${validationResult.found.length + selectedExisting.size} Anime`}
        </Button>
      </div>
    </div>
  );
}
