import { invoke } from "@tauri-apps/api/core";
import type {
  FeedWithMeta,
  Article,
  Tag,
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
  unreadOnly: boolean
): Promise<Article[]> {
  return invoke<Article[]>("get_articles", { feedId, offset, limit, unreadOnly });
}

export function getArticle(articleId: number): Promise<Article> {
  return invoke<Article>("get_article", { articleId });
}

export function toggleRead(articleId: number): Promise<void> {
  return invoke<void>("toggle_read", { articleId });
}

export function markAllRead(feedId: number): Promise<void> {
  return invoke<void>("mark_all_read", { feedId });
}

export function toggleFavorite(articleId: number): Promise<void> {
  return invoke<void>("toggle_favorite", { articleId });
}

export function searchArticles(query: string, limit: number): Promise<Article[]> {
  return invoke<Article[]>("search_articles", { query, limit });
}

export function getTags(): Promise<Tag[]> {
  return invoke<Tag[]>("get_tags");
}

export function createTag(name: string): Promise<Tag> {
  return invoke<Tag>("create_tag", { name });
}

export function tagFeed(feedId: number, tagId: number): Promise<void> {
  return invoke<void>("tag_feed", { feedId, tagId });
}

export function untagFeed(feedId: number, tagId: number): Promise<void> {
  return invoke<void>("untag_feed", { feedId, tagId });
}

export function deleteTag(tagId: number): Promise<void> {
  return invoke<void>("delete_tag", { tagId });
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
