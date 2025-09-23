import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Loader2, Clock, ChevronDown, ChevronUp } from "lucide-react";
import { formatTimeRemaining } from "@/lib/time-utils";

interface QualityMetrics {
  averageConfidenceScore: number;
  providersUsed: string[];
  enhancementOpportunities: number;
  dataQualityScore: number;
}

interface ValidationStepInfo {
  step: string;
  description: string;
  title: string;
  provider: string;
  estimated_time_remaining: number;
}

interface ValidatingStepProps {
  isValidating: boolean;
  validationProgress: number;
  currentValidating: string;
  qualityMetrics: QualityMetrics | null;
  processingDetails: ValidationStepInfo | null;
  onBack: () => void;
  onViewResults: () => void;
}

export function ValidatingStep({
  isValidating,
  validationProgress,
  currentValidating,
  qualityMetrics,
  processingDetails,
  onBack,
  onViewResults,
}: ValidatingStepProps) {
  const [showAdvancedInfo, setShowAdvancedInfo] = useState(false);

  // Calculate actual provider count from quality metrics
  const actualProvidersUsed =
    qualityMetrics?.providersUsed[0]?.match(/\d+/)?.[0] || "0";
  const providerList =
    parseInt(actualProvidersUsed) > 0
      ? ["AniList", "MyAnimeList", "Jikan"].slice(
          0,
          parseInt(actualProvidersUsed),
        )
      : ["Multiple providers"];

  return (
    <div className="space-y-6">
      <div className="text-center space-y-4">
        <div className="flex items-center justify-center gap-3">
          <Loader2 className="h-8 w-8 text-blue-600 animate-spin" />
          <div>
            <p className="text-lg font-medium">
              {isValidating
                ? "Multi-Provider Validation"
                : "Validation Complete!"}
            </p>
            <p className="text-sm text-gray-600">
              {isValidating
                ? `Searching ${providerList.join(", ")} ${parseInt(actualProvidersUsed) > 3 ? "and more" : ""}`
                : "All titles have been processed with quality analysis"}
            </p>
          </div>
        </div>

        {/* Basic Progress Display */}
        <div className="space-y-3">
          <Progress value={validationProgress} className="w-full h-3" />
          <div className="flex justify-between text-sm text-gray-600">
            <span>{Math.round(validationProgress)}% complete</span>
            {processingDetails && isValidating && (
              <span className="text-blue-600 font-medium">
                ETA:{" "}
                {formatTimeRemaining(
                  processingDetails.estimated_time_remaining,
                )}
              </span>
            )}
          </div>
        </div>

        {/* Basic Current Status */}
        {currentValidating && isValidating && (
          <div className="bg-blue-50 p-3 rounded-lg border border-blue-200">
            <div className="flex items-center justify-center gap-2">
              <Clock className="h-4 w-4 text-blue-600" />
              <span className="text-sm text-blue-800 font-medium">
                {processingDetails
                  ? `${processingDetails.description} - ${formatTimeRemaining(processingDetails.estimated_time_remaining)} remaining`
                  : currentValidating}
              </span>
            </div>
          </div>
        )}

        {/* Advanced Info Toggle Button */}
        {isValidating && (qualityMetrics || processingDetails) && (
          <div className="flex justify-center">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setShowAdvancedInfo(!showAdvancedInfo)}
              className="text-gray-600 hover:text-gray-900"
            >
              {showAdvancedInfo ? (
                <>
                  <ChevronUp className="h-4 w-4 mr-1" />
                  Hide Advanced Info
                </>
              ) : (
                <>
                  <ChevronDown className="h-4 w-4 mr-1" />
                  Show Advanced Info
                </>
              )}
            </Button>
          </div>
        )}

        {/* Advanced Details (Collapsible) */}
        {showAdvancedInfo && currentValidating && isValidating && (
          <div className="space-y-3 border-t border-gray-200 pt-4">
            {/* Detailed Processing Information */}
            <div className="bg-gray-50 p-4 rounded-lg border">
              <h4 className="text-sm font-medium text-gray-700 mb-2">
                Processing Details
              </h4>
              <div className="space-y-2 text-xs text-gray-600">
                <div className="flex justify-between">
                  <span>Current Title:</span>
                  <span className="font-mono">
                    {processingDetails?.title || "Processing..."}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Step:</span>
                  <span className="capitalize">
                    {processingDetails?.step || "validating"}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Provider:</span>
                  <span>{processingDetails?.provider || "multi-provider"}</span>
                </div>
                <div className="flex justify-between">
                  <span>Status:</span>
                  <span className="font-medium">{currentValidating}</span>
                </div>
              </div>
            </div>

            {/* Real-time Quality Metrics */}
            {qualityMetrics && (
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-gray-700">
                  Quality Metrics (Live)
                </h4>
                <div className="grid grid-cols-3 gap-2">
                  <div className="bg-green-50 p-3 rounded text-center border border-green-200">
                    <div className="text-lg font-bold text-green-700">
                      {qualityMetrics.averageConfidenceScore > 0
                        ? Math.round(
                            qualityMetrics.averageConfidenceScore * 100,
                          )
                        : 0}
                      %
                    </div>
                    <div className="text-xs text-green-600">Confidence</div>
                  </div>
                  <div className="bg-purple-50 p-3 rounded text-center border border-purple-200">
                    <div className="text-lg font-bold text-purple-700">
                      {qualityMetrics.dataQualityScore > 0
                        ? (qualityMetrics.dataQualityScore * 10).toFixed(1)
                        : "0.0"}
                      /10
                    </div>
                    <div className="text-xs text-purple-600">Quality Score</div>
                  </div>
                  <div className="bg-blue-50 p-3 rounded text-center border border-blue-200">
                    <div className="text-lg font-bold text-blue-700">
                      {actualProvidersUsed}
                    </div>
                    <div className="text-xs text-blue-600">Providers</div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {!isValidating && (
          <div className="flex gap-2 justify-center">
            <Button variant="outline" onClick={onBack}>
              Back
            </Button>
            <Button onClick={onViewResults}>View Results</Button>
          </div>
        )}
      </div>
    </div>
  );
}
