import { useState, useCallback, useRef } from "react";
import { useHotkeys } from "react-hotkeys-hook";
import type { Article, FeedWithMeta, SidebarItem } from "../types";
import { findCurrentSidebarIndex } from "../lib/sidebarItems";

export type FocusZone = "sidebar" | "list" | "reader";

interface KeyboardNavConfig {
  articles: Article[];
  selectedArticle: Article | null;
  feeds: FeedWithMeta[];
  activeFeedId: number | null;
  activeFilter: "all" | "unread" | "favourites";
  readerRef: React.RefObject<HTMLElement | null>;
  disabled: boolean;
  sidebarItems: SidebarItem[];
  onSelectArticle: (article: Article) => void;
  onClearArticle: () => void;
  onSelectFeed: (feedId: number | null) => void;
  onSelectFilter: (filter: "all" | "unread" | "favourites") => void;
  onToggleFolder: (folderId: number) => void;
  onToggleRead: (articleId: number) => void;
  onToggleFavorite: (articleId: number) => void;
  onOpenInBrowser: (url: string) => void;
  onRefreshFeed: () => void;
  onRefreshAll: () => void;
  onMarkAllRead: () => void;
  onOpenAddFeed: () => void;
  onOpenSearch: () => void;
  onToggleArticleList: () => void;
}

