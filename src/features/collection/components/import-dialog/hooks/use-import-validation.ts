import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/types";
import type { EnhancedValidationResult } from "@/types";

// Custom debounce hook for performance optimization
// @ts-ignore - unused for now, will be used for future UI optimizations
function useDebounce<T extends (...args: any[]) => void>(
  callback: T,
  delay: number,
): T {
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  return useCallback(
    ((...args: any[]) => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      timeoutRef.current = setTimeout(() => {
        callback(...args);
      }, delay);
    }) as T,
    [callback, delay],
  );
}

interface ValidationProgress {
  current: number;
  total: number;
  percentage: number;
  current_title: string;
  status: string;
  found_count: number;
  existing_count: number;
  failed_count: number;
  average_confidence: number;
  providers_used: number;
}

interface ValidationStepInfo {
  step: string;
  description: string;
  title: string;
  provider: string;
  estimated_time_remaining: number;
}

interface QualityMetrics {
  averageConfidenceScore: number;
  providersUsed: string[];
  enhancementOpportunities: number;
  dataQualityScore: number;
}

export function useImportValidation() {
  const [step, setStep] = useState<"input" | "validating" | "results">("input");
  const [validationResult, setValidationResult] =
    useState<EnhancedValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [validationProgress, setValidationProgress] = useState(0);
  const [currentValidating, setCurrentValidating] = useState("");
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetrics | null>(
    null,
  );
  const [processingDetails, setProcessingDetails] =
    useState<ValidationStepInfo | null>(null);

  // Set up real-time progress listeners
  useEffect(() => {
    let progressUnlisten: (() => void) | null = null;
    let stepUnlisten: (() => void) | null = null;

    const setupListeners = async () => {
      try {
        // Listen for main progress updates
        progressUnlisten = await listen<ValidationProgress>(
          "validation-progress",
          (event) => {
            const progress = event.payload;

            // Update progress using real backend percentage
            setValidationProgress(Math.round(progress.percentage));

            // Update current status with detailed information
            setCurrentValidating(
              `${progress.current}/${progress.total} - ${progress.current_title} (${progress.found_count} found, ${progress.existing_count} existing, ${progress.failed_count} failed)`,
            );

            // Update quality metrics in real-time with more realistic data
            if (progress.found_count > 0 || progress.existing_count > 0) {
              setQualityMetrics({
                averageConfidenceScore: progress.average_confidence || 0,
                providersUsed: [`${progress.providers_used} providers used`],
                enhancementOpportunities: Math.max(0, progress.failed_count), // Failures might need enhancement
                dataQualityScore: Math.min(
                  progress.average_confidence || 0,
                  1.0,
                ), // Ensure it's between 0-1
              });
            }

            // Log progress for debugging
            console.log(
              `Validation progress: ${progress.percentage.toFixed(1)}% - ${progress.current_title}`,
            );
          },
        );

        // Listen for detailed step information
        stepUnlisten = await listen<ValidationStepInfo>(
          "validation-step",
          (event) => {
            const stepInfo = event.payload;
            setProcessingDetails(stepInfo);

            // Update current validating with formatted time
            setCurrentValidating(`${stepInfo.description}`);

            console.log(
              `Validation step: ${stepInfo.step} - ${stepInfo.description}`,
            );
          },
        );
      } catch (error) {
        console.warn("Failed to set up validation progress listeners:", error);
      }
    };

    if (isValidating) {
      setupListeners();
    }

    return () => {
      if (progressUnlisten) {
        progressUnlisten();
      }
      if (stepUnlisten) {
        stepUnlisten();
      }
    };
  }, [isValidating]);

  const handleValidation = async (titles: string[]) => {
    setIsValidating(true);
    setStep("validating");
    setValidationProgress(0);
    setQualityMetrics(null);
    setProcessingDetails(null);
    setCurrentValidating(
      `Starting enhanced validation of ${titles.length} anime titles...`,
    );

    try {
      // Use enhanced validation with multi-provider support and quality tracking
      // Real-time progress updates will be handled by the event listeners
      const result = await commands.validateAnimeTitles({ titles });

      if (result.status === "ok") {
        setValidationResult(result.data);

        // Extract quality metrics from enhanced validation
        const enhancedData = result.data;
        const metrics: QualityMetrics = {
          averageConfidenceScore: enhancedData.average_confidence || 0,
          providersUsed: [
            `Found data from ${enhancedData.data_quality_summary.total_providers_used} providers`,
          ],
          enhancementOpportunities:
            enhancedData.data_quality_summary.fields_with_gaps.length || 0,
          dataQualityScore:
            enhancedData.data_quality_summary.average_completeness || 0,
        };
        setQualityMetrics(metrics);

        setValidationProgress(100);
        setCurrentValidating(
          `Enhanced validation complete! Found ${enhancedData.found.length}, Quality Score: ${(metrics.dataQualityScore * 10).toFixed(1)}/10`,
        );

        // Brief delay to show completion before transitioning
        setTimeout(() => {
          setIsValidating(false);
          setStep("results");
        }, 1500);
      } else {
        console.error("Enhanced validation failed:", result.error);
        setCurrentValidating("Enhanced validation failed");
        setIsValidating(false);
        setValidationProgress(100);
      }
    } catch (error) {
      console.error("Enhanced validation error:", error);
      setCurrentValidating("Enhanced validation error occurred");
      setIsValidating(false);
      setValidationProgress(100);
    }
  };

  const resetValidation = () => {
    setStep("input");
    setValidationResult(null);
    setIsValidating(false);
    setValidationProgress(0);
    setCurrentValidating("");
    setQualityMetrics(null);
    setProcessingDetails(null);
  };

  return {
    step,
    validationResult,
    isValidating,
    validationProgress,
    currentValidating,
    qualityMetrics,
    processingDetails,
    handleValidation,
    resetValidation,
    setStep,
  };
}
