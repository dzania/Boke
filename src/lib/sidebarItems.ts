import type { FeedWithMeta, Folder, SidebarItem } from "../types";

/**
 * Build a flat navigable array matching sidebar visual order:
 * 1. Smart filters: All Feeds, Favourites, Unread
 * 2. For each folder: folder header, then child feeds (if not collapsed)
 * 3. Unfoldered feeds
 */
export function buildSidebarItems(
  feeds: FeedWithMeta[],
  folders: Folder[],
  collapsedFolders: Set<number>
): SidebarItem[] {
  const items: SidebarItem[] = [
    { kind: "filter", filter: "all" },
    { kind: "filter", filter: "favourites" },
    { kind: "filter", filter: "unread" },
  ];

  for (const folder of folders) {
    items.push({ kind: "folder", folderId: folder.id });
    if (!collapsedFolders.has(folder.id)) {
      for (const feed of feeds) {
        if (feed.folder_id === folder.id) {
          items.push({ kind: "feed", feedId: feed.id });
        }
      }
    }
  }

  for (const feed of feeds) {
    if (!feed.folder_id) {
      items.push({ kind: "feed", feedId: feed.id });
    }
  }

  return items;
}

/**
 * Find the sidebar index matching the current active feed or filter.
 */
export function findCurrentSidebarIndex(
  items: SidebarItem[],
  activeFeedId: number | null,
  activeFilter: "all" | "unread" | "favourites"
): number {
  if (activeFeedId !== null) {
    const idx = items.findIndex((item) => item.kind === "feed" && item.feedId === activeFeedId);
    if (idx >= 0) return idx;
  }
  const idx = items.findIndex((item) => item.kind === "filter" && item.filter === activeFilter);
  return idx >= 0 ? idx : 0;
}
