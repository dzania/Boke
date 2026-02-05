// TypeScript interfaces mirroring Rust structs

export interface Feed {
  title: string;
  link: string;
  feed_url: string;
  description: string | null;
  language: string | null;
  last_updated: string | null;
  entries: FeedEntry[];
}

export interface FeedEntry {
  id: string;
  title: string;
  link: string;
  content: string | null;
  summary: string | null;
  author: string | null;
  published: string | null;
  updated: string | null;
  categories: string[];
  image_url: string | null;
}

export interface FeedWithMeta {
  id: number;
  title: string;
  feed_url: string;
  site_url: string | null;
  description: string | null;
  language: string | null;
  favicon_url: string | null;
  last_fetched_at: string | null;
  last_build_date: string | null;
  created_at: string;
  updated_at: string;
  unread_count: number;
}

export interface Article {
  id: number;
  feed_id: number;
  guid: string;
  title: string;
  link: string | null;
  author: string | null;
  summary: string | null;
  content: string | null;
  image_url: string | null;
  published_at: string | null;
  is_read: boolean;
  is_favorite: boolean;
  created_at: string;
}

export interface Tag {
  id: number;
  name: string;
}

export interface FeedTag {
  feed_id: number;
  tag_id: number;
}

export interface RefreshResult {
  feed_id: number;
  new_articles: number;
  error: string | null;
}

export interface DiscoveredFeed {
  url: string;
  title: string | null;
  feed_type: string;
}
