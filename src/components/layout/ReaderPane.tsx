import { openUrl } from "@tauri-apps/plugin-opener";
import ArticleContent from "../article/ArticleContent";
import type { Article } from "../../types";

interface ReaderPaneProps {
  article: Article | null;
}

export default function ReaderPane({ article }: ReaderPaneProps) {
  if (!article) {
    return (
      <main
        className="flex-1 h-full overflow-y-auto"
        style={{ backgroundColor: "var(--color-bg-primary)" }}
      >
        <div
          className="flex flex-col items-center justify-center h-full text-center px-4"
          style={{ color: "var(--color-text-muted)" }}
        >
          <p className="text-lg font-medium">No article selected</p>
          <p className="text-sm mt-1">Select an article from the sidebar to start reading</p>
        </div>
      </main>
    );
  }

  const displayContent = article.content || article.summary || "";

  return (
    <main
      className="flex-1 h-full overflow-y-auto"
      style={{ backgroundColor: "var(--color-bg-primary)" }}
    >
      <div className="reader-content py-8 px-6">
        <div className="flex items-start justify-between mb-2">
          <h1 className="text-2xl font-bold" style={{ color: "var(--color-text-primary)" }}>
            {article.title}
          </h1>
          {article.link && (
            <button
              type="button"
              aria-label="Open in browser"
              className="p-1.5 rounded-md hover:opacity-80 transition-opacity shrink-0 ml-4"
              style={{ color: "var(--color-text-secondary)" }}
              title="Open in browser"
              onClick={() => openUrl(article.link!)}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M12 9v4a1 1 0 01-1 1H3a1 1 0 01-1-1V5a1 1 0 011-1h4M9 2h5v5M7 9l7-7" />
              </svg>
            </button>
          )}
        </div>
        <p className="text-sm mb-6" style={{ color: "var(--color-text-muted)" }}>
          {article.author && <span>{article.author}</span>}
          {article.author && article.published_at && " · "}
          {article.published_at && <span>{new Date(article.published_at).toLocaleDateString()}</span>}
          {article.feed_title && <span> · {article.feed_title}</span>}
        </p>
        <ArticleContent content={displayContent} />
      </div>
    </main>
  );
}
