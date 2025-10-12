"use client";

import React from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { collectionLogger } from "@/lib/logger";
import { z } from "zod";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Loader2 } from "lucide-react";
import { useUpdateCollection } from "../../../hooks";
import type { Collection } from "@/types";

const editCollectionSchema = z.object({
  name: z
    .string()
    .min(1, "Collection name is required")
    .max(100, "Collection name must be less than 100 characters"),
  description: z
    .string()
    .max(500, "Description must be less than 500 characters")
    .optional()
    .or(z.literal("")),
});

type EditCollectionForm = z.infer<typeof editCollectionSchema>;

interface EditCollectionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  collection: Collection;
}

export function EditCollectionDialog({
  isOpen,
  onClose,
  collection,
}: EditCollectionDialogProps) {
  const updateCollection = useUpdateCollection();

  const form = useForm<EditCollectionForm>({
    resolver: zodResolver(editCollectionSchema),
    defaultValues: {
      name: collection.name,
      description: collection.description || "",
    },
  });

  const onSubmit = async (data: EditCollectionForm) => {
    try {
      await updateCollection.mutateAsync({
        id: collection.id,
        name: data.name,
        description: data.description || null,
      });
      onClose();
      form.reset();
    } catch (error) {
      collectionLogger.error("Failed to update collection", { error });
    }
  };

  // Reset form when collection changes or dialog opens
  React.useEffect(() => {
    if (isOpen) {
      form.reset({
        name: collection.name,
        description: collection.description || "",
      });
    }
  }, [isOpen, collection, form]);

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Edit Collection</DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Collection Name</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="Enter collection name"
                      {...field}
                      disabled={updateCollection.isPending}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description (Optional)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Enter collection description"
                      className="min-h-[100px]"
                      {...field}
                      disabled={updateCollection.isPending}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                disabled={updateCollection.isPending}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={updateCollection.isPending}>
                {updateCollection.isPending && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                Update Collection
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
