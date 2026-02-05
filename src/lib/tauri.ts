import { invoke } from "@tauri-apps/api/core";
import type {
  FeedWithMeta,
  Article,
  Folder,
  RefreshResult,
  DiscoveredFeed,
} from "../types";

export function addFeed(url: string): Promise<FeedWithMeta> {
  return invoke<FeedWithMeta>("add_feed", { url });
}

export function removeFeed(feedId: number): Promise<void> {
  return invoke<void>("remove_feed", { feedId });
}

export function getFeeds(): Promise<FeedWithMeta[]> {
  return invoke<FeedWithMeta[]>("get_feeds");
}

export function refreshFeed(feedId: number): Promise<RefreshResult> {
  return invoke<RefreshResult>("refresh_feed", { feedId });
}

export function refreshAllFeeds(): Promise<RefreshResult[]> {
  return invoke<RefreshResult[]>("refresh_all_feeds");
}

export function getArticles(
  feedId: number | null,
  offset: number,
  limit: number,
  unreadOnly: boolean,
  favoritesOnly: boolean,
): Promise<Article[]> {
  return invoke<Article[]>("get_articles", { feedId, offset, limit, unreadOnly, favoritesOnly });
}

export function getArticle(articleId: number): Promise<Article> {
  return invoke<Article>("get_article", { articleId });
}

export function toggleRead(articleId: number): Promise<void> {
  return invoke<void>("toggle_read", { articleId });
}

export function markAllRead(feedId: number | null): Promise<void> {
  return invoke<void>("mark_all_read", { feedId });
}

export function markAllUnread(feedId: number | null): Promise<void> {
  return invoke<void>("mark_all_unread", { feedId });
}

export function toggleFavorite(articleId: number): Promise<void> {
  return invoke<void>("toggle_favorite", { articleId });
}

export function getFavoritesCount(): Promise<number> {
  return invoke<number>("get_favorites_count");
}

export function searchArticles(query: string, limit: number): Promise<Article[]> {
  return invoke<Article[]>("search_articles", { query, limit });
}

export function fetchArticleContent(articleId: number): Promise<string> {
  return invoke<string>("fetch_article_content", { articleId });
}

export function getFolders(): Promise<Folder[]> {
  return invoke<Folder[]>("get_folders");
}

export function createFolder(name: string): Promise<Folder> {
  return invoke<Folder>("create_folder", { name });
}

export function renameFolder(folderId: number, name: string): Promise<void> {
  return invoke<void>("rename_folder", { folderId, name });
}

export function deleteFolder(folderId: number): Promise<void> {
  return invoke<void>("delete_folder", { folderId });
}

export function moveFeedToFolder(feedId: number, folderId: number | null): Promise<void> {
  return invoke<void>("move_feed_to_folder", { feedId, folderId });
}

export interface ImportResult {
  added: number;
  skipped: number;
  errors: string[];
}

export function importOpml(path: string): Promise<ImportResult> {
  return invoke<ImportResult>("import_opml", { path });
}

export function discoverFeed(url: string): Promise<DiscoveredFeed[]> {
  return invoke<DiscoveredFeed[]>("discover_feed", { url });
}

export function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}
