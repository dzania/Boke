import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useHotkeys } from "react-hotkeys-hook";
import type { ArticleFilter, ArticleListItem, SidebarItem } from "../types";
import { findCurrentSidebarIndex } from "../lib/sidebarItems";

export type FocusZone = "sidebar" | "list" | "reader";

/** Moving the cursor within this many rows of the end asks for the next page. */
const LOAD_MORE_THRESHOLD_ROWS = 5;

/** Maximum delay between the two presses of `gg`. */
const DOUBLE_G_WINDOW_MS = 500;

interface KeyboardNavConfig {
  articles: ArticleListItem[];
  /** List cursor position. Owned by the caller so it can reset it when the list changes. */
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
  selectedArticle: ArticleListItem | null;
  activeFeedId: number | null;
  activeFilter: ArticleFilter;
  readerRef: React.RefObject<HTMLElement | null>;
  /** True while a modal (search, add feed) owns the keyboard. */
  disabled: boolean;
  sidebarItems: SidebarItem[];
  onSelectArticle: (article: ArticleListItem) => void;
  onClearArticle: () => void;
  onSelectFeed: (feedId: number) => void;
  onSelectFilter: (filter: ArticleFilter) => void;
  onToggleFolder: (folderId: number) => void;
  onToggleRead: (article: ArticleListItem) => void;
  onToggleFavorite: (article: ArticleListItem) => void;
  onOpenInBrowser: (url: string) => void;
  onRefreshFeed: () => void;
  onRefreshAll: () => void;
  onMarkAllRead: () => void;
  onOpenAddFeed: () => void;
  onOpenSearch: () => void;
  onToggleArticleList: () => void;
  onLoadMore: () => void;
}

interface NavState {
  config: KeyboardNavConfig;
  focusZone: FocusZone;
  sidebarIndex: number;
  showShortcuts: boolean;
}

function clamp(index: number, length: number): number {
  return Math.max(0, Math.min(index, length - 1));
}

/**
 * Vim-style keyboard navigation across sidebar, article list and reader.
 *
 * Every handler reads the latest props and state from a ref, so each hotkey is bound once on
 * mount rather than re-bound whenever the article list or a callback identity changes.
 */
