import { useState, useMemo } from "react";
import { useFeeds, useRefreshAllFeeds } from "../../api/feeds";
import { useTags } from "../../api/tags";
import AddFeedDialog from "../feed/AddFeedDialog";
import TagManager from "../tags/TagManager";
import TagBadge from "../tags/TagBadge";
import type { Article, FeedWithMeta } from "../../types";

interface SidebarProps {
  activeFeedId: number | null;
  onSelectFeed: (feedId: number | null) => void;
  selectedArticleId: number | null;
  selectedIndex: number;
  onSelectArticle: (article: Article) => void;
  articles: Article[];
  showAddFeed: boolean;
  onOpenAddFeed: () => void;
  onCloseAddFeed: () => void;
}

export default function Sidebar({
  activeFeedId, onSelectFeed,
  selectedArticleId, selectedIndex, onSelectArticle,
  articles, showAddFeed, onOpenAddFeed, onCloseAddFeed,
}: SidebarProps) {
  const [tagManagerFeed, setTagManagerFeed] = useState<FeedWithMeta | null>(null);
  const [activeTagFilter, setActiveTagFilter] = useState<number | null>(null);
  const { data: feeds } = useFeeds();
  const { data: tags } = useTags();
  const refreshAll = useRefreshAllFeeds();

  const totalUnread = feeds?.reduce((sum, f) => sum + f.unread_count, 0) ?? 0;

  // Group feeds by tag for display
  const { taggedGroups, untaggedFeeds } = useMemo(() => {
    if (!feeds || !tags) return { taggedGroups: [], untaggedFeeds: feeds ?? [] };

    const taggedFeedIds = new Set(tags.flatMap((t) => t.feed_ids));
    const untagged = feeds.filter((f) => !taggedFeedIds.has(f.id));
    const groups = tags
      .filter((t) => t.feed_ids.length > 0)
      .map((tag) => ({
        tag,
        feeds: feeds.filter((f) => tag.feed_ids.includes(f.id)),
      }));

    return { taggedGroups: groups, untaggedFeeds: untagged };
  }, [feeds, tags]);

  // Filter feeds when a tag is selected
  const visibleFeeds = useMemo(() => {
    if (!activeTagFilter || !feeds || !tags) return feeds ?? [];
    const tag = tags.find((t) => t.id === activeTagFilter);
    if (!tag) return feeds ?? [];
    return feeds.filter((f) => tag.feed_ids.includes(f.id));
  }, [feeds, tags, activeTagFilter]);

  const renderFeedButton = (feed: FeedWithMeta) => (
    <button
      key={feed.id}
      type="button"
      className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors mt-0.5 group"
      style={{
        backgroundColor: activeFeedId === feed.id ? "var(--color-bg-secondary)" : "transparent",
        color: "var(--color-text-primary)",
      }}
      onClick={() => onSelectFeed(feed.id)}
      onContextMenu={(e) => {
        e.preventDefault();
        setTagManagerFeed(feed);
      }}
    >
      <span className="truncate mr-2">{feed.title}</span>
      <div className="flex items-center gap-1.5">
        <button
          type="button"
          aria-label="Manage tags"
          className="p-0.5 rounded opacity-0 group-hover:opacity-60 hover:!opacity-100 transition-opacity"
          style={{ color: "var(--color-text-muted)" }}
          title="Manage tags"
          onClick={(e) => {
            e.stopPropagation();
            setTagManagerFeed(feed);
          }}
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M1 9l7-7h6v6l-7 7z" />
            <circle cx="11.5" cy="4.5" r="1" fill="currentColor" stroke="none" />
          </svg>
        </button>
        {feed.unread_count > 0 && (
          <span className="text-xs shrink-0" style={{ color: "var(--color-text-muted)" }}>
            {feed.unread_count}
          </span>
        )}
      </div>
    </button>
  );

  return (
    <aside
      className="flex flex-col h-full overflow-hidden border-r"
      style={{
        backgroundColor: "var(--color-bg-sidebar)",
        borderColor: "var(--color-border)",
      }}
    >
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b shrink-0"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h1 className="text-lg font-semibold" style={{ color: "var(--color-text-primary)" }}>
          Boke
        </h1>
        <div className="flex items-center gap-1">
          <button
            type="button"
            aria-label="Refresh all feeds"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Refresh all feeds"
            onClick={() => refreshAll.mutate()}
            disabled={refreshAll.isPending}
          >
            <svg
              width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"
              className={refreshAll.isPending ? "animate-spin" : ""}
            >
              <path d="M1 8a7 7 0 0113.24-3.18M15 8A7 7 0 011.76 11.18" />
              <path d="M14.24 1v3.82h-3.82M1.76 15v-3.82h3.82" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Add feed"
            className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-secondary)" }}
            title="Add feed"
            onClick={onOpenAddFeed}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M8 3v10M3 8h10" />
            </svg>
          </button>
        </div>
      </div>

      {/* Feed List */}
      <div className="px-2 py-2 shrink-0 overflow-y-auto" style={{ maxHeight: "40%" }}>
        {/* All Feeds */}
        <button
          type="button"
          className="w-full flex items-center justify-between px-3 py-2 rounded-md text-sm transition-colors"
          style={{
            backgroundColor: activeFeedId === null && activeTagFilter === null ? "var(--color-bg-secondary)" : "transparent",
            color: "var(--color-text-primary)",
          }}
          onClick={() => { onSelectFeed(null); setActiveTagFilter(null); }}
        >
          <span className="font-medium">All Feeds</span>
          {totalUnread > 0 && (
            <span className="text-xs px-1.5 py-0.5 rounded-full" style={{ backgroundColor: "var(--color-accent)", color: "white" }}>
              {totalUnread}
            </span>
          )}
        </button>

        {/* Tag filter chips */}
        {tags && tags.length > 0 && (
          <div className="flex flex-wrap gap-1 px-2 mt-1.5 mb-1">
            {tags.map((tag) => (
              <TagBadge
                key={tag.id}
                name={tag.name}
                onClick={() => {
                  setActiveTagFilter(activeTagFilter === tag.id ? null : tag.id);
                  onSelectFeed(null);
                }}
              />
            ))}
          </div>
        )}

        {/* Feeds - grouped by tag or filtered */}
        {activeTagFilter ? (
          visibleFeeds.map(renderFeedButton)
        ) : taggedGroups.length > 0 ? (
          <>
            {taggedGroups.map(({ tag, feeds: groupFeeds }) => (
              <div key={tag.id} className="mt-2">
                <p className="px-3 py-1 text-xs font-medium uppercase tracking-wider" style={{ color: "var(--color-text-muted)" }}>
                  {tag.name}
                </p>
                {groupFeeds.map(renderFeedButton)}
              </div>
            ))}
            {untaggedFeeds.length > 0 && (
              <div className="mt-2">
                <p className="px-3 py-1 text-xs font-medium uppercase tracking-wider" style={{ color: "var(--color-text-muted)" }}>
                  Untagged
                </p>
                {untaggedFeeds.map(renderFeedButton)}
              </div>
            )}
          </>
        ) : (
          feeds?.map(renderFeedButton)
        )}

        {(!feeds || feeds.length === 0) && (
          <div className="text-center py-4" style={{ color: "var(--color-text-muted)" }}>
            <p className="text-sm">No feeds yet</p>
            <p className="text-xs mt-1">Press + to add your first feed</p>
          </div>
        )}
      </div>

      {/* Article List */}
      <div
        className="flex-1 overflow-y-auto border-t"
        style={{ borderColor: "var(--color-border)" }}
      >
        {articles.length > 0 ? (
          articles.map((article, i) => (
            <button
              key={article.id}
              type="button"
              className="w-full text-left px-4 py-3 border-b transition-colors"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor:
                  selectedArticleId === article.id
                    ? "var(--color-bg-secondary)"
                    : i === selectedIndex
                      ? "var(--color-bg-secondary)"
                      : "transparent",
                borderLeft: i === selectedIndex && selectedArticleId !== article.id
                  ? "2px solid var(--color-accent)"
                  : "2px solid transparent",
              }}
              onClick={() => onSelectArticle(article)}
            >
              <div className="flex items-start gap-2">
                {!article.is_read && (
                  <span
                    className="w-2 h-2 rounded-full mt-1.5 shrink-0"
                    style={{ backgroundColor: "var(--color-unread-dot)" }}
                  />
                )}
                <div className="min-w-0 flex-1">
                  <p
                    className="text-sm truncate"
                    style={{
                      color: "var(--color-text-primary)",
                      fontWeight: article.is_read ? 400 : 600,
                    }}
                  >
                    {article.title}
                  </p>
                  <p className="text-xs mt-0.5 truncate" style={{ color: "var(--color-text-muted)" }}>
                    {article.feed_title}
                    {article.published_at && ` · ${formatRelativeTime(article.published_at)}`}
                  </p>
                </div>
              </div>
            </button>
          ))
        ) : (
          <div className="flex items-center justify-center h-full text-center px-4" style={{ color: "var(--color-text-muted)" }}>
            <p className="text-sm">
              {feeds && feeds.length > 0 ? "No articles" : "Add a feed to see articles"}
            </p>
          </div>
        )}
      </div>

      <AddFeedDialog open={showAddFeed} onClose={onCloseAddFeed} />
      {tagManagerFeed && (
        <TagManager feed={tagManagerFeed} open={true} onClose={() => setTagManagerFeed(null)} />
      )}
    </aside>
  );
}

function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffMin < 1) return "now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return date.toLocaleDateString();
}
