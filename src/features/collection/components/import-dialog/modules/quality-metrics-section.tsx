import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Star,
  Zap,
  Target,
  TrendingUp,
  ChevronDown,
  ChevronUp,
  BarChart3,
  Clock,
} from "lucide-react";

interface QualityMetrics {
  averageConfidenceScore: number;
  providersUsed: string[];
  enhancementOpportunities: number;
  dataQualityScore: number;
  validationDurationMs?: number;
  importDurationMs?: number;
}

interface QualityMetricsSectionProps {
  qualityMetrics: QualityMetrics;
}

export function QualityMetricsSection({
  qualityMetrics,
}: QualityMetricsSectionProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  // Helper function to format duration
  const formatDuration = (ms: number): string => {
    if (ms < 1000) {
      return `${ms}ms`;
    } else if (ms < 60000) {
      return `${(ms / 1000).toFixed(1)}s`;
    } else {
      const minutes = Math.floor(ms / 60000);
      const seconds = ((ms % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  };

  return (
    <div className="border border-purple-200 rounded-lg overflow-hidden">
      <Button
        variant="ghost"
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full p-4 flex items-center justify-between bg-gradient-to-r from-purple-50 to-blue-50  rounded-none"
      >
        <div className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5 text-purple-600" />
          <span className="font-semibold text-purple-900">Quality Metrics</span>
          <Badge className="bg-purple-100 text-purple-800">🌟 Enhanced</Badge>
        </div>
        {isExpanded ? (
          <ChevronUp className="h-4 w-4 text-purple-600" />
        ) : (
          <ChevronDown className="h-4 w-4 text-purple-600" />
        )}
      </Button>

      {isExpanded && (
        <div className="p-4 bg-gradient-to-r from-purple-50 to-blue-50">
          <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
            <div className="text-center">
              <div className="flex items-center justify-center gap-1 mb-1">
                <Star className="h-4 w-4 text-yellow-500" />
                <span className="text-lg font-bold text-gray-900">
                  {(qualityMetrics.dataQualityScore * 10).toFixed(1)}
                </span>
              </div>
              <div className="text-xs text-gray-600">Quality Score</div>
            </div>
            <div className="text-center">
              <div className="flex items-center justify-center gap-1 mb-1">
                <TrendingUp className="h-4 w-4 text-green-500" />
                <span className="text-lg font-bold text-gray-900">
                  {(qualityMetrics.averageConfidenceScore * 100).toFixed(0)}%
                </span>
              </div>
              <div className="text-xs text-gray-600">Avg Confidence</div>
            </div>
            <div className="text-center">
              <div className="flex items-center justify-center gap-1 mb-1">
                <Zap className="h-4 w-4 text-blue-500" />
                <span className="text-lg font-bold text-gray-900">
                  {qualityMetrics.providersUsed.length}
                </span>
              </div>
              <div className="text-xs text-gray-600">Providers Used</div>
            </div>
            <div className="text-center">
              <div className="flex items-center justify-center gap-1 mb-1">
                <Target className="h-4 w-4 text-purple-500" />
                <span className="text-lg font-bold text-gray-900">
                  {qualityMetrics.enhancementOpportunities}
                </span>
              </div>
              <div className="text-xs text-gray-600">Enhancements</div>
            </div>
            {qualityMetrics.validationDurationMs && (
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 mb-1">
                  <Clock className="h-4 w-4 text-orange-500" />
                  <span className="text-lg font-bold text-gray-900">
                    {formatDuration(qualityMetrics.validationDurationMs)}
                  </span>
                </div>
                <div className="text-xs text-gray-600">Validation</div>
              </div>
            )}
            {qualityMetrics.importDurationMs && (
              <div className="text-center">
                <div className="flex items-center justify-center gap-1 mb-1">
                  <Clock className="h-4 w-4 text-indigo-500" />
                  <span className="text-lg font-bold text-gray-900">
                    {formatDuration(qualityMetrics.importDurationMs)}
                  </span>
                </div>
                <div className="text-xs text-gray-600">Import</div>
              </div>
            )}
          </div>
          {qualityMetrics.providersUsed.length > 0 && (
            <div className="mt-3 pt-3 border-t border-purple-200">
              <div className="flex items-center gap-2 text-xs text-purple-700">
                <span>Sources:</span>
                {qualityMetrics.providersUsed.map((provider) => (
                  <Badge key={provider} variant="outline" className="text-xs">
                    {provider}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
