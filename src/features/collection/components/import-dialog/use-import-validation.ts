import { useState } from "react";
import { commands } from "@/types";
import type { ValidationResult } from "@/types";

export function useImportValidation() {
  const [step, setStep] = useState<"input" | "validating" | "results">("input");
  const [validationResult, setValidationResult] =
    useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [validationProgress, setValidationProgress] = useState(0);
  const [currentValidating, setCurrentValidating] = useState("");

  const handleValidation = async (titles: string[]) => {
    setIsValidating(true);
    setStep("validating");
    setValidationProgress(0);
    setCurrentValidating(`Validating ${titles.length} anime titles...`);

    try {
      // Use new smart validation backend function
      const result = await commands.validateAnimeTitles({ titles });

      if (result.status === "ok") {
        setValidationResult(result.data);
        setValidationProgress(100);
        setCurrentValidating("Validation complete!");
        setIsValidating(false);

        // Use a single timeout to transition to results
        setTimeout(() => {
          setStep("results");
        }, 500);
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
