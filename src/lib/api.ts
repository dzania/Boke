/**
 * Unified API layer that works in both Tauri (desktop) and web modes.
 *
 * In Tauri mode, uses IPC invoke calls.
 * In web mode, uses HTTP REST API calls.
 */

import type { FeedWithMeta, Article, Folder, RefreshResult, DiscoveredFeed } from "../types";

// Runtime detection - check if we're running in Tauri
const isTauri = typeof window !== "undefined" && "__TAURI__" in window;

// Lazy imports for tree-shaking
type TauriApi = typeof import("./tauri");
type HttpApi = typeof import("./http");

let tauriApi: TauriApi | null = null;
let httpApi: HttpApi | null = null;

async function getTauriApi(): Promise<TauriApi> {
  if (!tauriApi) {
    tauriApi = await import("./tauri");
  }
  return tauriApi;
}

async function getHttpApi(): Promise<HttpApi> {
  if (!httpApi) {
    httpApi = await import("./http");
  }
  return httpApi;
}

// Re-export the ImportResult type
export type { ImportResult } from "./tauri";

// Feed operations

export async function addFeed(url: string): Promise<FeedWithMeta> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.addFeed(url);
  } else {
    const api = await getHttpApi();
    return api.addFeed(url);
  }
}

export async function removeFeed(feedId: number): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.removeFeed(feedId);
  } else {
    const api = await getHttpApi();
    return api.removeFeed(feedId);
  }
}

export async function getFeeds(): Promise<FeedWithMeta[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getFeeds();
  } else {
    const api = await getHttpApi();
    return api.getFeeds();
  }
}

export async function refreshFeed(feedId: number): Promise<RefreshResult> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.refreshFeed(feedId);
  } else {
    const api = await getHttpApi();
    return api.refreshFeed(feedId);
  }
}

export async function refreshAllFeeds(): Promise<RefreshResult[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.refreshAllFeeds();
  } else {
    const api = await getHttpApi();
    return api.refreshAllFeeds();
  }
}

// Article operations

export async function getArticles(
  feedId: number | null,
  offset: number,
  limit: number,
  unreadOnly: boolean,
  favoritesOnly: boolean
): Promise<Article[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getArticles(feedId, offset, limit, unreadOnly, favoritesOnly);
  } else {
    const api = await getHttpApi();
    return api.getArticles(feedId, offset, limit, unreadOnly, favoritesOnly);
  }
}

export async function getArticle(articleId: number): Promise<Article> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getArticle(articleId);
  } else {
    const api = await getHttpApi();
    return api.getArticle(articleId);
  }
}

export async function toggleRead(articleId: number): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.toggleRead(articleId);
  } else {
    const api = await getHttpApi();
    return api.toggleRead(articleId);
  }
}

export async function markAllRead(feedId: number | null): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.markAllRead(feedId);
  } else {
    const api = await getHttpApi();
    return api.markAllRead(feedId);
  }
}

export async function markAllUnread(feedId: number | null): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.markAllUnread(feedId);
  } else {
    const api = await getHttpApi();
    return api.markAllUnread(feedId);
  }
}

export async function toggleFavorite(articleId: number): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.toggleFavorite(articleId);
  } else {
    const api = await getHttpApi();
    return api.toggleFavorite(articleId);
  }
}

export async function getFavoritesCount(): Promise<number> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getFavoritesCount();
  } else {
    const api = await getHttpApi();
    return api.getFavoritesCount();
  }
}

export async function searchArticles(query: string, limit: number): Promise<Article[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.searchArticles(query, limit);
  } else {
    const api = await getHttpApi();
    return api.searchArticles(query, limit);
  }
}

export async function fetchArticleContent(articleId: number): Promise<string> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.fetchArticleContent(articleId);
  } else {
    const api = await getHttpApi();
    return api.fetchArticleContent(articleId);
  }
}

// Folder operations

export async function getFolders(): Promise<Folder[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getFolders();
  } else {
    const api = await getHttpApi();
    return api.getFolders();
  }
}

export async function createFolder(name: string): Promise<Folder> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.createFolder(name);
  } else {
    const api = await getHttpApi();
    return api.createFolder(name);
  }
}

export async function renameFolder(folderId: number, name: string): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.renameFolder(folderId, name);
  } else {
    const api = await getHttpApi();
    return api.renameFolder(folderId, name);
  }
}

export async function deleteFolder(folderId: number): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.deleteFolder(folderId);
  } else {
    const api = await getHttpApi();
    return api.deleteFolder(folderId);
  }
}

export async function moveFeedToFolder(feedId: number, folderId: number | null): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.moveFeedToFolder(feedId, folderId);
  } else {
    const api = await getHttpApi();
    return api.moveFeedToFolder(feedId, folderId);
  }
}

// Settings - only available in Tauri mode

export async function getSetting(key: string): Promise<string | null> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.getSetting(key);
  }
  // In web mode, use localStorage as fallback
  return localStorage.getItem(`boke_${key}`);
}

export async function setSetting(key: string, value: string): Promise<void> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.setSetting(key, value);
  }
  // In web mode, use localStorage as fallback
  localStorage.setItem(`boke_${key}`, value);
}

// Feed discovery - only available in Tauri mode

export async function discoverFeed(url: string): Promise<DiscoveredFeed[]> {
  if (isTauri) {
    const api = await getTauriApi();
    return api.discoverFeed(url);
  }
  // In web mode, feed discovery is handled server-side during addFeed
  throw new Error("discoverFeed is only available in desktop mode");
}

// Helper to check if running in Tauri/desktop mode
export function isDesktopMode(): boolean {
  return isTauri;
}

// Helper to check if running in web/browser mode
export function isWebMode(): boolean {
  return !isTauri;
}