export function useKeyboardNav(config: KeyboardNavConfig) {
  const { setSelectedIndex } = config;
  const [focusZone, setFocusZone] = useState<FocusZone>("list");
  const [sidebarIndex, setSidebarIndex] = useState(0);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const lastGPress = useRef(0);

  const latest = useRef<NavState>({ config, focusZone, sidebarIndex, showShortcuts });
  useLayoutEffect(() => {
    latest.current = { config, focusZone, sidebarIndex, showShortcuts };
  });

  // Opening an article moves focus to the reader; closing it returns focus to the list.
  const hasSelection = config.selectedArticle !== null;
  useEffect(() => {
    setFocusZone((zone) => {
      if (hasSelection) return "reader";
      return zone === "reader" ? "list" : zone;
    });
  }, [hasSelection]);

  const navEnabled = !config.disabled && !showShortcuts;
  const opts = { enabled: navEnabled, preventDefault: true } as const;

  /** The article the m/s/v keys act on: the open one in the reader, else the cursor row. */
  const targetArticle = (): ArticleListItem | undefined => {
    const { config, focusZone } = latest.current;
    if (focusZone === "reader") return config.selectedArticle ?? undefined;
    return config.articles[config.selectedIndex];
  };

  const moveCursor = (delta: number) => {
    const { config } = latest.current;
    const next = clamp(config.selectedIndex + delta, config.articles.length);
    config.setSelectedIndex(next);
    if (next >= config.articles.length - LOAD_MORE_THRESHOLD_ROWS) config.onLoadMore();
  };

  const selectSidebarItem = (item: SidebarItem) => {
    const { config } = latest.current;
    if (item.kind === "folder") {
      // Toggling a folder keeps focus in the sidebar.
      config.onToggleFolder(item.folderId);
      return;
    }
    if (item.kind === "filter") config.onSelectFilter(item.filter);
    else config.onSelectFeed(item.feedId);
    setFocusZone("list");
    setSelectedIndex(0);
  };

  const scrollReader = (top: number) => {
    latest.current.config.readerRef.current?.scrollBy({ top, behavior: "smooth" });
  };

  // h: move focus to the sidebar
  useHotkeys(
    "h",
    () => {
      const { config, focusZone } = latest.current;
      if (focusZone !== "list") return;
      setSidebarIndex(
        findCurrentSidebarIndex(config.sidebarItems, config.activeFeedId, config.activeFilter)
      );
      setFocusZone("sidebar");
    },
    opts
  );

  // l, Enter, o: activate the sidebar item, or open the article under the cursor
  useHotkeys(
    "l, enter, o",
    (event) => {
      const { config, focusZone, sidebarIndex } = latest.current;
      if (focusZone === "sidebar") {
        const item = config.sidebarItems[sidebarIndex];
        if (item) selectSidebarItem(item);
        return;
      }
      if (focusZone === "list" && event.key !== "l") {
        const article = config.articles[config.selectedIndex];
        if (article) config.onSelectArticle(article);
      }
    },
    opts
  );

  useHotkeys(
    "j, ArrowDown",
    () => {
      const { config, focusZone } = latest.current;
      if (focusZone === "sidebar") {
        setSidebarIndex((index) => clamp(index + 1, config.sidebarItems.length));
      } else if (focusZone === "reader") {
        scrollReader(100);
      } else {
        moveCursor(1);
      }
    },
    opts
  );

  useHotkeys(
    "k, ArrowUp",
    () => {
      const { config, focusZone } = latest.current;
      if (focusZone === "sidebar") {
        setSidebarIndex((index) => clamp(index - 1, config.sidebarItems.length));
      } else if (focusZone === "reader") {
        scrollReader(-100);
      } else {
        moveCursor(-1);
      }
    },
    opts
  );

  // Escape: close the shortcuts overlay, leave the sidebar, or close the reader
  useHotkeys(
    "escape",
    () => {
      const { config, focusZone, showShortcuts } = latest.current;
      if (showShortcuts) {
        setShowShortcuts(false);
      } else if (focusZone === "sidebar") {
        setFocusZone("list");
      } else if (focusZone === "reader") {
        config.onClearArticle();
      }
    },
    { enabled: !config.disabled, preventDefault: true }
  );

  const inReader = focusZone === "reader";
  useHotkeys("shift+j", () => scrollReader(200), { ...opts, enabled: navEnabled && inReader });
  useHotkeys("shift+k", () => scrollReader(-200), { ...opts, enabled: navEnabled && inReader });
  useHotkeys("space", () => scrollReader(window.innerHeight * 0.75), {
    ...opts,
    enabled: navEnabled && inReader,
  });
  useHotkeys("shift+space", () => scrollReader(-window.innerHeight * 0.75), {
    ...opts,
    enabled: navEnabled && inReader,
  });

  // gg: jump to the top
  useHotkeys(
    "g",
    () => {
      const { focusZone } = latest.current;
      if (focusZone === "reader") return;
      const now = Date.now();
      if (now - lastGPress.current < DOUBLE_G_WINDOW_MS) {
        if (focusZone === "sidebar") setSidebarIndex(0);
        else setSelectedIndex(0);
        lastGPress.current = 0;
      } else {
        lastGPress.current = now;
      }
    },
    opts
  );

  // G: jump to the bottom
  useHotkeys(
    "shift+g",
    () => {
      const { config, focusZone } = latest.current;
      if (focusZone === "sidebar") {
        setSidebarIndex(Math.max(0, config.sidebarItems.length - 1));
      } else if (focusZone === "list") {
        setSelectedIndex(Math.max(0, config.articles.length - 1));
        config.onLoadMore();
      }
    },
    opts
  );

  useHotkeys(
    "m",
    () => {
      const article = targetArticle();
      if (article) latest.current.config.onToggleRead(article);
    },
    opts
  );

  useHotkeys(
    "s",
    () => {
      const article = targetArticle();
      if (article) latest.current.config.onToggleFavorite(article);
    },
    opts
  );

  useHotkeys(
    "v",
    () => {
      const article = targetArticle();
      if (article?.link) latest.current.config.onOpenInBrowser(article.link);
    },
    opts
  );

  useHotkeys("r", () => latest.current.config.onRefreshFeed(), opts);
  useHotkeys("shift+r", () => latest.current.config.onRefreshAll(), opts);
  useHotkeys("shift+a", () => latest.current.config.onMarkAllRead(), opts);
  useHotkeys("b", () => latest.current.config.onToggleArticleList(), opts);

  useHotkeys(
    "a",
    () => {
      if (latest.current.focusZone !== "reader") latest.current.config.onOpenAddFeed();
    },
    opts
  );

  // "/" and Cmd/Ctrl+K open search
  useHotkeys("slash, mod+k", () => latest.current.config.onOpenSearch(), opts);

  // ?: toggle the shortcuts overlay
  useHotkeys("shift+slash", () => setShowShortcuts((visible) => !visible), {
    enabled: !config.disabled,
    preventDefault: true,
  });

  // 1-9: jump to the nth feed in sidebar order
  useHotkeys(
    "1,2,3,4,5,6,7,8,9",
    (event) => {
      const { config } = latest.current;
      const position = Number.parseInt(event.key, 10) - 1;
      let seen = 0;
      for (const item of config.sidebarItems) {
        if (item.kind !== "feed") continue;
        if (seen === position) {
          config.onSelectFeed(item.feedId);
          setSelectedIndex(0);
          setFocusZone("list");
          return;
        }
        seen += 1;
      }
    },
    opts
  );

  return {
    focusZone,
    sidebarIndex,
    showShortcuts,
    setShowShortcuts,
  };
}
