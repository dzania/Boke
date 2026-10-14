import type { FeedWithMeta, ArticleListItem, ArticleScope, Folder, RefreshResult } from "../types";

const API_BASE = import.meta.env.VITE_API_BASE_URL || "/api";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || "Request failed");
  }

  // Handle empty responses (204 No Content)
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

export function addFeed(url: string): Promise<FeedWithMeta> {
  return request("/feeds", {
    method: "POST",
    body: JSON.stringify({ url }),
  });
}

export function removeFeed(feedId: number): Promise<void> {
  return request(`/feeds/${feedId}`, { method: "DELETE" });
}

export function getFeeds(): Promise<FeedWithMeta[]> {
  return request("/feeds");
}

export function refreshFeed(feedId: number): Promise<RefreshResult> {
  return request(`/feeds/${feedId}/refresh`, { method: "POST" });
}

export function refreshAllFeeds(): Promise<RefreshResult[]> {
  return request("/feeds/refresh", { method: "POST" });
}

export function getArticles(
  feedId: number | null,
  offset: number,
  limit: number,
  unreadOnly: boolean,
  favoritesOnly: boolean
): Promise<ArticleListItem[]> {
  const params = new URLSearchParams({
    offset: String(offset),
    limit: String(limit),
    unread_only: String(unreadOnly),
    favorites_only: String(favoritesOnly),
  });
  if (feedId !== null) {
    params.set("feed_id", String(feedId));
  }
  return request(`/articles?${params}`);
}

export function getArticleContent(articleId: number): Promise<string> {
  return request<{ content: string }>(`/articles/${articleId}/content`).then((r) => r.content);
}

export function setRead(articleId: number, isRead: boolean): Promise<void> {
  return request(`/articles/${articleId}/read`, {
    method: "PUT",
    body: JSON.stringify({ value: isRead }),
  });
}

export function setFavorite(articleId: number, isFavorite: boolean): Promise<void> {
  return request(`/articles/${articleId}/favorite`, {
    method: "PUT",
    body: JSON.stringify({ value: isFavorite }),
  });
}

function scopeBody(scope: ArticleScope): string {
  return JSON.stringify({ feed_id: scope.feedId, favorites_only: scope.favoritesOnly });
}

export function markAllRead(scope: ArticleScope): Promise<void> {
  return request("/articles/mark-read", { method: "POST", body: scopeBody(scope) });
}

export function markAllUnread(scope: ArticleScope): Promise<void> {
  return request("/articles/mark-unread", { method: "POST", body: scopeBody(scope) });
}

export function getFavoritesCount(): Promise<number> {
  return request<{ count: number }>("/articles/favorites/count").then((r) => r.count);
}

export function searchArticles(query: string, limit: number): Promise<ArticleListItem[]> {
  const params = new URLSearchParams({
    q: query,
    limit: String(limit),
  });
  return request(`/articles/search?${params}`);
}

export function getFolders(): Promise<Folder[]> {
  return request("/folders");
}

export function createFolder(name: string): Promise<Folder> {
  return request("/folders", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export function renameFolder(folderId: number, name: string): Promise<void> {
  return request(`/folders/${folderId}`, {
    method: "PUT",
    body: JSON.stringify({ name }),
  });
}

export function deleteFolder(folderId: number): Promise<void> {
  return request(`/folders/${folderId}`, { method: "DELETE" });
}

export function moveFeedToFolder(feedId: number, folderId: number | null): Promise<void> {
  if (folderId === null) {
    return request(`/feeds/${feedId}/folder`, { method: "DELETE" });
  }
  return request(`/folders/${folderId}/feeds/${feedId}`, { method: "PUT" });
}

export interface ImportResult {
  added: number;
  skipped: number;
  errors: string[];
}

// OPML import in web mode uses file upload
export async function importOpml(file: File): Promise<ImportResult> {
  const formData = new FormData();
  formData.append("file", file);

  const response = await fetch(`${API_BASE}/feeds/import`, {
    method: "POST",
    body: formData,
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || "Import failed");
  }

  return response.json();
}
