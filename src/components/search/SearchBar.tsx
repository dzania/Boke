import { useState, useEffect, useRef } from "react";
import { useSearchArticles } from "../../api/search";
import { SEARCH_DEBOUNCE_MS } from "../../lib/constants";
import type { Article } from "../../types";

interface SearchBarProps {
  onSelectArticle: (article: Article) => void;
  onClose: () => void;
}

export default function SearchBar({ onSelectArticle, onClose }: SearchBarProps) {
  const [input, setInput] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const { data: results } = useSearchArticles(debouncedQuery);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Debounce the search query
  useEffect(() => {
    const timer = setTimeout(() => setDebouncedQuery(input.trim()), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [input]);

  // Reset selection when results change
  useEffect(() => {
    setSelectedIndex(0);
  }, [results]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (results) setSelectedIndex((i) => Math.min(i + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && results && results[selectedIndex]) {
      onSelectArticle(results[selectedIndex]);
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]"
      style={{ backgroundColor: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="w-[560px] rounded-lg shadow-xl overflow-hidden"
        style={{
          backgroundColor: "var(--color-bg-primary)",
          border: "1px solid var(--color-border)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <div
          className="flex items-center gap-2 px-4 py-3 border-b"
          style={{ borderColor: "var(--color-border)" }}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            style={{ color: "var(--color-text-muted)", flexShrink: 0 }}
          >
            <circle cx="7" cy="7" r="5" />
            <path d="M11 11l3 3" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search articles..."
            className="flex-1 bg-transparent text-sm outline-none"
            style={{ color: "var(--color-text-primary)" }}
          />
          <kbd
            className="text-xs px-1.5 py-0.5 rounded"
            style={{
              backgroundColor: "var(--color-bg-secondary)",
              color: "var(--color-text-muted)",
            }}
          >
            esc
          </kbd>
        </div>

        {results && results.length > 0 && (
          <div className="max-h-[50vh] overflow-y-auto">
            {results.map((article, i) => (
              <button
                key={article.id}
                type="button"
                className="w-full text-left px-4 py-2.5 transition-colors"
                style={{
                  backgroundColor:
                    i === selectedIndex ? "var(--color-bg-secondary)" : "transparent",
                }}
                onClick={() => {
                  onSelectArticle(article);
                  onClose();
                }}
                onMouseEnter={() => setSelectedIndex(i)}
              >
                <p className="text-sm truncate" style={{ color: "var(--color-text-primary)" }}>
                  {article.title}
                </p>
                <p className="text-xs truncate mt-0.5" style={{ color: "var(--color-text-muted)" }}>
                  {article.feed_title}
                  {article.author && ` · ${article.author}`}
                </p>
              </button>
            ))}
          </div>
        )}

        {results && results.length === 0 && debouncedQuery.length >= 3 && (
          <div className="px-4 py-6 text-center" style={{ color: "var(--color-text-muted)" }}>
            <p className="text-sm">No matches found</p>
          </div>
        )}
      </div>
    </div>
  );
}
