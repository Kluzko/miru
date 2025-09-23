import { Button } from "@/components/ui/button";
import { Upload } from "lucide-react";
import { useDropzone } from "react-dropzone";
import { cn } from "@/lib/utils";

interface InputStepProps {
  manualInput: string;
  setManualInput: (value: string) => void;
  onValidation: (titles: string[]) => Promise<void>;
  isValidating: boolean;
}

export function InputStep({
  manualInput,
  setManualInput,
  onValidation,
  isValidating,
}: InputStepProps) {
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
        await onValidation(titles);
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

    await onValidation(titles);
  };

  const titleCount = manualInput.split(/[,\n]/).filter((t) => t.trim()).length;

  return (
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
          Supports .txt and .csv files with anime titles
        </p>
      </div>

      <div className="space-y-3">
        <label className="text-sm font-medium">
          Or enter titles manually (one per line or comma-separated):
        </label>
        <textarea
          value={manualInput}
          onChange={(e) => setManualInput(e.target.value)}
          placeholder="Naruto&#10;One Piece&#10;Attack on Titan"
          className="w-full h-32 p-3 border rounded-lg resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <Button
          onClick={handleManualSubmit}
          disabled={!manualInput.trim() || isValidating}
          className="w-full"
        >
          {isValidating
            ? "Processing..."
            : `Validate ${titleCount} Title${titleCount !== 1 ? "s" : ""}`}
        </Button>
      </div>
    </div>
  );
}
