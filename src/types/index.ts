// TypeScript interfaces mirroring Rust structs

export interface FeedWithMeta {
  id: number;
  title: string;
  feed_url: string;
  site_url: string | null;
  description: string | null;
  language: string | null;
  favicon_url: string | null;
  folder_id: number | null;
  last_fetched_at: string | null;
  last_build_date: string | null;
  created_at: string;
  updated_at: string;
  unread_count: number;
}

/** Lightweight article row returned by list and search queries. Has no body. */
export interface ArticleListItem {
  id: number;
  feed_id: number;
  title: string;
  link: string | null;
  author: string | null;
  image_url: string | null;
  published_at: string | null;
  is_read: boolean;
  is_favorite: boolean;
  created_at: string;
  feed_title: string | null;
  feed_favicon_url: string | null;
  feed_site_url: string | null;
}

/** Which articles a bulk read-state change applies to. `feedId: null` means every feed. */
export interface ArticleScope {
  feedId: number | null;
  favoritesOnly: boolean;
}

export type ArticleFilter = "all" | "unread" | "favourites";

export interface Folder {
  id: number;
  name: string;
  feed_count: number;
}

export interface RefreshResult {
  feed_id: number;
  new_articles: number;
  error: string | null;
}

export type SidebarItem =
  | { kind: "filter"; filter: ArticleFilter }
  | { kind: "folder"; folderId: number }
  | { kind: "feed"; feedId: number };
