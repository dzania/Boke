import { useState } from "react";
import { useFeeds, useRefreshAllFeeds } from "../../api/feeds";
import { useArticles } from "../../api/articles";
import AddFeedDialog from "../feed/AddFeedDialog";
import type { Article } from "../../types";

interface SidebarProps {
  activeFeedId: number | null;
  onSelectFeed: (feedId: number | null) => void;
  selectedArticleId: number | null;
  onSelectArticle: (article: Article) => void;
}

export default function Sidebar({ activeFeedId, onSelectFeed, selectedArticleId, onSelectArticle }: SidebarProps) {
  const [showAddDialog, setShowAddDialog] = useState(false);
  const { data: feeds } = useFeeds();
  const refreshAll = useRefreshAllFeeds();
  const { data: articles } = useArticles(activeFeedId);

  const totalUnread = feeds?.reduce((sum, f) => sum + f.unread_count, 0) ?? 0;

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
            onClick={() => setShowAddDialog(true)}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M8 3v10M3 8h10" />
            </svg>
          </button>
        </div>
      </div>

      {/* Feed List */}
      <div className="px-2 py-2 shrink-0 overflow-y-auto" style={{ maxHeight: "40%" }}>
        <button
          type="button"
          className="w-full flex items-center justify-between px-3 py-2 rounded-md text-sm transition-colors"
          style={{
            backgroundColor: activeFeedId === null ? "var(--color-bg-secondary)" : "transparent",
            color: "var(--color-text-primary)",
          }}
          onClick={() => onSelectFeed(null)}
        >
          <span className="font-medium">All Feeds</span>
          {totalUnread > 0 && (
            <span className="text-xs px-1.5 py-0.5 rounded-full" style={{ backgroundColor: "var(--color-accent)", color: "white" }}>
              {totalUnread}
            </span>
          )}
        </button>

        {feeds?.map((feed) => (
          <button
            key={feed.id}
            type="button"
            className="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors mt-0.5"
            style={{
              backgroundColor: activeFeedId === feed.id ? "var(--color-bg-secondary)" : "transparent",
              color: "var(--color-text-primary)",
            }}
            onClick={() => onSelectFeed(feed.id)}
          >
            <span className="truncate mr-2">{feed.title}</span>
            {feed.unread_count > 0 && (
              <span className="text-xs shrink-0" style={{ color: "var(--color-text-muted)" }}>
                {feed.unread_count}
              </span>
            )}
          </button>
        ))}

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
        {articles && articles.length > 0 ? (
          articles.map((article) => (
            <button
              key={article.id}
              type="button"
              className="w-full text-left px-4 py-3 border-b transition-colors"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: selectedArticleId === article.id ? "var(--color-bg-secondary)" : "transparent",
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

      <AddFeedDialog open={showAddDialog} onClose={() => setShowAddDialog(false)} />
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
