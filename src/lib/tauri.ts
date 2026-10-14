import { invoke } from "@tauri-apps/api/core";
import type { FeedWithMeta, ArticleListItem, ArticleScope, Folder, RefreshResult } from "../types";
import type { ImportResult } from "./http";

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
  favoritesOnly: boolean
): Promise<ArticleListItem[]> {
  return invoke<ArticleListItem[]>("get_articles", {
    feedId,
    offset,
    limit,
    unreadOnly,
    favoritesOnly,
  });
}

export function getArticleContent(articleId: number): Promise<string> {
  return invoke<string>("get_article_content", { articleId });
}

export function setRead(articleId: number, isRead: boolean): Promise<void> {
  return invoke<void>("set_read", { articleId, isRead });
}

export function setFavorite(articleId: number, isFavorite: boolean): Promise<void> {
  return invoke<void>("set_favorite", { articleId, isFavorite });
}

export function markAllRead(scope: ArticleScope): Promise<void> {
  return invoke<void>("mark_all_read", { ...scope });
}

export function markAllUnread(scope: ArticleScope): Promise<void> {
  return invoke<void>("mark_all_unread", { ...scope });
}

export function getFavoritesCount(): Promise<number> {
  return invoke<number>("get_favorites_count");
}

export function searchArticles(query: string, limit: number): Promise<ArticleListItem[]> {
  return invoke<ArticleListItem[]>("search_articles", { query, limit });
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

export function importOpml(path: string): Promise<ImportResult> {
  return invoke<ImportResult>("import_opml", { path });
}
