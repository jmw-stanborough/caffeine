import type {
  SystemUpdateItem,
  UpdateFilterCategory,
  UpdateSortOption,
} from "../types";
import { getSeverityWeight } from "./severity";

export function filterAndSortUpdates(
  updates: SystemUpdateItem[] | undefined,
  filter: UpdateFilterCategory,
  searchQuery: string,
  sortOption: UpdateSortOption,
): SystemUpdateItem[] {
  if (!updates) return [];

  let list = [...updates];

  if (filter === "pending_download") {
    list = list.filter((u) => !u.is_downloaded);
  } else if (filter === "ready_to_install") {
    list = list.filter((u) => u.is_downloaded);
  } else if (filter === "mandatory") {
    list = list.filter((u) => u.is_mandatory);
  }

  const query = searchQuery.toLowerCase().trim();
  if (query) {
    list = list.filter(
      (u) =>
        u.title.toLowerCase().includes(query) ||
        (u.kb_article_id && u.kb_article_id.toLowerCase().includes(query)) ||
        (u.description && u.description.toLowerCase().includes(query)) ||
        u.categories.some((c) => c.toLowerCase().includes(query)),
    );
  }

  list.sort((a, b) => {
    if (sortOption === "severity") {
      const diff = getSeverityWeight(b.severity) - getSeverityWeight(a.severity);
      if (diff !== 0) return diff;
      return (b.size_bytes ?? 0) - (a.size_bytes ?? 0);
    }
    if (sortOption === "size") {
      return (b.size_bytes ?? 0) - (a.size_bytes ?? 0);
    }
    if (sortOption === "title") {
      return a.title.localeCompare(b.title);
    }
    if (sortOption === "kb") {
      return (a.kb_article_id ?? "").localeCompare(b.kb_article_id ?? "");
    }
    return 0;
  });

  return list;
}
