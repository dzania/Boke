import { useState, useEffect } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import ArticleContent from "../article/ArticleContent";
import { ReaderSkeleton } from "../ui/Skeleton";
import { fetchArticleContent } from "../../lib/tauri";
import type { Article } from "../../types";

const CONTENT_SHORT_THRESHOLD = 800;

interface ReaderPaneProps {
  article: Article | null;
  readerRef: React.RefObject<HTMLElement | null>;
  onToggleFavorite: (articleId: number) => void;
  onToggleRead: (articleId: number) => void;
  theme: "light" | "dark";
}

export default function ReaderPane({
  article,
  readerRef,
  onToggleFavorite,
  onToggleRead,
  theme,
}: ReaderPaneProps) {
  const [fullContent, setFullContent] = useState<string | null>(null);
  const [fetching, setFetching] = useState(false);

  useEffect(() => {
    setFullContent(null);
    setFetching(false);

    if (!article) return;

    const existing = article.content || article.summary || "";
    // Strip HTML tags for length check
    const textLen = existing.replace(/<[^>]*>/g, "").length;

    if (textLen < CONTENT_SHORT_THRESHOLD && article.link) {
      setFetching(true);
      fetchArticleContent(article.id)
        .then((content) => {
          // Only use fetched content if it's substantially longer
          const fetchedTextLen = content.replace(/<[^>]*>/g, "").length;
          if (fetchedTextLen > textLen) {
            setFullContent(content);
          }
        })
        .catch(() => {})
        .finally(() => setFetching(false));
    }
  }, [article?.id]);

  if (!article) {
    return (
      <main
        ref={readerRef}
        className="flex-1 h-full min-h-0 overflow-y-auto"
        style={{ backgroundColor: "var(--color-bg-primary)", overscrollBehavior: "contain" }}
      >
        <div
          className="flex flex-col items-center justify-center h-full text-center px-4"
          style={{ color: "var(--color-text-muted)" }}
        >
          <svg
            width="48"
            height="48"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1"
            className="mb-4 opacity-20"
          >
            <path d="M2 3h6a4 4 0 014 4v14a3 3 0 00-3-3H2z" />
            <path d="M22 3h-6a4 4 0 00-4 4v14a3 3 0 013-3h7z" />
          </svg>
          <p className="text-base font-medium">Select an article to read</p>
          <p className="text-sm mt-1 opacity-60">
            Use{" "}
            <kbd
              className="px-1.5 py-0.5 rounded text-xs"
              style={{ backgroundColor: "var(--color-bg-secondary)" }}
            >
              j
            </kbd>
            /
            <kbd
              className="px-1.5 py-0.5 rounded text-xs"
              style={{ backgroundColor: "var(--color-bg-secondary)" }}
            >
              k
            </kbd>{" "}
            to navigate,{" "}
            <kbd
              className="px-1.5 py-0.5 rounded text-xs"
              style={{ backgroundColor: "var(--color-bg-secondary)" }}
            >
              Enter
            </kbd>{" "}
            to open
          </p>
        </div>
      </main>
    );
  }

  const displayContent = fullContent || article.content || article.summary || "";

  return (
    <main
      ref={readerRef}
      className="flex-1 h-full min-h-0 overflow-y-auto"
      style={{ backgroundColor: "var(--color-bg-primary)", overscrollBehavior: "contain" }}
      aria-label={`Reading: ${article.title}`}
    >
      <article className="reader-content py-8 px-6">
        <div className="flex items-start justify-between mb-2">
          <h1 className="text-2xl font-bold" style={{ color: "var(--color-text-primary)" }}>
            {article.title}
          </h1>
          <div className="flex items-center gap-1 shrink-0 ml-4">
            <button
              type="button"
              aria-label={article.is_read ? "Mark as unread" : "Mark as read"}
              className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
              style={{ color: "var(--color-text-secondary)" }}
              title={article.is_read ? "Mark as unread" : "Mark as read"}
              onClick={() => onToggleRead(article.id)}
            >
              {article.is_read ? (
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                >
                  <circle cx="8" cy="8" r="5.5" />
                </svg>
              ) : (
                <svg
                  width="16"
                  height="16"
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
              className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
              style={{
                color: article.is_favorite ? "var(--color-accent)" : "var(--color-text-secondary)",
              }}
              title={article.is_favorite ? "Remove from favourites" : "Add to favourites"}
              onClick={() => onToggleFavorite(article.id)}
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill={article.is_favorite ? "currentColor" : "none"}
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <path d="M8 1.5l2.1 4.2 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.4.8-4.7L1.2 6.4l4.7-.7z" />
              </svg>
            </button>
            {article.link && (
              <button
                type="button"
                aria-label="Open in browser"
                className="p-1.5 rounded-md hover:opacity-80 transition-opacity"
                style={{ color: "var(--color-text-secondary)" }}
                title="Open in browser"
                onClick={() => openUrl(article.link!)}
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                >
                  <path d="M12 9v4a1 1 0 01-1 1H3a1 1 0 01-1-1V5a1 1 0 011-1h4M9 2h5v5M7 9l7-7" />
                </svg>
              </button>
            )}
          </div>
        </div>
        <p className="text-sm mb-6" style={{ color: "var(--color-text-muted)" }}>
          {article.author && <span>{article.author}</span>}
          {article.author && article.published_at && " · "}
          {article.published_at && (
            <span>{new Date(article.published_at).toLocaleDateString()}</span>
          )}
          {article.feed_title && <span> · {article.feed_title}</span>}
        </p>
        {fetching && !fullContent ? (
          <ReaderSkeleton />
        ) : (
          <ArticleContent content={displayContent} theme={theme} />
        )}
      </article>
    </main>
  );
}
