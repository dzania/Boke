/**
 * Backend API that works in both Tauri (desktop) and web modes.
 *
 * The transport is chosen once at startup: IPC `invoke` calls on desktop, the HTTP REST API in
 * the browser. Both modules expose the same functions, except for OPML import, which takes a
 * file path on desktop and a `File` in the browser.
 */

import { isDesktop } from "./platform";
import * as tauri from "./tauri";
import * as http from "./http";
import type { ImportResult } from "./http";

export type { ImportResult };

const backend = isDesktop() ? tauri : http;

export const {
  addFeed,
  removeFeed,
  getFeeds,
  refreshFeed,
  refreshAllFeeds,
  getArticles,
  getArticleContent,
  setRead,
  setFavorite,
  markAllRead,
  markAllUnread,
  getFavoritesCount,
  searchArticles,
  getFolders,
  createFolder,
  renameFolder,
  deleteFolder,
  moveFeedToFolder,
} = backend;

/**
 * Import feeds from an OPML file: a file path on desktop, a `File` in the browser.
 */
export function importOpml(fileOrPath: string | File): Promise<ImportResult> {
  if (typeof fileOrPath === "string") {
    if (!isDesktop()) {
      return Promise.reject(new Error("In web mode, importOpml expects a File object"));
    }
    return tauri.importOpml(fileOrPath);
  }
  if (isDesktop()) {
    return Promise.reject(new Error("In desktop mode, importOpml expects a file path string"));
  }
  return http.importOpml(fileOrPath);
}
