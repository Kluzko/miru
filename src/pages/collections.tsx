import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { CollectionsGrid } from "@/features/collection/components";

export function CollectionsPage() {
  return (
    <div className="p-6 space-y-6">
      <h1 className="text-3xl font-bold">Collections</h1>
      <Tabs defaultValue="mine">
        <TabsList className="mb-4">
          <TabsTrigger value="mine">My Collections</TabsTrigger>
          <TabsTrigger value="community">Community</TabsTrigger>
        </TabsList>
        <TabsContent value="mine">
          <CollectionsGrid />
        </TabsContent>
        <TabsContent value="community">
          <h1 className="text-lg font-medium">
            Community collections feature is under development.
          </h1>
        </TabsContent>
      </Tabs>
    </div>
  );
}
