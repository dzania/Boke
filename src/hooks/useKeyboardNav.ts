import { useState, useCallback, useRef } from "react";
import { useHotkeys } from "react-hotkeys-hook";
import type { Article, FeedWithMeta } from "../types";

interface KeyboardNavConfig {
  articles: Article[];
  selectedArticle: Article | null;
  feeds: FeedWithMeta[];
  activeFeedId: number | null;
  readerRef: React.RefObject<HTMLElement | null>;
  disabled: boolean;
  onSelectArticle: (article: Article) => void;
  onClearArticle: () => void;
  onSelectFeed: (feedId: number | null) => void;
  onToggleRead: (articleId: number) => void;
  onToggleFavorite: (articleId: number) => void;
  onOpenInBrowser: (url: string) => void;
  onRefreshFeed: () => void;
  onRefreshAll: () => void;
  onMarkAllRead: () => void;
  onOpenAddFeed: () => void;
  onOpenSearch: () => void;
}

export function useKeyboardNav(config: KeyboardNavConfig) {
  const {
    articles, selectedArticle, feeds,
    readerRef, disabled,
    onSelectArticle, onClearArticle, onSelectFeed,
    onToggleRead, onToggleFavorite, onOpenInBrowser,
    onRefreshFeed, onRefreshAll, onMarkAllRead,
    onOpenAddFeed, onOpenSearch,
  } = config;

  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const lastGPress = useRef(0);

  const opts = { enabled: !disabled && !showShortcuts, preventDefault: true } as const;
  const inReader = selectedArticle !== null;

  // Clamp index to valid range
  const clampIndex = useCallback(
    (i: number) => Math.max(0, Math.min(i, articles.length - 1)),
    [articles.length],
  );

  // j / ArrowDown — next article
  useHotkeys("j, ArrowDown", () => {
    if (inReader) return;
    setSelectedIndex((i) => clampIndex(i + 1));
  }, opts, [clampIndex, inReader]);

  // k / ArrowUp — previous article
  useHotkeys("k, ArrowUp", () => {
    if (inReader) return;
    setSelectedIndex((i) => clampIndex(i - 1));
  }, opts, [clampIndex, inReader]);

  // Enter / o — open selected article
  useHotkeys("enter, o", () => {
    if (inReader) return;
    const article = articles[selectedIndex];
    if (article) onSelectArticle(article);
  }, opts, [articles, selectedIndex, inReader, onSelectArticle]);

  // Escape — close reader / shortcuts
  useHotkeys("escape", () => {
    if (showShortcuts) {
      setShowShortcuts(false);
    } else if (inReader) {
      onClearArticle();
    }
  }, { enabled: !disabled, preventDefault: true }, [inReader, showShortcuts, onClearArticle]);

  // J — scroll reader down
  useHotkeys("shift+j", () => {
    readerRef.current?.scrollBy({ top: 100, behavior: "smooth" });
  }, { ...opts, enabled: opts.enabled && inReader }, [inReader, readerRef]);

  // K — scroll reader up
  useHotkeys("shift+k", () => {
    readerRef.current?.scrollBy({ top: -100, behavior: "smooth" });
  }, { ...opts, enabled: opts.enabled && inReader }, [inReader, readerRef]);

  // Space — page down in reader
  useHotkeys("space", () => {
    if (!inReader) return;
    readerRef.current?.scrollBy({ top: window.innerHeight * 0.75, behavior: "smooth" });
  }, opts, [inReader, readerRef]);

  // Shift+Space — page up in reader
  useHotkeys("shift+space", () => {
    if (!inReader) return;
    readerRef.current?.scrollBy({ top: -window.innerHeight * 0.75, behavior: "smooth" });
  }, opts, [inReader, readerRef]);

  // g — handle gg sequence manually
  useHotkeys("g", () => {
    if (inReader) return;
    const now = Date.now();
    if (now - lastGPress.current < 500) {
      setSelectedIndex(0);
      lastGPress.current = 0;
    } else {
      lastGPress.current = now;
    }
  }, opts, [inReader]);

  // G — jump to bottom
  useHotkeys("shift+g", () => {
    if (inReader) return;
    setSelectedIndex(Math.max(0, articles.length - 1));
  }, opts, [articles.length, inReader]);

  // m — toggle read
  useHotkeys("m", () => {
    const article = inReader ? selectedArticle : articles[selectedIndex];
    if (article) onToggleRead(article.id);
  }, opts, [articles, selectedIndex, selectedArticle, inReader, onToggleRead]);

  // s — toggle favorite
  useHotkeys("s", () => {
    const article = inReader ? selectedArticle : articles[selectedIndex];
    if (article) onToggleFavorite(article.id);
  }, opts, [articles, selectedIndex, selectedArticle, inReader, onToggleFavorite]);

  // v — open in browser
  useHotkeys("v", () => {
    const article = inReader ? selectedArticle : articles[selectedIndex];
    if (article?.link) onOpenInBrowser(article.link);
  }, opts, [articles, selectedIndex, selectedArticle, inReader, onOpenInBrowser]);

  // r — refresh current feed
  useHotkeys("r", () => {
    onRefreshFeed();
  }, opts, [onRefreshFeed]);

  // R — refresh all feeds
  useHotkeys("shift+r", () => {
    onRefreshAll();
  }, opts, [onRefreshAll]);

  // Shift+A — mark all read
  useHotkeys("shift+a", () => {
    onMarkAllRead();
  }, opts, [onMarkAllRead]);

  // a — open add feed dialog
  useHotkeys("a", () => {
    if (inReader) return;
    onOpenAddFeed();
  }, opts, [inReader, onOpenAddFeed]);

  // / — open search (handled in App.tsx already, but adding here for completeness)
  useHotkeys("/", () => {
    onOpenSearch();
  }, opts, [onOpenSearch]);

  // ? — toggle shortcuts overlay
  useHotkeys("shift+/", () => {
    setShowShortcuts((v) => !v);
  }, { enabled: !disabled, preventDefault: true }, []);

  // 1-9 — switch to nth feed
  useHotkeys("1,2,3,4,5,6,7,8,9", (e) => {
    const n = parseInt(e.key, 10) - 1;
    if (feeds[n]) {
      onSelectFeed(feeds[n].id);
      setSelectedIndex(0);
    }
  }, opts, [feeds, onSelectFeed]);

  return { selectedIndex, setSelectedIndex, showShortcuts, setShowShortcuts };
}
