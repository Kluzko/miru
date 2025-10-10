import { useState } from "react";
import {
  AlertTriangle,
  RefreshCw,
  Search,
  CheckCircle,
  XCircle,
  ChevronDown,
} from "lucide-react";
import { Button } from "@/components/ui/button";

import type { AnimeDetailed } from "@/types/bindings";
import {
  analyzeProviderData,
  hasSufficientData,
} from "@/features/anime/utils/provider-detection";

interface ProviderWarningCardProps {
  anime: AnimeDetailed;
  onEnrich: () => Promise<void>;
  onResync: () => Promise<void>;
}

export function ProviderWarningCard({
  anime,
  onEnrich,
  onResync,
}: ProviderWarningCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isEnriching, setIsEnriching] = useState(false);
  const [isResyncing, setIsResyncing] = useState(false);

  const enrichmentInfo = analyzeProviderData(anime);
  const dataStatus = hasSufficientData(anime);

  // Don't show if data is complete
  if (!enrichmentInfo.hasAnyMissing && dataStatus.issues.length === 0) {
    return null;
  }

  const handleEnrich = async () => {
    setIsEnriching(true);
    try {
      await onEnrich();
    } finally {
      setIsEnriching(false);
    }
  };

  const handleResync = async () => {
    setIsResyncing(true);
    try {
      await onResync();
    } finally {
      setIsResyncing(false);
    }
  };

  const getProviderIcon = (_provider: string, isComplete: boolean) => {
    return isComplete ? (
      <CheckCircle className="h-4 w-4 text-green-500" />
    ) : (
      <XCircle className="h-4 w-4 text-red-500" />
    );
  };

  const getSeverityColor = () => {
    const criticalMissing = enrichmentInfo.missingProviders.filter(
      (p) => p.provider === "anilist" || p.provider === "jikan",
    ).length;

    if (criticalMissing >= 2) return "border-red-200 bg-red-50";
    if (criticalMissing === 1) return "border-amber-200 bg-amber-50";
    return "border-blue-200 bg-blue-50";
  };

  const getSeverityIcon = () => {
    const criticalMissing = enrichmentInfo.missingProviders.filter(
      (p) => p.provider === "anilist" || p.provider === "jikan",
    ).length;

    return criticalMissing > 0 ? (
      <AlertTriangle className="h-5 w-5 text-amber-500" />
    ) : (
      <Search className="h-5 w-5 text-blue-500" />
    );
  };

  return (
    <div className="relative">
      {/* Compact Warning Bar */}
      <div
        className={`flex items-center justify-between p-2 rounded-lg border cursor-pointer transition-all hover:shadow-sm ${
          isExpanded
            ? getSeverityColor()
            : "border-amber-200 bg-amber-50 hover:bg-amber-100"
        }`}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center space-x-2">
          <AlertTriangle className="h-4 w-4 text-amber-500" />
          <span className="text-sm font-medium text-amber-800">
            Provider Data{" "}
            {enrichmentInfo.hasAnyMissing ? "Incomplete" : "Issues"}
          </span>
          <span className="text-xs text-amber-600">
            {enrichmentInfo.missingProviders.length} missing
          </span>
        </div>

        <div className="flex items-center space-x-2">
          <Button
            onClick={(e) => {
              e.stopPropagation();
              if (enrichmentInfo.canEnrich) {
                handleEnrich();
              } else {
                handleResync();
              }
            }}
            disabled={isEnriching || isResyncing}
            variant="outline"
            size="sm"
            className="h-6 px-2 text-xs border-amber-300 hover:bg-amber-100"
          >
            {isEnriching || isResyncing ? (
              <RefreshCw className="h-3 w-3 animate-spin" />
            ) : (
              "Fix"
            )}
          </Button>
          <ChevronDown
            className={`h-4 w-4 text-amber-600 transition-transform ${
              isExpanded ? "rotate-180" : ""
            }`}
          />
        </div>
      </div>

      {/* Expanded Details Drawer */}
      {isExpanded && (
        <div className={`mt-2 border rounded-lg p-4 ${getSeverityColor()}`}>
          <div className="flex items-start space-x-3">
            {getSeverityIcon()}

            <div className="flex-1">
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-sm font-medium text-gray-900">
                  Provider Data Details
                </h4>

                <Button
                  onClick={handleResync}
                  disabled={isResyncing}
                  variant="outline"
                  size="sm"
                  className="flex items-center gap-1 text-xs"
                >
                  {isResyncing ? (
                    <RefreshCw className="h-3 w-3 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3 w-3" />
                  )}
                  Re-sync
                </Button>
              </div>

              <div className="mt-1 text-sm text-gray-600">
                {enrichmentInfo.hasAnyMissing && (
                  <>
                    {enrichmentInfo.missingProviders.length} provider
                    {enrichmentInfo.missingProviders.length > 1 ? "s" : ""}{" "}
                    missing data
                    {enrichmentInfo.canEnrich && (
                      <span className="text-green-600 ml-1">
                        • Can auto-enrich
                      </span>
                    )}
                  </>
                )}
                {dataStatus.issues.length > 0 && (
                  <div className="mt-1">
                    <span className="text-amber-600">
                      {dataStatus.issues.join(", ")}
                    </span>
                  </div>
                )}
              </div>

              <div className="space-y-3">
                {/* Provider Status Grid */}
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                  {[
                    ...enrichmentInfo.availableProviders,
                    ...enrichmentInfo.missingProviders,
                  ]
                    .sort((a, b) => a.provider.localeCompare(b.provider))
                    .map((status) => (
                      <div
                        key={status.provider}
                        className={`flex items-center justify-between p-2 rounded border text-xs ${
                          status.isComplete
                            ? "border-green-200 bg-green-50"
                            : "border-red-200 bg-red-50"
                        }`}
                      >
                        <span className="font-medium capitalize">
                          {status.provider}
                        </span>
                        {getProviderIcon(status.provider, status.isComplete)}
                      </div>
                    ))}
                </div>

                {/* Available Actions */}
                {enrichmentInfo.canEnrich && (
                  <div className="border-t pt-3">
                    <div className="flex flex-col sm:flex-row gap-2">
                      <Button
                        onClick={handleEnrich}
                        disabled={isEnriching}
                        size="sm"
                        className="flex items-center gap-2"
                      >
                        {isEnriching ? (
                          <>
                            <RefreshCw className="h-4 w-4 animate-spin" />
                            Enriching...
                          </>
                        ) : (
                          <>
                            <Search className="h-4 w-4" />
                            Auto-Enrich Missing Data
                          </>
                        )}
                      </Button>

                      <div className="text-xs text-gray-500 flex items-center">
                        {enrichmentInfo.enrichmentSuggestion}
                      </div>
                    </div>
                  </div>
                )}

                {/* Feature Impact */}
                {dataStatus.issues.length > 0 && (
                  <div className="border-t pt-3">
                    <div className="text-xs text-gray-600">
                      <span className="font-medium">Feature Impact:</span>
                      <ul className="list-disc list-inside mt-1 space-y-0.5">
                        {!dataStatus.hasRelationsData && (
                          <li>Franchise relations unavailable</li>
                        )}
                        {!dataStatus.hasRatingsData && (
                          <li>Limited rating and review data</li>
                        )}
                      </ul>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
