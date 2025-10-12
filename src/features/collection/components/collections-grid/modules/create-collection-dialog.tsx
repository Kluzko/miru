import { useState } from "react";
import { collectionLogger } from "@/lib/logger";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useCreateCollection } from "../../../hooks";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import type { CreateCollectionRequest } from "@/types";

const createCollectionSchema = z.object({
  name: z.string().min(1, "Name is required"),
  description: z.string().optional(),
});

export type CreateCollectionForm = z.infer<typeof createCollectionSchema>;

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreateCollectionDialog({ open, onOpenChange }: Props) {
  const [step, setStep] = useState(1);
  const [error, setError] = useState<string | null>(null);
  const createCollection = useCreateCollection();

  const form = useForm<CreateCollectionForm>({
    resolver: zodResolver(createCollectionSchema),
    defaultValues: { name: "", description: "" },
  });

  const handleSubmit = form.handleSubmit(() => setStep(2));

  const handleFinish = async () => {
    const values = createCollectionSchema.parse(form.getValues());
    const payload: CreateCollectionRequest = {
      name: values.name,
      description: values.description?.trim()
        ? values.description.trim()
        : null,
    };
    setError(null);
    try {
      await createCollection.mutateAsync(payload);
      form.reset();
      setStep(1);
      onOpenChange(false);
    } catch (e) {
      collectionLogger.error("Failed to create collection", { error: e });
      setError("Failed to create collection");
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) {
          setStep(1);
          form.reset();
        }
        onOpenChange(o);
      }}
    >
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Create Collection</DialogTitle>
        </DialogHeader>
        {step === 1 && (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="name">
                Name
              </label>
              <Input id="name" {...form.register("name")} />
              {form.formState.errors.name && (
                <p className="text-sm text-destructive">
                  {form.formState.errors.name.message}
                </p>
              )}
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="description">
                Description
              </label>
              <Textarea
                id="description"
                rows={3}
                {...form.register("description")}
              />
            </div>
            <div className="flex justify-end">
              <Button type="submit">Next</Button>
            </div>
          </form>
        )}
        {step === 2 && (
          <div className="space-y-6">
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground">
                Choose how you want to add anime to this collection. Options are
                coming soon.
              </p>
              {error && <p className="text-sm text-destructive">{error}</p>}
            </div>
            <div className="flex flex-col gap-2">
              <Button variant="outline" disabled>
                Import from MAL (coming soon)
              </Button>
              <Button variant="outline" disabled>
                Add manually (coming soon)
              </Button>
            </div>
            <div className="flex justify-between">
              <Button variant="ghost" onClick={() => setStep(1)}>
                Back
              </Button>
              <Button
                onClick={handleFinish}
                disabled={createCollection.isPending}
              >
                Create
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
