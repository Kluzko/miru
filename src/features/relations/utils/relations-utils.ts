import { Link, Folder, Film, Tv, Play, BookOpen, Calendar } from "lucide-react";

// Local type definition for relation display
export interface RelationLink {
  target_id: string;
  title: string;
  relation_type: string;
  category?: string;
}

// Map relation types to display categories
export const getCategoryFromType = (type: string): string => {
  const lowerType = type.toLowerCase();

  // Main story elements (highest priority)
  if (
    lowerType.includes("sequel") ||
    lowerType.includes("prequel") ||
    lowerType.includes("parent_story") ||
    lowerType.includes("full_story")
  ) {
    return "mainStory";
  }

  // Side stories and spin-offs
  if (
    lowerType.includes("side_story") ||
    lowerType.includes("spin_off") ||
    lowerType.includes("character")
  ) {
    return "sideStories";
  }

  // Movies and films
  if (lowerType.includes("movie") || lowerType.includes("film")) {
    return "movies";
  }

  // TV series
  if (lowerType.includes("tv") || lowerType.includes("series")) {
    return "tvSeries";
  }

  // OVAs and specials
  if (
    lowerType.includes("ova") ||
    lowerType.includes("special") ||
    lowerType.includes("summary")
  ) {
    return "ovas";
  }

  // Manga and source material
  if (lowerType.includes("manga") || lowerType.includes("adaptation")) {
    return "manga";
  }

  return "other";
};

export const getRelationIcon = (category: string) => {
  switch (category) {
    case "mainStory":
      return Link;
    case "sideStories":
      return Folder;
    case "movies":
      return Film;
    case "tvSeries":
      return Tv;
    case "ovas":
      return Play;
    case "manga":
      return BookOpen;
    case "other":
      return Calendar;
    default:
      return Calendar;
  }
};

export const getRelationColor = (category: string): string => {
  switch (category) {
    case "mainStory":
      return "text-emerald-600 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800 bg-emerald-50 dark:bg-emerald-950/20";
    case "sideStories":
      return "text-amber-600 dark:text-amber-400 border-amber-200 dark:border-amber-800 bg-amber-50 dark:bg-amber-950/20";
    case "movies":
      return "text-purple-600 dark:text-purple-400 border-purple-200 dark:border-purple-800 bg-purple-50 dark:bg-purple-950/20";
    case "tvSeries":
      return "text-blue-600 dark:text-blue-400 border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-950/20";
    case "ovas":
      return "text-cyan-600 dark:text-cyan-400 border-cyan-200 dark:border-cyan-800 bg-cyan-50 dark:bg-cyan-950/20";
    case "manga":
      return "text-orange-600 dark:text-orange-400 border-orange-200 dark:border-orange-800 bg-orange-50 dark:bg-orange-950/20";
    case "other":
      return "text-gray-600 dark:text-gray-400 border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-950/20";
    default:
      return "text-slate-600 dark:text-slate-400 border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-950/20";
  }
};

// Group relations by category
export const groupRelationsByType = (relations: RelationLink[]) => {
  return relations.reduce(
    (groups, relation) => {
      // Use backend category if available, otherwise fall back to relation type mapping
      const category = relation.category
        ? getCategoryFromBackend(relation.category)
        : getCategoryFromType(relation.relation_type || "other");

      if (!groups[category]) {
        groups[category] = [];
      }
      groups[category].push(relation);
      return groups;
    },
    {} as Record<string, RelationLink[]>,
  );
};

// Map backend categories to display categories
const getCategoryFromBackend = (backendCategory: string): string => {
  switch (backendCategory) {
    case "mainStory":
      return "mainStory";
    case "sideStory":
      return "sideStories";
    case "movie":
      return "movies";
    case "ova":
      return "ovas";
    case "other":
    default:
      return "other";
  }
};

export const getRelationDisplayName = (relationType: string): string => {
  const relationMap: Record<string, string> = {
    sequel: "Sequel",
    prequel: "Prequel",
    side_story: "Side Story",
    parent_story: "Parent Story",
    summary: "Summary",
    full_story: "Full Story",
    spin_off: "Spin-off",
    adaptation: "Adaptation",
    character: "Character",
    other: "Related",
    alternative_setting: "Alt. Setting",
    alternative_version: "Alt. Version",
  };

  return (
    relationMap[relationType.toLowerCase()] ||
    relationType.replace(/_/g, " ").replace(/\b\w/g, (l) => l.toUpperCase())
  );
};

export const getCategoryDisplayName = (category: string): string => {
  switch (category) {
    case "mainStory":
      return "Main Story Timeline";
    case "sideStories":
      return "Side Stories & Spin-offs";
    case "movies":
      return "Movies & Films";
    case "tvSeries":
      return "TV Series";
    case "ovas":
      return "OVAs & Specials";
    case "manga":
      return "Source Material";
    case "other":
      return "Related Content";
    default:
      return category.replace(/([A-Z])/g, " $1").trim();
  }
};

export const getCategoryDescription = (category: string): string => {
  switch (category) {
    case "mainStory":
      return "Core narrative entries including sequels, prequels, and main storyline";
    case "sideStories":
      return "Character-focused stories and alternative perspectives";
    case "movies":
      return "Feature films and theatrical releases";
    case "tvSeries":
      return "Television series and seasonal content";
    case "ovas":
      return "Original video animations and special episodes";
    case "manga":
      return "Source material and related publications";
    case "other":
      return "Additional related content and adaptations";
    default:
      return "Related content";
  }
};

// Sort relations with priority
export const sortRelations = (relations: any[]) => {
  return relations.sort((a, b) => {
    // Priority order for relation types within main story
    const relationPriority: Record<string, number> = {
      prequel: 1,
      parent_story: 2,
      sequel: 3,
      full_story: 4,
      side_story: 5,
      spin_off: 6,
      character: 7,
      summary: 8,
      adaptation: 9,
      other: 10,
    };

    const aPriority = relationPriority[a.relation_type?.toLowerCase()] || 10;
    const bPriority = relationPriority[b.relation_type?.toLowerCase()] || 10;

    if (aPriority !== bPriority) {
      return aPriority - bPriority;
    }

    // Finally sort alphabetically
    return (a.title || "").localeCompare(b.title || "");
  });
};

// Category priorities for sorting
export const getCategoryPriority = (category: string): number => {
  const priorities = {
    mainStory: 1, // Highest priority - core narrative
    sideStories: 2, // Character stories and spin-offs
    movies: 3, // Feature films
    tvSeries: 4, // TV content
    ovas: 5, // Special content
    manga: 6, // Source material
    other: 7, // Everything else
  };
  return priorities[category as keyof typeof priorities] || 8;
};
