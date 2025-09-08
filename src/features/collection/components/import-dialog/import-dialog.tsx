"use client";

import React, { useState, useRef } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Upload,
  CheckCircle,
  XCircle,
  Loader2,
  Clock,
  Plus,
} from "lucide-react";
import { useDropzone } from "react-dropzone";
import { cn } from "@/lib/utils";
import { Progress } from "@/components/ui/progress";

import { useImportValidation } from "./use-import-validation";
import { useImportExecution } from "./use-import-execution";
import { useSelection } from "./use-selection";

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

  const {
    step,
    validationResult,
    isValidating,
    validationProgress,
    currentValidating,
    handleValidation,
    resetValidation,
    setStep,
  } = useImportValidation();

  const { isImporting, handleImport } = useImportExecution();

  const {
    selectedExisting,
    toggleExistingSelection,
    setInitialSelection,
    resetSelection,
  } = useSelection();

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    accept: { "text/plain": [".txt"], "text/csv": [".csv"] },
    onDrop: async (files) => {
      const file = files[0];
      if (!file) return;

      const text = await file.text();
      const titles = text
        .split(/[,\n]/)
        .map((t) => t.trim())
        .filter(Boolean);

      if (titles.length > 0) {
        await handleValidation(titles);
      }
    },
  });

  const handleManualSubmit = async () => {
    if (!manualInput.trim()) return;

    const titles = manualInput
      .split(/[,\n]/)
      .map((t) => t.trim())
      .filter(Boolean);
    if (titles.length === 0) return;

    await handleValidation(titles);
  };

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

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh]">
        <DialogHeader>
          <DialogTitle>
            {step === "input" && "Import Anime"}
            {step === "validating" && "Checking Anime..."}
            {step === "results" && "Import Results"}
          </DialogTitle>
        </DialogHeader>

        {step === "input" && (
          <div className="space-y-6">
            <div
              {...getRootProps()}
              className={cn(
                "border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors",
                isDragActive
                  ? "border-blue-500 bg-blue-50"
                  : "border-gray-300 hover:border-blue-400 hover:bg-gray-50",
              )}
            >
              <input {...getInputProps()} />
              <Upload className="h-12 w-12 mx-auto mb-4 text-gray-400" />
              <p className="text-lg font-medium mb-2">Drop your file here</p>
              <p className="text-sm text-gray-500">
                Supports .txt and .csv files
              </p>
            </div>

            <div className="space-y-3">
              <label className="text-sm font-medium">
                Or enter titles manually:
              </label>
              <textarea
                value={manualInput}
                onChange={(e) => setManualInput(e.target.value)}
                placeholder="Naruto&#10;One Piece&#10;Attack on Titan"
                className="w-full h-32 p-3 border rounded-lg resize-none"
              />
              <Button
                onClick={handleManualSubmit}
                disabled={!manualInput.trim() || isValidating}
                className="w-full"
              >
                {isValidating
                  ? "Checking..."
                  : `Check ${manualInput.split(/[,\n]/).filter((t) => t.trim()).length} Titles`}
              </Button>
            </div>
          </div>
        )}

        {step === "validating" && (
          <div className="space-y-6">
            <div className="text-center space-y-4">
              <div className="flex items-center justify-center gap-3">
                <Loader2 className="h-8 w-8 text-blue-600 animate-spin" />
                <div>
                  <p className="text-lg font-medium">
                    {isValidating
                      ? "Checking anime availability..."
                      : "Validation Complete!"}
                  </p>
                  <p className="text-sm text-gray-600">
                    {isValidating
                      ? "Please wait while we verify each title"
                      : "All titles have been processed"}
                  </p>
                </div>
              </div>

              <div className="space-y-3">
                <Progress value={validationProgress} className="w-full h-3" />
                <div className="flex justify-between text-sm text-gray-600">
                  <span>{Math.round(validationProgress)}% complete</span>
                  {validationResult && (
                    <span>
                      {validationResult.found.length} new •{" "}
                      {validationResult.already_exists.length} existing •{" "}
                      {validationResult.not_found.length} failed
                    </span>
                  )}
                </div>
              </div>

              {currentValidating && isValidating && (
                <div className="flex items-center justify-center gap-2 text-sm text-gray-600 bg-blue-50 p-3 rounded-lg">
                  <Clock className="h-4 w-4 text-blue-600" />
                  Processing:{" "}
                  <span className="font-medium text-blue-700">
                    {currentValidating}
                  </span>
                </div>
              )}

              {!isValidating && validationResult && (
                <div className="flex gap-2 justify-center">
                  <Button variant="outline" onClick={() => setStep("input")}>
                    Back
                  </Button>
                  <Button onClick={() => setStep("results")}>
                    View Results
                  </Button>
                </div>
              )}
            </div>
          </div>
        )}

        {step === "results" && validationResult && (
          <div className="space-y-6">
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
                          toggleExistingSelection(item.anime.id)
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

            <div className="flex justify-between items-center">
              <Button variant="outline" onClick={() => setStep("input")}>
                Start Over
              </Button>

              {(() => {
                const totalToImport =
                  validationResult.found.length + selectedExisting.size;
                const hasNewAnime = validationResult.found.length > 0;
                const hasSelectedExisting = selectedExisting.size > 0;

                if (totalToImport === 0) {
                  return (
                    <Button disabled size="lg" variant="outline">
                      {collectionId
                        ? "Nothing Selected"
                        : "No New Anime to Import"}
                    </Button>
                  );
                }

                return (
                  <Button
                    onClick={onImportClick}
                    disabled={isImporting}
                    size="lg"
                    className="bg-blue-600 hover:bg-blue-700"
                  >
                    {isImporting ? (
                      <>
                        <Loader2 className="h-5 w-5 mr-2 animate-spin" />
                        {collectionId
                          ? "Adding to Collection..."
                          : "Importing..."}
                      </>
                    ) : (
                      <>
                        <CheckCircle className="h-5 w-5 mr-2" />
                        {collectionId
                          ? hasNewAnime && hasSelectedExisting
                            ? `Import ${validationResult.found.length} + Add ${selectedExisting.size} to Collection`
                            : hasNewAnime
                              ? `Import ${validationResult.found.length} to Collection`
                              : `Add ${selectedExisting.size} to Collection`
                          : `Import ${validationResult.found.length} Anime`}
                      </>
                    )}
                  </Button>
                );
              })()}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