export function useKeyboardNav(config: KeyboardNavConfig) {
  const {
    articles,
    selectedArticle,
    feeds,
    activeFeedId,
    activeFilter,
    readerRef,
    disabled,
    sidebarItems,
    onSelectArticle,
    onClearArticle,
    onSelectFeed,
    onSelectFilter,
    onToggleFolder,
    onToggleRead,
    onToggleFavorite,
    onOpenInBrowser,
    onRefreshFeed,
    onRefreshAll,
    onMarkAllRead,
    onOpenAddFeed,
    onToggleArticleList,
  } = config;

  const [selectedIndex, setSelectedIndex] = useState(0);
  const [focusZone, setFocusZone] = useState<FocusZone>("list");
  const [sidebarIndex, setSidebarIndex] = useState(0);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const lastGPress = useRef(0);

  const opts = { enabled: !disabled && !showShortcuts, preventDefault: true } as const;
  const inReader = focusZone === "reader";
  const inSidebar = focusZone === "sidebar";

  // Clamp helpers
  const clampArticle = useCallback(
    (i: number) => Math.max(0, Math.min(i, articles.length - 1)),
    [articles.length]
  );
  const clampSidebar = useCallback(
    (i: number) => Math.max(0, Math.min(i, sidebarItems.length - 1)),
    [sidebarItems.length]
  );

  // Sync focusZone with selectedArticle
  // When an article is selected, enter reader; when cleared, return to list
  const prevArticleRef = useRef<Article | null>(null);
  if (selectedArticle && !prevArticleRef.current) {
    if (focusZone !== "reader") setFocusZone("reader");
  } else if (!selectedArticle && prevArticleRef.current) {
    if (focusZone === "reader") setFocusZone("list");
  }
  prevArticleRef.current = selectedArticle;

  // Helper: select sidebar item and return to list
  const selectSidebarItem = useCallback(
    (item: SidebarItem) => {
      if (item.kind === "filter") {
        onSelectFilter(item.filter);
      } else if (item.kind === "feed") {
        onSelectFeed(item.feedId);
      } else if (item.kind === "folder") {
        onToggleFolder(item.folderId);
        return; // stay in sidebar after toggling folder
      }
      setFocusZone("list");
      setSelectedIndex(0);
    },
    [onSelectFilter, onSelectFeed, onToggleFolder]
  );

  // h — move focus to sidebar
  useHotkeys(
    "h",
    () => {
      if (inSidebar || inReader) return;
      const idx = findCurrentSidebarIndex(sidebarItems, activeFeedId, activeFilter);
      setSidebarIndex(idx);
      setFocusZone("sidebar");
    },
    opts,
    [inSidebar, inReader, sidebarItems, activeFeedId, activeFilter]
  );

  // l — from sidebar, select item and return to list
  useHotkeys(
    "l",
    () => {
      if (!inSidebar) return;
      const item = sidebarItems[sidebarIndex];
      if (item) selectSidebarItem(item);
    },
    opts,
    [inSidebar, sidebarItems, sidebarIndex, selectSidebarItem]
  );

  // j / ArrowDown — context-dependent navigation
  useHotkeys(
    "j, ArrowDown",
    () => {
      if (inSidebar) {
        setSidebarIndex((i) => clampSidebar(i + 1));
      } else if (inReader) {
        readerRef.current?.scrollBy({ top: 100, behavior: "smooth" });
      } else {
        setSelectedIndex((i) => clampArticle(i + 1));
      }
    },
    opts,
    [clampArticle, clampSidebar, inReader, inSidebar, readerRef]
  );

  // k / ArrowUp — context-dependent navigation
  useHotkeys(
    "k, ArrowUp",
    () => {
      if (inSidebar) {
        setSidebarIndex((i) => clampSidebar(i - 1));
      } else if (inReader) {
        readerRef.current?.scrollBy({ top: -100, behavior: "smooth" });
      } else {
        setSelectedIndex((i) => clampArticle(i - 1));
      }
    },
    opts,
    [clampArticle, clampSidebar, inReader, inSidebar, readerRef]
  );

  // Enter / o — open article or select sidebar item
  useHotkeys(
    "enter, o",
    () => {
      if (inSidebar) {
        const item = sidebarItems[sidebarIndex];
        if (item) selectSidebarItem(item);
        return;
      }
      if (inReader) return;
      const article = articles[selectedIndex];
      if (article) onSelectArticle(article);
    },
    opts,
    [
      articles,
      selectedIndex,
      inReader,
      inSidebar,
      sidebarItems,
      sidebarIndex,
      onSelectArticle,
      selectSidebarItem,
    ]
  );

  // Escape — close reader, exit sidebar, or close shortcuts
  useHotkeys(
    "escape",
    () => {
      if (showShortcuts) {
        setShowShortcuts(false);
      } else if (inSidebar) {
        setFocusZone("list");
      } else if (inReader) {
        onClearArticle();
      }
    },
    { enabled: !disabled, preventDefault: true },
    [inReader, inSidebar, showShortcuts, onClearArticle]
  );

  // J / K — fast scroll reader
  useHotkeys(
    "shift+j",
    () => {
      readerRef.current?.scrollBy({ top: 200, behavior: "smooth" });
    },
    { ...opts, enabled: opts.enabled && inReader },
    [inReader, readerRef]
  );

  useHotkeys(
    "shift+k",
    () => {
      readerRef.current?.scrollBy({ top: -200, behavior: "smooth" });
    },
    { ...opts, enabled: opts.enabled && inReader },
    [inReader, readerRef]
  );

  // Space — page down in reader
  useHotkeys(
    "space",
    () => {
      if (!inReader) return;
      readerRef.current?.scrollBy({ top: window.innerHeight * 0.75, behavior: "smooth" });
    },
    opts,
    [inReader, readerRef]
  );

  // Shift+Space — page up in reader
  useHotkeys(
    "shift+space",
    () => {
      if (!inReader) return;
      readerRef.current?.scrollBy({ top: -window.innerHeight * 0.75, behavior: "smooth" });
    },
    opts,
    [inReader, readerRef]
  );

  // g — handle gg sequence
  useHotkeys(
    "g",
    () => {
      if (inReader) return;
      const now = Date.now();
      if (now - lastGPress.current < 500) {
        if (inSidebar) {
          setSidebarIndex(0);
        } else {
          setSelectedIndex(0);
        }
        lastGPress.current = 0;
      } else {
        lastGPress.current = now;
      }
    },
    opts,
    [inReader, inSidebar]
  );

  // G — jump to bottom
  useHotkeys(
    "shift+g",
    () => {
      if (inReader) return;
      if (inSidebar) {
        setSidebarIndex(Math.max(0, sidebarItems.length - 1));
      } else {
        setSelectedIndex(Math.max(0, articles.length - 1));
      }
    },
    opts,
    [articles.length, sidebarItems.length, inReader, inSidebar]
  );

  // m — toggle read
  useHotkeys(
    "m",
    () => {
      const article = inReader ? selectedArticle : articles[selectedIndex];
      if (article) onToggleRead(article.id);
    },
    opts,
    [articles, selectedIndex, selectedArticle, inReader, onToggleRead]
  );

  // s — toggle favorite
  useHotkeys(
    "s",
    () => {
      const article = inReader ? selectedArticle : articles[selectedIndex];
      if (article) onToggleFavorite(article.id);
    },
    opts,
    [articles, selectedIndex, selectedArticle, inReader, onToggleFavorite]
  );

  // v — open in browser
  useHotkeys(
    "v",
    () => {
      const article = inReader ? selectedArticle : articles[selectedIndex];
      if (article?.link) onOpenInBrowser(article.link);
    },
    opts,
    [articles, selectedIndex, selectedArticle, inReader, onOpenInBrowser]
  );

  // r — refresh current feed
  useHotkeys(
    "r",
    () => {
      onRefreshFeed();
    },
    opts,
    [onRefreshFeed]
  );

  // R — refresh all feeds
  useHotkeys(
    "shift+r",
    () => {
      onRefreshAll();
    },
    opts,
    [onRefreshAll]
  );

  // Shift+A — mark all read
  useHotkeys(
    "shift+a",
    () => {
      onMarkAllRead();
    },
    opts,
    [onMarkAllRead]
  );

  // a — open add feed dialog
  useHotkeys(
    "a",
    () => {
      if (inReader) return;
      onOpenAddFeed();
    },
    opts,
    [inReader, onOpenAddFeed]
  );

  // b — toggle article list panel
  useHotkeys(
    "b",
    () => {
      onToggleArticleList();
    },
    opts,
    [onToggleArticleList]
  );

  // ? — toggle shortcuts overlay
  useHotkeys(
    "shift+/",
    () => {
      setShowShortcuts((v) => !v);
    },
    { enabled: !disabled, preventDefault: true },
    []
  );

  // 1-9 — switch to nth feed
  useHotkeys(
    "1,2,3,4,5,6,7,8,9",
    (e) => {
      const n = parseInt(e.key, 10) - 1;
      if (feeds[n]) {
        onSelectFeed(feeds[n].id);
        setSelectedIndex(0);
        if (inSidebar) setFocusZone("list");
      }
    },
    opts,
    [feeds, onSelectFeed, inSidebar]
  );

  return {
    selectedIndex,
    setSelectedIndex,
    focusZone,
    sidebarIndex,
    showShortcuts,
    setShowShortcuts,
  };
}
