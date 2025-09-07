// src/features/collection/components/import-dialog.tsx
import { useState, useCallback } from "react";
import { Upload, FileText, Loader2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useImportAnimeBatch } from "../hooks";
import { cn } from "@/lib/utils";

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  collectionId: string;
}

export function ImportDialog({
  isOpen,
  onClose,
  collectionId,
}: ImportDialogProps) {
  const [dragActive, setDragActive] = useState(false);
  const [manualInput, setManualInput] = useState("");
  const importBatch = useImportAnimeBatch();

  const handleDrag = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);

    const file = e.dataTransfer.files[0];
    if (file && (file.type === "text/plain" || file.type === "text/csv")) {
      const text = await file.text();
      const titles = text
        .split(/[,\n]/)
        .map((t) => t.trim())
        .filter(Boolean);
      await handleImport(titles);
    }
  }, []);

  const handleImport = async (titles: string[]) => {
    try {
      await importBatch.mutateAsync(titles);
      onClose();
    } catch (error) {
      console.error("Import failed:", error);
    }
  };

  const handleManualImport = () => {
    const titles = manualInput
      .split(/[,\n]/)
      .map((t) => t.trim())
      .filter(Boolean);
    if (titles.length > 0) {
      handleImport(titles);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Import Anime</DialogTitle>
        </DialogHeader>

        <div className="space-y-6">
          {/* Drag & Drop Zone */}
          <div
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
            className={cn(
              "border-2 border-dashed rounded-lg p-8 text-center transition-colors",
              dragActive
                ? "border-primary bg-primary/10"
                : "border-border hover:border-primary/50",
              importBatch.isPending && "opacity-50 cursor-not-allowed",
            )}
          >
            <Upload className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
            <p className="text-lg font-medium mb-2">
              {importBatch.isPending
                ? "Processing..."
                : "Drop your file here or click to browse"}
            </p>
            <p className="text-sm text-muted-foreground">
              Supports .txt and .csv files with comma or newline separated
              titles
            </p>
            <input
              type="file"
              accept=".txt,.csv"
              className="hidden"
              id="file-upload"
              onChange={async (e) => {
                const file = e.target.files?.[0];
                if (file) {
                  const text = await file.text();
                  const titles = text
                    .split(/[,\n]/)
                    .map((t) => t.trim())
                    .filter(Boolean);
                  await handleImport(titles);
                }
              }}
              disabled={importBatch.isPending}
            />
            <label htmlFor="file-upload">
              <Button
                variant="outline"
                className="mt-4"
                disabled={importBatch.isPending}
                asChild
              >
                <span>
                  <FileText className="h-4 w-4 mr-2" />
                  Choose File
                </span>
              </Button>
            </label>
          </div>

          {/* Manual Input */}
          <div className="space-y-2">
            <label className="text-sm font-medium">
              Or enter titles manually:
            </label>
            <Textarea
              value={manualInput}
              onChange={(e) => setManualInput(e.target.value)}
              placeholder="Enter anime titles separated by commas or new lines..."
              rows={5}
              disabled={importBatch.isPending}
            />
            <Button
              onClick={handleManualImport}
              disabled={importBatch.isPending || !manualInput.trim()}
              className="w-full"
            >
              {importBatch.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Importing...
                </>
              ) : (
                "Import Anime"
              )}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
