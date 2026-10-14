import { memo, useEffect, useRef } from "react";
import { ArticleListSkeleton } from "../ui/Skeleton";
import type { ArticleListItem } from "../../types";

interface ArticleListProps {
  articles: ArticleListItem[];
  articlesLoading: boolean;
  articlesError: string | null;
  refreshError: string | null;
  hasFeeds: boolean;
  hasMore: boolean;
  loadingMore: boolean;
  selectedArticleId: number | null;
  /** Keyboard cursor position, or -1 while an article is open. */
  selectedIndex: number;
  onSelectArticle: (article: ArticleListItem) => void;
  onToggleFavorite: (article: ArticleListItem) => void;
  onToggleRead: (article: ArticleListItem) => void;
  onMarkAllRead: () => void;
  onMarkAllUnread: () => void;
  onLoadMore: () => void;
  visible: boolean;
}

/** Start loading the next page this far before the end of the list scrolls into view. */
const LOAD_MORE_MARGIN_PX = 600;

function ArticleList({
  articles,
  articlesLoading,
  articlesError,
  refreshError,
  hasFeeds,
  hasMore,
  loadingMore,
  selectedArticleId,
  selectedIndex,
  onSelectArticle,
  onToggleFavorite,
  onToggleRead,
  onMarkAllRead,
  onMarkAllUnread,
  onLoadMore,
  visible,
}: ArticleListProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = scrollRef.current;
    if (!container || selectedIndex < 0) return;
    const el = container.querySelector(`[data-index="${selectedIndex}"]`);
    if (el) el.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  useEffect(() => {
    const root = scrollRef.current;
    const sentinel = sentinelRef.current;
    if (!root || !sentinel || !hasMore || loadingMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) onLoadMore();
      },
      { root, rootMargin: `${LOAD_MORE_MARGIN_PX}px` }
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [hasMore, loadingMore, onLoadMore, articles.length]);

  if (!visible) return <div />;

  return (
    <div
      className="flex flex-col h-full overflow-hidden border-r"
      style={{
        backgroundColor: "var(--color-bg-sidebar)",
        borderColor: "var(--color-border)",
      }}
    >
      {/* Article toolbar */}
      {articles.length > 0 && (
        <div
          className="flex items-center justify-end gap-1 px-3 py-1 shrink-0 border-b"
          style={{ borderColor: "var(--color-border)" }}
        >
          <button
            type="button"
            aria-label="Mark all as read"
            title="Mark all as read"
            className="p-1 rounded hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-muted)" }}
            onClick={onMarkAllRead}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <path d="M2 8.5l4 4 8-8" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Mark all as unread"
            title="Mark all as unread"
            className="p-1 rounded hover:opacity-80 transition-opacity"
            style={{ color: "var(--color-text-muted)" }}
            onClick={onMarkAllUnread}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <circle cx="8" cy="8" r="3" fill="currentColor" stroke="none" />
            </svg>
          </button>
        </div>
      )}

      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto"
        style={{ overscrollBehavior: "contain" }}
        role="list"
        aria-label="Articles"
      >
        {/* Error banners */}
        {refreshError && (
          <div
            className="px-3 py-2 text-xs border-b"
            style={{
              backgroundColor: "rgba(239,68,68,0.1)",
              borderColor: "var(--color-border)",
              color: "#ef4444",
            }}
          >
            Refresh failed: {refreshError}
          </div>
        )}
        {articlesError && (
          <div
            className="px-3 py-2 text-xs border-b"
            style={{
              backgroundColor: "rgba(239,68,68,0.1)",
              borderColor: "var(--color-border)",
              color: "#ef4444",
            }}
          >
            Failed to load articles
          </div>
        )}

        {articlesLoading ? (
          <ArticleListSkeleton />
        ) : articles.length > 0 ? (
          <>
            {articles.map((article, index) => (
              <ArticleRow
                key={article.id}
                article={article}
                index={index}
                isSelected={selectedArticleId === article.id}
                isCursor={index === selectedIndex}
                onSelect={onSelectArticle}
                onToggleRead={onToggleRead}
                onToggleFavorite={onToggleFavorite}
              />
            ))}
            <div ref={sentinelRef} aria-hidden="true" />
            {loadingMore && (
              <p className="px-4 py-3 text-xs" style={{ color: "var(--color-text-muted)" }}>
                Loading more…
              </p>
            )}
          </>
        ) : (
          <div
            className="flex flex-col items-center justify-center h-full text-center px-4"
            style={{ color: "var(--color-text-muted)" }}
          >
            <svg
              width="28"
              height="28"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              className="mb-2 opacity-30"
            >
              <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
            </svg>
            <p className="text-sm">{hasFeeds ? "No articles yet" : "Add a feed to get started"}</p>
            {hasFeeds && (
              <p className="text-xs mt-1">
                Press{" "}
                <kbd
                  className="px-1 py-0.5 rounded text-[10px]"
                  style={{ backgroundColor: "var(--color-bg-secondary)" }}
                >
                  R
                </kbd>{" "}
                to refresh
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default memo(ArticleList);

interface ArticleRowProps {
  article: ArticleListItem;
  index: number;
  isSelected: boolean;
  isCursor: boolean;
  onSelect: (article: ArticleListItem) => void;
  onToggleRead: (article: ArticleListItem) => void;
  onToggleFavorite: (article: ArticleListItem) => void;
}

/**
 * One list row. Memoized so moving the cursor or toggling a flag re-renders only the rows whose
 * props changed, not the whole list.
 */
const ArticleRow = memo(function ArticleRow({
  article,
  index,
  isSelected,
  isCursor,
  onSelect,
  onToggleRead,
  onToggleFavorite,
}: ArticleRowProps) {
  return (
    <div
      data-index={index}
      role="listitem"
      aria-current={isSelected ? "true" : undefined}
      className="flex items-center border-b transition-colors group/article"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: isSelected || isCursor ? "var(--color-bg-secondary)" : "transparent",
        borderLeft:
          isCursor && !isSelected ? "2px solid var(--color-accent)" : "2px solid transparent",
      }}
    >
      <button
        type="button"
        className="flex-1 min-w-0 text-left px-4 py-3"
        onClick={() => onSelect(article)}
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
      <div className="flex items-center shrink-0 mr-1">
        <button
          type="button"
          aria-label={article.is_read ? "Mark as unread" : "Mark as read"}
          title={article.is_read ? "Mark as unread" : "Mark as read"}
          className="p-1.5 rounded-md transition-opacity"
          style={{ color: "var(--color-text-muted)" }}
          onClick={(e) => {
            e.stopPropagation();
            onToggleRead(article);
          }}
        >
          {article.is_read ? (
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <circle cx="8" cy="8" r="5.5" />
            </svg>
          ) : (
            <svg
              width="14"
              height="14"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <circle cx="8" cy="8" r="5.5" />
              <path d="M5.5 8l2 2 3.5-3.5" />
            </svg>
          )}
        </button>
        <button
          type="button"
          aria-label={article.is_favorite ? "Remove from favourites" : "Add to favourites"}
          title={article.is_favorite ? "Remove from favourites" : "Add to favourites"}
          className="p-1.5 rounded-md transition-opacity"
          style={{
            color: article.is_favorite ? "var(--color-accent)" : "var(--color-text-muted)",
          }}
          onClick={(e) => {
            e.stopPropagation();
            onToggleFavorite(article);
          }}
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 16 16"
            fill={article.is_favorite ? "currentColor" : "none"}
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <path d="M8 1.5l2.1 4.2 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.4.8-4.7L1.2 6.4l4.7-.7z" />
          </svg>
        </button>
      </div>
    </div>
  );
});

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
