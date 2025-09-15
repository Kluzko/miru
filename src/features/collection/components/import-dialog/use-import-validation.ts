import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { commands } from "@/types";
import type { ValidationResult } from "@/types";

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
  current_title: string;
  processed: number;
  found_count: number;
  existing_count: number;
  failed_count: number;
}

export function useImportValidation() {
  const [step, setStep] = useState<"input" | "validating" | "results">("input");
  const [validationResult, setValidationResult] =
    useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [validationProgress, setValidationProgress] = useState(0);
  const [currentValidating, setCurrentValidating] = useState("");

  // Set up real-time progress listener
  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        unlisten = await listen<ValidationProgress>(
          "validation_progress",
          (event) => {
            const progress = event.payload;

            // Update progress percentage
            const progressPercent =
              progress.total > 0
                ? Math.round((progress.processed / progress.total) * 100)
                : 0;
            setValidationProgress(progressPercent);

            // Update current status
            setCurrentValidating(progress.current_title);

            // Log progress for debugging
            console.log(
              `Validation progress: ${progress.processed}/${progress.total} - ${progress.current_title}`,
            );
          },
        );
      } catch (error) {
        console.warn("Failed to set up validation progress listener:", error);
      }
    };

    if (isValidating) {
      setupListener();
    }

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [isValidating]);

  const handleValidation = async (titles: string[]) => {
    setIsValidating(true);
    setStep("validating");
    setValidationProgress(0);
    setCurrentValidating(
      `Starting validation of ${titles.length} anime titles...`,
    );

    try {
      // Use optimized validation backend function with real-time progress
      const result = await commands.validateAnimeTitles({ titles });

      if (result.status === "ok") {
        setValidationResult(result.data);
        setValidationProgress(100);
        setCurrentValidating("Validation complete!");

        // Brief delay to show completion before transitioning
        setTimeout(() => {
          setIsValidating(false);
          setStep("results");
        }, 1000);
      } else {
        console.error("Validation failed:", result.error);
        setCurrentValidating("Validation failed");
        setIsValidating(false);
        setValidationProgress(100);
      }
    } catch (error) {
      console.error("Validation error:", error);
      setCurrentValidating("Validation error occurred");
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
  };

  return {
    step,
    validationResult,
    isValidating,
    validationProgress,
    currentValidating,
    handleValidation,
    resetValidation,
    setStep,
  };
}
