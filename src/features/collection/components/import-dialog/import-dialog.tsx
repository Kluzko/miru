"use client";

import React, { useState, useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

// Import hooks from the hooks directory
import { useImportValidation, useImportExecution, useSelection } from "./hooks";

// Import step components from the modules directory
import { InputStep, ValidatingStep, ResultsStep } from "./modules";

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onAnimesImported: (animeIds: string[]) => void;
  collectionId?: string; // Optional: if provided, will also add to collection
}

export function ImportDialog({
  isOpen,
  onClose,
  onAnimesImported,
  collectionId,
}: ImportDialogProps) {
  const [manualInput, setManualInput] = useState("");
  const initialSelectionSet = useRef(false);

  // Custom hooks for state management
  const {
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
  } = useImportValidation();

  const {
    isImporting,
    enhancedMetrics: _enhancedMetrics,
    handleImport,
  } = useImportExecution();

  const {
    selectedExisting,
    toggleExistingSelection,
    setInitialSelection,
    resetSelection,
  } = useSelection();

  // Event handlers
  const onImportClick = async () => {
    if (!validationResult) return;

    try {
      const importedIds = await handleImport(
        validationResult,
        selectedExisting,
        collectionId,
        onAnimesImported,
      );

      if (importedIds) {
        onClose();
        resetDialog();
      }
    } catch (error) {
      // Error handling is done in the hook
    }
  };

  const resetDialog = () => {
    setManualInput("");
    resetValidation();
    resetSelection();
    initialSelectionSet.current = false;
  };

  const handleStepNavigation = {
    backToInput: () => setStep("input"),
    viewResults: () => setStep("results"),
  };

  // Pre-select existing anime when validation completes for collection context
  React.useEffect(() => {
    if (
      collectionId &&
      validationResult &&
      step === "results" &&
      validationResult.already_exists.length > 0 &&
      !initialSelectionSet.current
    ) {
      const existingIds = validationResult.already_exists.map(
        (anime) => anime.anime.id,
      );
      setInitialSelection(existingIds);
      initialSelectionSet.current = true;
    }
  }, [collectionId, validationResult, step, setInitialSelection]);

  // Dialog title based on current step
  const getDialogTitle = () => {
    switch (step) {
      case "input":
        return "Import Anime";
      case "validating":
        return "Validating Anime...";
      case "results":
        return "Import Results";
      default:
        return "Import Anime";
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle>{getDialogTitle()}</DialogTitle>
        </DialogHeader>

        {/* Input Step */}
        {step === "input" && (
          <InputStep
            manualInput={manualInput}
            setManualInput={setManualInput}
            onValidation={handleValidation}
            isValidating={isValidating}
          />
        )}

        {/* Validating Step */}
        {step === "validating" && (
          <ValidatingStep
            isValidating={isValidating}
            validationProgress={validationProgress}
            currentValidating={currentValidating}
            qualityMetrics={qualityMetrics}
            processingDetails={processingDetails}
            onBack={handleStepNavigation.backToInput}
            onViewResults={handleStepNavigation.viewResults}
          />
        )}

        {/* Results Step */}
        {step === "results" && validationResult && (
          <ResultsStep
            validationResult={validationResult}
            qualityMetrics={qualityMetrics}
            selectedExisting={selectedExisting}
            collectionId={collectionId}
            isImporting={isImporting}
            onToggleExistingSelection={toggleExistingSelection}
            onImport={onImportClick}
            onBack={handleStepNavigation.backToInput}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
